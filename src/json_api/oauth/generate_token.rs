use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use rocket::{get, State};
use rocket::serde::json::Json;
use serde::Serialize;
use crate::account::account::Auth;
use crate::nnid::oauth::generate_token::create_token;
use crate::nnid::oauth::generate_token::token_type::AUTH_TOKEN;
use crate::Pool;

#[derive(Serialize)]
struct TokenData{
    token: String,
    expiry: NaiveDateTime
}

#[get("/api/v2/oauth2/generate_token")]
pub async fn generate_token(pool: &State<Pool>, auth: Auth<false>) -> Json<TokenData>{
    let pool = pool.inner();



    Json(
        TokenData{
            expiry: Utc::now().naive_utc() + Duration::hours(1),
            token: create_token(pool, auth.pid, AUTH_TOKEN, None).await,

        }
    )
}