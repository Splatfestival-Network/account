use rocket::{post, State};
use rocket::form::Form;
use serde::Deserialize;
use serde::Serialize;
use crate::Pool;
use crate::account::account::{User, read_bearer_auth_token};
use crate::nnid::oauth::generate_token::{create_token, token_type::AUTH_TOKEN, token_type::AUTH_REFRESH_TOKEN};
use crate::error::{Error, Errors};
use rocket::serde::json::Json;

#[derive(Deserialize)]
pub struct LoginRequest {
    grant_type: String,
    username: Option<String>,
    password: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
}

const INVALID_GRANT_TYPE_ERROR: Errors<'static> = Errors {
    error: &[Error {
        code: "0100",
        message: "Invalid grant type",
    }]
};

const ACCOUNT_ID_OR_PASSWORD_ERRORS: Errors<'static> = Errors {
    error: &[Error {
        code: "0106",
        message: "Invalid account ID or password",
    }]
};

const INVALID_REFRESH_TOKEN_ERRORS: Errors<'static> = Errors {
    error: &[Error {
        code: "0107",
        message: "Invalid or missing refresh token",
    }]
};

#[post("/v1/login", data = "<form_data>")]
pub async fn login(pool: &State<Pool>, form_data: Json<LoginRequest>) -> Result<Json<LoginResponse>, Option<Errors<'static>>> {
    let pool = pool.inner();
    let grant_type = form_data.grant_type.as_str();

    if grant_type != "password" && grant_type != "refresh_token" {
        return Err(Some(INVALID_GRANT_TYPE_ERROR));
    }

    let user: User;

    if grant_type == "password" {
        let username = form_data.username.as_ref().ok_or(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS))?;
        let password = form_data.password.as_ref().ok_or(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS))?;

        user = User::get_by_username(username, pool)
            .await
            .ok_or(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS))?;

        if !user.verify_cleartext_password(password).is_some_and(|v| v) {
            return Err(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS));
        }
    } else {
        let refresh_token = form_data.refresh_token.as_ref().ok_or(Some(INVALID_REFRESH_TOKEN_ERRORS))?;

        user = read_bearer_auth_token(pool, refresh_token)
            .await
            .ok_or(Some(INVALID_REFRESH_TOKEN_ERRORS))?;
    }

    if user.account_level < 0 {
        return Err(Some(ACCOUNT_ID_OR_PASSWORD_ERRORS));
    }

    let access_token = create_token(pool, user.pid, AUTH_TOKEN, None).await;
    let refresh_token = create_token(pool, user.pid, AUTH_REFRESH_TOKEN, None).await;

    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
    }))
}
