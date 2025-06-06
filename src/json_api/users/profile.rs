use rocket::serde::json::Json;
use rocket::{get, State};
use crate::account::account::Auth;
use crate::nnid::people::{build_profile, GetOwnProfileData};
use crate::Pool;

#[get("/api/v2/users/@me/profile")]
pub async fn get_own_profile(pool: &State<Pool>, auth: Auth<true>) -> Json<GetOwnProfileData> {
    Json(build_profile(auth.into()))
}