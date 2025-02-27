use std::marker::PhantomData;
use rocket::{Request, Response};
use rocket::http::{Header, Status};
use rocket::response::Responder;
use crate::error::Errors;
use crate::xml::Xml;

pub struct Ds<T>(pub T);

impl<'r, 'o: 'r, T: Responder<'r, 'o>> Responder<'r, 'o> for Ds<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from(self.0.respond_to(request)?)
            .header(Header::new("Server", "Nintendo 3DS (http)"))
            .ok()
    }
}