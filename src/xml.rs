use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::result;
use rocket::http::Status;
use rocket::{async_trait, Data, Request};
use rocket::response::Responder;
use serde::{Deserialize, Deserializer, Serialize};
use rocket::response::Result;
use log::error;
use quick_xml::se::Serializer;
use quick_xml::{DeError, SeError};
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::response::content::RawXml;
use serde::de::{DeserializeOwned, Error, Visitor};

pub fn serialize_with_version(serializable: &impl Serialize) -> result::Result<Box<str>, SeError>{
    let mut write_dest = "<?xml version=\"1.0\"?>".to_owned();



    serializable.serialize(Serializer::new(&mut write_dest))?;
    Ok(write_dest.into_boxed_str())
}

#[derive(Debug)]
pub struct Xml<T>(pub T);

impl<T> Deref for Xml<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Xml<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for Xml<T>{
    fn respond_to(self, request: &'r Request<'_>) -> Result<'o> {
        match serialize_with_version(&self.0){
            Ok(ser) => {
                RawXml(ser).respond_to(request)
            },
            Err(e) => {
                error!("serialization error: {}", e);
                Err(Status::InternalServerError)
            }
        }
    }
}

#[async_trait]
impl<'r, T: DeserializeOwned> FromData<'r> for Xml<T>{
    type Error = Option<DeError>;

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let data = data.open(1 * ByteUnit::MB);

        let Ok(data) = data.into_string().await else {
            return Outcome::Error((Status::BadRequest, None))
        };



        match quick_xml::de::from_str(&data){
            Ok(v) => Outcome::Success(Self(v)),
            Err(e) => Outcome::Error((Status::BadRequest, Some(e)))
        }
    }
}



pub struct YesNoVal(pub bool);

struct YesNoVisitor;

// #[derive(Debug, Error)]
// #[error("did not find Y or N")]
// struct NotYNError;



impl<'de> Visitor<'de> for YesNoVisitor{
    type Value = YesNoVal;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "expected Y or N")
    }
    
    fn visit_str<E>(self, v: &str) -> result::Result<Self::Value, E>
    where
        E: Error,
    {
        match v{
            "Y" => Ok(YesNoVal(true)),
            "N" => Ok(YesNoVal(false)),
            _ => Err(E::custom("didnt get N or Y"))
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> result::Result<Self::Value, E>
    where
        E: Error,
    {

        const Y_BYTES: &[u8] = "Y".as_bytes();
        const N_BYTES: &[u8] = "N".as_bytes();

        match v{
            Y_BYTES => Ok(YesNoVal(true)),
            N_BYTES => Ok(YesNoVal(false)),
            _ => Err(E::custom("didnt get N or Y"))
        }
    }
    


}

impl<'de> Deserialize<'de> for YesNoVal{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        deserializer.deserialize_str(YesNoVisitor)
    }
}

impl Serialize for YesNoVal{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_char(
            match self.0{
                true => 'Y',
                false => 'N',
            }
        )
    }
}