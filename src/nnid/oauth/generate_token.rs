use rocket::{post, FromForm, State};
use rocket::form::Form;
use serde::Deserialize;
use crate::account::account::User;
use crate::error::{Error, Errors};
use crate::Pool;

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

#[post("/v1/api/oauth20/access_token/generate", data="<data>")]
pub async fn generate_token(pool: &State<Pool>, data: Form<TokenRequestData<'_>>) -> Result<(), Option<Errors<'static>>>{
    let pool = pool.inner();

    let user = User::get_by_username(data.user_id, pool).await
        .ok_or(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS))?;

    if !user.verify_hashed_password(&data.password).is_some_and(|v| v){
        return Err(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS));
    }

    if user.account_level < 0{
        return Err(Some(ACCOUNT_BANNED_ERRORS));
    }
    
     

    Ok(())
}