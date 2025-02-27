use rocket::{post, FromForm};
use rocket::form::Form;

#[derive(FromForm)]
struct ValidateEmailInput{
    email: String,
}
#[post("/v1/api/support/validate/email", data="<data>")]
pub fn validate(data: Form<ValidateEmailInput>){

}