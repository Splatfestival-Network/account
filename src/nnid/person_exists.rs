use rocket::{get, State};
use sqlx::Row;
use crate::error::{Error, Errors};
use crate::Pool;
use crate::xml::Xml;


#[get("/v1/api/people/<username>")]
pub async fn person_exists(database: &State<Pool>, username: &str) -> Result<(), Errors<'static>>{
    let database = database.inner();

    let exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 ) as exists",
        username
    ).fetch_one(database)
        .await
        .ok()
        .map(|v| v.exists)
        .flatten()
        .unwrap_or(true);

    if exists {
        Err(
            Errors{
                error: &[
                    Error{
                        code: "0100",
                        message: "Account ID already exists"
                    }
                ],
            }
        )
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test{
    use crate::error::{Error, Errors};
    use crate::xml::serialize_with_version;

    #[test]
    fn test(){
        let val = Errors{
            error: &[
                Error{
                    code: "0100",
                    message: "Account ID already exists"
                }
            ],
        };

        let enc = serialize_with_version(&val).unwrap();

        assert_eq!(
            enc.as_ref(),
            "<?xml version=\"1.0\"?><errors><error><code>0100</code><message>Account ID already exists</message></error></errors>"
        )
    }
}