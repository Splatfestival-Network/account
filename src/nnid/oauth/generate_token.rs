use rocket::{post, FromForm, State};
use rocket::form::Form;
use serde::{Deserialize, Serialize};
use crate::account::account::User;
use crate::error::{Error, Errors};
use crate::nnid::oauth::TokenData;
use crate::Pool;
use crate::xml::Xml;

const ACCOUNT_ID_OR_PASSWORD_ERRORS: Errors = Errors{
    error: &[
        Error{
            code: "0106",
            message: "Invalid account ID or password"
        }
    ]
};

const ACCOUNT_BANNED_ERRORS: Errors = Errors{
    error: &[
        Error{
            code: "0122",
            message: "Device has been banned by game server"
        }
    ]
};

#[derive(FromForm)]
pub struct TokenRequestData<'a>{
    grant_type: &'a str,
    user_id: &'a str,
    password: &'a str,
    password_type: &'a str,
}

#[derive(Serialize)]
pub struct TokenReturnData {
    token: String,
    refresh_token: String,
    expires_in: i32
}

impl TokenReturnData {
    async fn create_token(pid: i32, pool: &Pool, is_refresh_token: bool) -> (i64, i32){
        let token_type = if is_refresh_token{
            0x0
        } else {
            0x1
        };
        let data = sqlx::query!(
            "insert into tokens (token_type, pid)
            values ($1, $2) returning token_id, random",
            token_type, pid
        )
            .fetch_one(pool)
            .await.unwrap();

        (data.token_id, data.random)
    }
    async fn create_regular_token(pid: i32, pool: &Pool) -> (i64, i32){
        Self::create_token(pid, pool, false).await
    }

    async fn create_refresh_token(pid: i32, pool: &Pool) -> (i64, i32){
        Self::create_token(pid, pool, true).await
    }

    async fn new(pid: i32, pool: &Pool) -> Self{
        let (token_id, random) = Self::create_regular_token(pid, pool).await;

        let token = TokenData {
            token_id,
            random,
            pid
        };

        let token = token.encode().to_string();

        let (token_id, random) = Self::create_refresh_token(pid, pool).await;

        let refresh_token = TokenData {
            token_id,
            random,
            pid
        };

        let refresh_token = refresh_token.encode().to_string();

        Self{
            token,
            refresh_token,
            expires_in: 3600
        }
    }
}

#[derive(Serialize)]
#[serde(rename="OAuth20")]
pub struct TokenRequestReturnData{
    access_token: TokenReturnData
}

#[post("/v1/api/oauth20/access_token/generate", data="<data>")]
pub async fn generate_token(pool: &State<Pool>, data: Form<TokenRequestData<'_>>) -> Result<Xml<TokenRequestReturnData>, Option<Errors<'static>>>{
    let pool = pool.inner();

    let user = User::get_by_username(data.user_id, pool).await
        .ok_or(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS))?;

    if !user.verify_hashed_password(&data.password).is_some_and(|v| v){
        return Err(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS));
    }

    if user.account_level < 0{
        return Err(Some(ACCOUNT_BANNED_ERRORS));
    }
    
    let access_token = TokenReturnData::new(user.pid, pool).await;

    Ok(Xml(TokenRequestReturnData{
        access_token
    }))
}