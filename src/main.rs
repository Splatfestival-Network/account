use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use juniper::{EmptyMutation, EmptySubscription};
use minio::s3::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use once_cell::sync::Lazy;
use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Header, Status};
use rocket::{catch, catchers, routes, Request};
use rocket::response::content::RawXml;
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;
use crate::graphql::{Query, Schema};
use crate::nnid::people::S3ClientState;

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

    let graph_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&act_database_url).await
        .expect("unable to create pool");

    pub static S3_URL_STRING: Lazy<Box<str>> = Lazy::new(||
        env::var("S3_URL").expect("S3_URL not specified").into_boxed_str()
    );

    pub static S3_URL: Lazy<BaseUrl> = Lazy::new(||
        S3_URL_STRING.parse().unwrap()
    );

    pub static S3_USER: Lazy<Box<str>> = Lazy::new(||
        env::var("S3_USER").expect("S3_USER not specified").into_boxed_str()
    );

    pub static S3_PASSWD: Lazy<Box<str>> = Lazy::new(||
        env::var("S3_PASSWD").expect("S3_PASSWD not specified").into_boxed_str()
    );

    let s3_client = ClientBuilder::new(S3_URL.clone())
        .provider(Some(Box::new(StaticProvider::new(&S3_USER, &S3_PASSWD, None))))
        .build()
        .expect("failed to create s3 client");

    rocket::build()
        .manage(pool)
        .manage(S3ClientState {
            client: Arc::new(s3_client),
        })
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
            nnid::people::get_own_device,
            nnid::people::change_mii,
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
        .register("/", catchers![not_found])
}
