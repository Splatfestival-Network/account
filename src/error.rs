use log::error;
use rocket::http::Status;
use rocket::{Request, Response};
use rocket::response::content::RawXml;
use rocket::response::Responder;
use rocket::serde::Serialize;
use crate::nnid::timezones::Timezone;
use crate::xml::{serialize_with_version, Xml};

#[derive(Serialize, Debug)]
pub struct Error<'a>{
    pub code: &'a str,
    pub message: &'a str
}

#[derive(Serialize, Debug)]
#[serde(rename(serialize = "errors"))]
pub struct Errors<'a>{
    pub error: &'a [Error<'a>],
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Errors<'r> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from(Xml(self).respond_to(request)?)
            .status(Status::BadRequest)
            .ok()
    }
}