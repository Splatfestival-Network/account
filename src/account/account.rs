use std::io::Write;
use std::ops::{Deref, DerefMut};
use argon2::{Algorithm, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytemuck::bytes_of;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use log::{error, warn};
use rocket::http::Status;
use rocket::{async_trait, Request};
use rocket::request::{FromRequest, Outcome};
use sha2::{Digest, Sha256};
use sha2::digest::FixedOutput;
use crate::error::{Error, Errors};
use crate::nnid::oauth::TokenData;
use crate::Pool;

macro_rules! request_try {
    ($expression:expr) => {
        match $expression{
            Ok(v) => v,
            Err(e) => return Outcome::Error((Status::BadRequest, e))
        }
    };
}

const INVALID_TOKEN_ERRORS: Errors<'static> = Errors{
    error: &[
        Error{
            message: "Invalid access token",
            code: "0005"
        }
    ]
};

// optimization note: add token caching
pub struct User {
    pub pid: i32,
    pub username: String,
    pub password: String,
    pub birthdate: NaiveDate,
    pub timezone: String,
    pub email: String,
    pub account_level: i32,
    pub email_verified_since: Option<NaiveDateTime>,
    pub gender: String,
    pub country: String,
    pub language: String,
    pub marketing_allowed: bool,
    pub off_device_allowed: bool,
    pub region: i32,
    pub mii_data: String,
    pub creation_date: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub nex_password: String
}

fn generate_nintendo_hash(pid: i32, text_password: &str) -> String{
    let mut sha = Sha256::new();

    sha.write_all(&bytes_of(&pid)).unwrap();
    sha.write_all(&[0x02, 0x65, 0x43 ,0x46]).unwrap();
    sha.write_all(text_password.as_bytes()).unwrap();

    hex::encode(&sha.finalize()[..])
}

impl User{
    pub async fn get_by_username(name: &str, pool: &Pool) -> Option<Self>{
        sqlx::query_as!(
            Self,
            "SELECT * FROM users WHERE username = $1",
            name
        ).fetch_one(pool)
            .await
            .ok()
    }

    fn generate_nintendo_hash(&self, text_password: &str) -> String{
        generate_nintendo_hash(self.pid, text_password)
    }

    pub fn verify_cleartext_password(&self, cleartext_password: &str) -> Option<bool>{
        let nintendo_hash = self.generate_nintendo_hash(cleartext_password);

        self.verify_hashed_password(cleartext_password)
    }

    pub fn verify_hashed_password(&self, hashed_password: &str) -> Option<bool>{
        bcrypt::verify(hashed_password, &self.password).ok()
    }
}

pub fn generate_password(pid: i32, cleartext_password: &str) -> Option<String>{
    let password = generate_nintendo_hash(pid, cleartext_password);

    bcrypt::hash(password, 10).ok()
}


pub async fn read_basic_auth_token(connection: &Pool, token: &str) -> Option<User> {
    let data = BASE64_STANDARD.decode(&token).ok()?;

    let decoded_basic_token = String::from_utf8(data).ok()?;

    let (login_username, login_password) = decoded_basic_token.split_once(' ')?;



    let mut user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        login_username
    ).fetch_one(connection).await.ok()?;

    let password_valid = user.verify_cleartext_password(&login_password);

    if password_valid == Some(true){
        Some(user)
    } else {
        None
    }
}

pub async fn read_bearer_auth_token(connection: &Pool, token: &str) -> Option<User> {
    let data = TokenData::decode(token)?;

    let token_info =
        sqlx::query!(
            "select * from tokens where pid = $1 and token_id = $2 and random =$3",
            data.pid, data.token_id, data.random
        ).
            fetch_one(connection).await.ok()?;

    if token_info.expires.and_utc() < Utc::now(){
        return None
    }

    let mut user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE pid = $1",
        token_info.pid
    ).fetch_one(connection).await.ok()?;

    Some(user)
}



pub struct Auth<const FORCE_BEARER_AUTH: bool>(pub User);

impl<const FORCE_BEARER_AUTH: bool> AsRef<User> for Auth<FORCE_BEARER_AUTH>{
    fn as_ref(&self) -> &User {
        &self.0
    }
}

impl<const FORCE_BEARER_AUTH: bool> AsMut<User> for Auth<FORCE_BEARER_AUTH>{
    fn as_mut(&mut self) -> &mut User {
        &mut self.0
    }
}

impl<const FORCE_BEARER_AUTH: bool> Deref for Auth<FORCE_BEARER_AUTH>{
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const FORCE_BEARER_AUTH: bool> DerefMut for Auth<FORCE_BEARER_AUTH>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const FORCE_BEARER_AUTH: bool> Into<User> for Auth<FORCE_BEARER_AUTH>{
    fn into(self) -> User {
        self.0
    }
}



#[async_trait]
impl<'r, const FORCE_BEARER_AUTH: bool> FromRequest<'r> for Auth<FORCE_BEARER_AUTH>{
    type Error = Errors<'static>;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool: &Pool = request.rocket().state().unwrap();

        let auth = request_try!(request.headers().get("Authorization").next().ok_or(INVALID_TOKEN_ERRORS));

        let (auth_type, token) = request_try!(auth.split_once(' ').ok_or(INVALID_TOKEN_ERRORS));

        let user = match auth_type{
            "Basic" if !FORCE_BEARER_AUTH => read_basic_auth_token(pool, token).await,
            "Bearer" => read_bearer_auth_token(pool, token).await,
            _ => return Outcome::Error((Status::BadRequest, INVALID_TOKEN_ERRORS)),
        };

        let Some(user) = user else {
            return Outcome::Error((Status::BadRequest, INVALID_TOKEN_ERRORS));
        };

        Outcome::Success(Self(user))
    }
}