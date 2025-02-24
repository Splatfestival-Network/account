use std::env;
use diesel::{Connection, MysqlConnection};
use dotenvy::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::Header;
use rocket::routes;

mod xml;
mod conntest;
mod db;
mod account;

#[rocket::launch]
async fn launch() -> _ {
    dotenv().ok();

    let act_database_url = env::var("ACCOUNT_DATABASE_URL").expect("account database url is not set");

    let conn = MysqlConnection::establish(&act_database_url).expect("unable to connect to database");

    rocket::build()
        .attach(AdHoc::on_response("org", |_, response| Box::pin(async move {
            response.adjoin_header(Header::new("x-organization", "Nintendo"));
        })))
        .mount("/", routes![conntest::conntest])
}
