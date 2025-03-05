

use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Header;
use rocket::routes;
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;

mod xml;
mod conntest;
mod nnid;
mod account;
mod error;
mod dsresponse;
mod data_wrapper;

type Pool = sqlx::Pool<Postgres>;

#[rocket::launch]
async fn launch() -> _ {
    dotenv().ok();



    let act_database_url = env::var("DATABASE_URL").expect("account database url is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url).await
        .expect("unable to create pool");

    rocket::build()
        .manage(pool)
        .attach(AdHoc::on_response("org", |_, response| Box::pin(async move {
            //response.adjoin_header(Header::new("x-organization", "Nintendo"));
            response.adjoin_header(Header::new("x-nintendo-date", SystemTime::now()
                                                                    .duration_since(UNIX_EPOCH)
                                                                    .unwrap()
                                                                    .as_millis()
                                                                    .to_string()
            ));

            //response.adjoin_header(Header::new("Content-Type", "text/xml; charset=utf-8"));


            response.remove_header("x-content-type-options");
            response.remove_header("x-frame-options");
            response.remove_header("permissions-policy");
        })))
        .mount("/", routes![
            conntest::conntest,
            nnid::devices::current_device_status,
            nnid::agreements::get_agreement,
            nnid::timezones::get_timezone,
            nnid::person_exists::person_exists,
            nnid::email::validate,
            nnid::people::create_account,
            nnid::people::get_own_profile,
            nnid::oauth::generate_token::generate_token
        ])
}
