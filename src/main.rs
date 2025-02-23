use rocket::routes;

mod xml;
mod conntest;

#[rocket::launch]
async fn launch() -> _ {
    rocket::build()
        .mount("/", routes![conntest::conntest])
}
