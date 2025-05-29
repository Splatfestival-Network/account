use std::net::Ipv4Addr;
use rocket::{get, State};
use serde::Serialize;
use sqlx::types::ipnetwork::IpNetwork::V4;
use crate::account::account::Auth;
use crate::error::{Error, Errors};
use crate::nnid::oauth::generate_token::{create_token};
use crate::nnid::oauth::generate_token::token_type::NEX_TOKEN;
use crate::Pool;
use crate::xml::Xml;

const NO_IPV4_ERROR: Errors = Errors{
    error: &[
        Error{
            code: "1022",
            message: "Server is not a valid IPv4 address"
        }
    ]
};

const NO_SERVER_ERROR: Errors = Errors{
    error: &[
        Error{
            code: "1021",
            message: "The requested game server was not found"
        }
    ]
};


#[derive(Serialize)]
#[serde(rename = "nex_token")]
pub struct NexToken{
    host: Ipv4Addr,
    nex_password: String,
    pid: i32,
    port: u16,
    token: String
}

#[derive(Serialize)]
#[serde(rename = "service_token")]
pub struct ServiceToken{
    token: String
}

#[get("/v1/api/provider/service_token/@me")]
pub async fn get_service_token(pool: &State<Pool>, auth: Auth<true>) -> Result<Xml<ServiceToken>, Option<Errors<'static>>>{
    // just gonna put this here as a side note for the future:
    // we could also be using key derivation to derive the nex token as if it were a key
    // that way we could reduce the data the database needs to store and also reduce the transfer
    // cost of sending an entire row from the user table (which is required for the auth code unless
    // we change the way we read in data to essentially having the user object be a proxy for its
    // table row)

    let pool = pool.inner();

    let token = create_token(pool, auth.pid, NEX_TOKEN, None).await;



    Ok(
        Xml(
            ServiceToken{
                token
            }
        )
    )
}

#[get("/v1/api/provider/nex_token/@me?<game_server_id>")]
pub async fn get_nex_token(pool: &State<Pool>, auth: Auth<true>, game_server_id: &str) -> Result<Xml<NexToken>, Option<Errors<'static>>>{
    // just gonna put this here as a side note for the future:
    // we could also be using key derivation to derive the nex token as if it were a key
    // that way we could reduce the data the database needs to store and also reduce the transfer
    // cost of sending an entire row from the user table (which is required for the auth code unless
    // we change the way we read in data to essentially having the user object be a proxy for its
    // table row)

    let pool = pool.inner();

    let server = sqlx::query!(
    "select address, port from nex_servers where game_server_id = $1",
    game_server_id
    )
        .fetch_optional(pool)
        .await
        .expect("database error"); // only crash on db failure (not missing row)

    let server = match server {
        Some(server) => server,
        None => return Err(Some(NO_SERVER_ERROR)), // or custom error
    };


    let token = create_token(pool, auth.pid, NEX_TOKEN, None).await;

    let V4(host) = server.address else {
        return Err(Some(NO_IPV4_ERROR));
    };

    let host = host.ip();



    Ok(
        Xml(
            NexToken{
                host,
                port: server.port as u16,
                nex_password: auth.nex_password.clone(),
                pid: auth.pid,
                token
            }
        )
    )
}