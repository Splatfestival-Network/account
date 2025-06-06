use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use juniper::{EmptyMutation, EmptySubscription};
use once_cell::sync::Lazy;
use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Header, Method, Status};
use rocket::{catch, catchers, routes, Request};
use rocket::response::content::RawXml;
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;
use crate::graphql::{Query, Schema};

mod xml;
mod conntest;
mod nnid;
mod account;
mod error;
mod dsresponse;
mod data_wrapper;
// #[deprecated]
mod grpc;
mod graphql;
mod email;
mod papi;
mod mii_util;
mod json_api;

type Pool = sqlx::Pool<Postgres>;

async fn start_grpc(){
    let act_database_url = env::var("DATABASE_URL").expect("account database url is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url)
        .await
        .expect("unable to create pool");

    let grpc_instance = grpc::AccountService(pool);

    let addr: SocketAddr =
        SocketAddr::from((
            env::var("ROCKET_ADDRESS").ok()
                .map(|v| v.parse().expect("unable to read address"))
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                7071
            )
        );



    tokio::spawn(async move{
        Server::builder()
            .add_service(grpc::grpc::account_server::AccountServer::new(grpc_instance))
            .serve(addr)
            .await
            .expect("unable to start grpc server");
    });

}

#[catch(404)]
fn not_found(_req: &Request) -> (Status, (ContentType, RawXml<&'static str>)) {
    (
        Status::NotFound,
        (
            ContentType::XML,
            RawXml(
                r#"<?xml version="1.0"?>
<errors>
    <error>
        <cause/>
        <code>0008</code>
        <message>Not found</message>
    </error>
</errors>"#,
            ),
        ),
    )
}

#[rocket::launch]
async fn launch() -> _ {
    dotenv().ok();

    start_grpc().await;

    let act_database_url = env::var("DATABASE_URL").expect("account database url is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url).await
        .expect("unable to create pool");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::All)
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    rocket::build()
        .attach(cors.to_cors().unwrap())
        .manage(pool)
        .manage(Schema::new(
            Query,
            EmptyMutation::new(),
            EmptySubscription::new())
        )
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
            nnid::support::validate,
            nnid::support::verify_email,
            nnid::people::create_account,
            nnid::people::get_own_profile,
            nnid::people::get_device_owner,
            nnid::people::get_own_device,
            nnid::people::change_mii,
            nnid::oauth::generate_token::generate_token,
            nnid::provider::get_nex_token,
            nnid::provider::get_service_token,
            nnid::mapped_ids::mapped_ids,
            json_api::oauth::generate_token::generate_token,
            json_api::users::profile::get_own_profile,
            papi::login::login,
            papi::user::get_user,
            // graphql::graphiql,
            // graphql::playground,
            graphql::get_graphql,
            graphql::post_graphql,
        ])
        .register("/", catchers![not_found])
}
