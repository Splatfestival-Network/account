use rocket::get;
use rocket::response::content::RawHtml;

#[get("/")]
pub fn conntest() -> RawHtml<&'static str>{
    RawHtml(include_str!("../res/conntest.html"))
}