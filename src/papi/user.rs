use std::env;
use once_cell::sync::Lazy;
use rocket::{get};
use crate::account::account::{Auth};
use rocket::serde::json::Json;

pub static CDN_URL: Lazy<Box<str>> = Lazy::new(||
    env::var("CDN_URL").expect("CDN_URL not specified").into_boxed_str()
);

#[derive(serde::Serialize)]
struct EmailInfo {
    address: String,
}

#[derive(serde::Serialize)]
struct TimezoneInfo {
    name: String,
}

#[derive(serde::Serialize)]
struct MiiInfo {
    data: String,
    name: String,
    image_url: String,
}

#[derive(serde::Serialize)]
struct FlagsInfo {
    marketing: bool,
}

#[derive(serde::Serialize)]
struct ConnectionsInfo {
    discord: DiscordInfo,
    stripe: StripeInfo,
}

#[derive(serde::Serialize)]
struct DiscordInfo {
    id: Option<String>,
}

#[derive(serde::Serialize)]
struct StripeInfo {
    tier_name: Option<String>,
    tier_level: Option<i32>,
}

#[derive(serde::Serialize)]
struct UserInfoResponse {
    deleted: bool,
    access_level: i32,
    server_access_level: String,
    pid: i32,
    creation_date: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    username: String,
    birthdate: chrono::NaiveDate,
    gender: String,
    country: String,
    email: EmailInfo,
    timezone: TimezoneInfo,
    mii: MiiInfo,
    flags: FlagsInfo,
    connections: ConnectionsInfo,
}

#[get("/v1/user")]
pub async fn get_user(auth: Auth<false>) -> Json<UserInfoResponse> {
    let user = auth.0;

    Json(UserInfoResponse {
        deleted: false,
        access_level: user.account_level,
        server_access_level: "test".to_string(),
        pid: user.pid,
        creation_date: user.creation_date,
        updated: user.updated,
        username: user.username.clone(),
        birthdate: user.birthdate,
        gender: user.gender.clone(),
        country: user.country.clone(),
        email: EmailInfo {
            address: user.email.clone(),
        },
        timezone: TimezoneInfo {
            name: user.timezone.clone(),
        },
        mii: MiiInfo {
            data: user.mii_data.clone(),
            name: {
                let cleaned = user.mii_data.replace('\n', "").replace('\r', "");
                mii::MiiData::read(&cleaned)
                    .map(|v| v.name)
                    .unwrap_or_else(|| "INVALID".to_string())
            },
            image_url: format!("https://{}/mii/{}/normal_face.png", &CDN_URL.to_string(), user.pid),
        },
        flags: FlagsInfo {
            marketing: user.marketing_allowed,
        },
        connections: ConnectionsInfo {
            discord: DiscordInfo {
                id: None,
            },
            stripe: StripeInfo {
                tier_name: None,
                tier_level: None,
            },
        },
    })
}
