use rocket::http::Status;
use rocket::Request;
use rocket::response::Responder;
use serde::Serialize;
use rocket::response::Result;
use log::error;
use rocket::response::content::RawXml;

#[derive(Debug)]
pub struct Xml<T>(pub T);

impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for Xml<T>{
    fn respond_to(self, request: &'r Request<'_>) -> Result<'o> {
        match quick_xml::se::to_string(&self.0){
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