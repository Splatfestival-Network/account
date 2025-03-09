use rocket::{get, State};
use serde::Serialize;
use crate::Pool;
use crate::xml::Xml;

#[derive(Serialize)]
#[serde(rename = "mapped_id")]
struct MappedId {
    in_id: String,
    out_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename = "mapped_ids")]
struct MappedIds {
    mapped_id: Vec<MappedId>,
}

struct UserIdAndName {
    pid: i32,
    username: String,
}

#[get("/v1/api/admin/mapped_ids?<input_type>&<output_type>&<input>")]
pub async fn mapped_ids(pool: &State<Pool>, input_type: String, output_type: String, input: String) -> Option<Xml<MappedIds>> {
    let pool = pool.inner();

    let is_input_pid = input_type == "pid";
    let is_output_pid = output_type == "pid";

    let mut outputs = Vec::new();

    for input in input.split(',') {
        if input == ""{
            continue;
        }
        let Some(user) =
            (if is_input_pid {
                let id: i32 = input.parse().ok()?;

                sqlx::query_as!(
                    UserIdAndName,
                    "select pid, username from users where pid = $1",
                    id
                ).fetch_one(pool)
                    .await.ok()
            } else {
                sqlx::query_as!(
                    UserIdAndName,
                    "select pid, username from users where username = $1",
                    input
                ).fetch_one(pool)
                    .await.ok()
            }) else {
            outputs.push(MappedId{
                in_id: input.to_string(),
                out_id: None,
            });

            continue
        };


        if is_output_pid{
            outputs.push(MappedId{
                in_id: input.to_string(),
                out_id: Some(user.pid.to_string()),
            })
        } else {
            outputs.push(MappedId{
                in_id: input.to_string(),
                out_id: Some(user.username),
            })
        }

    }

    Some(Xml(
        MappedIds{
            mapped_id: outputs
        }
    ))
}