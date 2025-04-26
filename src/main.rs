

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use juniper::{EmptyMutation, EmptySubscription};
use log::info;
use rocket::fairing::AdHoc;
use rocket::futures::FutureExt;
use rocket::http::Header;
use rocket::routes;
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
#[deprecated]
mod grpc;
mod graphql;
mod email;

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

#[rocket::launch]
async fn launch() -> _ {
    dotenv().ok();

    start_grpc().await;

    let act_database_url = env::var("DATABASE_URL").expect("account database url is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url).await
        .expect("unable to create pool");

    let graph_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url).await
        .expect("unable to create pool");

    rocket::build()
        .manage(pool)
        .manage(graphql::Context(graph_pool))
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
            nnid::oauth::generate_token::generate_token,
            nnid::provider::get_nex_token,
            nnid::provider::get_service_token,
            nnid::mapped_ids::mapped_ids,
            //graphql
            graphql::graphiql,
            graphql::playground,
            graphql::get_graphql,
            graphql::post_graphql,
        ])
}
