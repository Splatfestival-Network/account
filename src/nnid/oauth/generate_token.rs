use rocket::{post, FromForm, State};
use rocket::form::Form;
use serde::{Deserialize, Serialize};
use crate::account::account::User;
use crate::error::{Error, Errors};
use crate::nnid::oauth::generate_token::token_type::{AUTH_REFRESH_TOKEN, AUTH_TOKEN};
use crate::nnid::oauth::TokenData;
use crate::Pool;
use crate::xml::Xml;

pub mod token_type{
    pub const AUTH_REFRESH_TOKEN: i32 = 1;
    pub const AUTH_TOKEN: i32 = 0;
    pub const NEX_TOKEN: i32 = 2;
}

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

pub async fn create_token(pool: &Pool, pid: i32, token_type: i32, title_id: Option<&str>) -> String{
    let data = sqlx::query!(
            "insert into tokens (token_type, pid, title_id)
            values ($1, $2, $3) returning token_id, random",
            token_type, pid, title_id
        )
        .fetch_one(pool)
        .await.unwrap();

    let token_id = data.token_id;
    let random = data.random;

    let token = TokenData {
        token_id,
        random,
        pid
    };

    token.encode().to_string()
}


impl TokenReturnData {
    async fn new(pid: i32, pool: &Pool) -> Self{
        let token = create_token(pool, pid, AUTH_TOKEN, None).await;

        let refresh_token = create_token(pool, pid, AUTH_REFRESH_TOKEN, None).await;

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