use std::net::Ipv4Addr;
use std::str::FromStr;
use rocket::get;
use serde::Serialize;
use crate::xml::Xml;

#[derive(Serialize)]
#[serde(rename = "nex_token")]
struct NexToken{
    host: Ipv4Addr,
    nex_password: String,
    pid: i32,
    port: u16,
    token: String
}

#[get("/v1/api/provider/nex_token/@me?<game_server_id>")]
pub async fn get_nex_token(game_server_id: String) -> Option<Xml<NexToken>>{
    None
}