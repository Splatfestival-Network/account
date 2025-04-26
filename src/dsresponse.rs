use rocket::{Request, Response};
use rocket::http::Header;
use rocket::response::Responder;

pub struct Ds<T>(pub T);

impl<'r, 'o: 'r, T: Responder<'r, 'o>> Responder<'r, 'o> for Ds<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from(self.0.respond_to(request)?)
            .header(Header::new("Server", "Nintendo 3DS (http)"))
            .ok()
    }
}