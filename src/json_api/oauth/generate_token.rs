use rocket::{get, State};
use crate::account::account::Auth;
use crate::nnid::oauth::generate_token::create_token;
use crate::nnid::oauth::generate_token::token_type::AUTH_TOKEN;
use crate::Pool;

#[get("/api/v2/oauth2/generate_token")]
pub async fn generate_token(pool: &State<Pool>, auth: Auth<false>) -> String{
    let pool = pool.inner();



    create_token(pool, auth.pid, AUTH_TOKEN, None).await
}