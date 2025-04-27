use rocket::{State, post, FromForm, put};
use crate::Pool;
use rocket::form::Form;
use crate::email::send_verification_email;
use crate::error::{Error, Errors};
use chrono::Utc;

const BAD_CODE_ERROR: Errors = Errors{
    error: &[
        Error{
            code: "0116",
            message: "Missing or invalid verification code"
        }
    ]
};

#[derive(FromForm)]
pub struct ValidateEmailInput{
    email: String,
}
#[post("/v1/api/support/validate/email", data="<data>")]
pub async fn validate(data: Form<ValidateEmailInput>){

}

#[put("/v1/api/support/email_confirmation/<pid>/<code>")]
pub async fn verify_email(database: &State<Pool>, pid: i32, code: i32) -> Result<(), Errors<'static>> {
    let db = database.inner();

    let result = sqlx::query!(
        "SELECT verification_code FROM users WHERE pid = $1",
        pid
    )
        .fetch_optional(db)
        .await;

    let Ok(Some(record)) = result else {
        return Err(BAD_CODE_ERROR);
    };

    if let Some(stored_code) = record.verification_code {
        if stored_code == code {
            // Set email_verified_since to NOW
            let now = Utc::now().naive_utc();
            let update_result = sqlx::query!(
                "UPDATE users SET email_verified_since = $1 WHERE pid = $2",
                now,
                pid
            )
                .execute(db)
                .await;

            if update_result.is_err() {
                return Err(BAD_CODE_ERROR); // fallback in case the update fails
            }

            return Ok(()); // Success
        }
    }

    Err(BAD_CODE_ERROR)
}
