use rocket::{get, State};
use rocket::serde::json::Json;
use serde::de::IntoDeserializer;
use sqlx::query;
use crate::account::account::Auth;
use crate::nnid::people::{build_profile, GetOwnProfileData};
use crate::Pool;



#[get("/api/v2/users/<pid>/mii")]
pub async fn get_mii_data_by_pid(pool: &State<Pool>, pid: i32) -> Option<Json<String>> {
    let pool = pool.inner();

    let mii_data = query!("SELECT mii_data from users where pid = $1", pid).fetch_one(pool).await;

    let Ok(mii_data) = mii_data else {
        return None;
    };

    Some(Json(mii_data.mii_data))
}