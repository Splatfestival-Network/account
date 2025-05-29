use chrono::NaiveDateTime;
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use rocket::response::content::RawHtml;
use rocket::State;
use rocket::request::{FromRequest, Outcome, Request};
use std::env;
use once_cell::sync::Lazy;
// use crate::account::account::{read_basic_auth_token, read_bearer_auth_token};
use crate::nnid::oauth::TokenData;
use crate::Pool;

pub static API_KEY: Lazy<String> = Lazy::new(|| {
    env::var("GRAPHQL_API_KEY").expect("GRAPHQL_API_KEY not set")
});

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Context {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = req.rocket().state::<Pool>().cloned().unwrap(); // assume Pool is managed as state

        // Grab API key from header
        let api_key = req.headers().get_one("X-API-Key").map(|s| s.to_string());

        Outcome::Success(Context {
            pool,
            api_key,
        })
    }
}

pub type Schema = RootNode<
    'static,
    Query,
    EmptyMutation<Context>,
    EmptySubscription<Context>
>;


pub struct Context {
    pub pool: Pool,
    pub api_key: Option<String>,
}
impl juniper::Context for Context {}


#[derive(GraphQLObject)]
#[graphql(description = "Data inside of a token")]
struct TokenInfo {
    pid: i32,
    expire_date: NaiveDateTime,
    title_id: Option<String>
}

#[derive(GraphQLObject)]
#[graphql(description = "User information from a token")]
struct UserInfo {
    username: String,
    account_level: i32,
    nex_password: String,
    mii_data: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "User information from a username")]
pub struct UserInfoWithPId {
    pub username: String,
    pub account_level: i32,
    pub nex_password: String,
    pub mii_data: String,
    pub pid: i32,
}

pub struct Query;

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    fn api_version() -> &'static str {
        "1.0"
    }

    async fn token(
        token_data: String,
        context: &Context,
    ) -> Option<TokenInfo>{
        let data = TokenData::decode(&token_data)?;

        let token_info =
            sqlx::query!(
            "select * from tokens where pid = $1 and token_id = $2 and random = $3",
            data.pid, data.token_id, data.random
        ).
                fetch_one(&context.pool).await.ok()?;

        Some(TokenInfo{
            pid: data.pid,
            expire_date: token_info.expires,
            title_id: token_info.title_id,
        })
    }

    async fn user_from_token(
        token_data: String,
        context: &Context,
    ) -> Option<UserInfo> {
        let data = match TokenData::decode(&token_data) {
            Some(data) => data,
            None => {
                eprintln!("Failed to decode token");
                return None;
            }
        };

        let user = match sqlx::query!(
        "SELECT username, account_level, nex_password, mii_data FROM users WHERE pid = $1",
        data.pid
    )
            .fetch_one(&context.pool)
            .await
            .ok() {
            Some(user) => user,
            None => {
                eprintln!("No user found for PID {}", data.pid);
                return None;
            }
        };
        
        let nex_password = format!("{:a>16}",user.nex_password);

        Some(UserInfo {
            username: user.username,
            account_level: user.account_level,
            nex_password: user.nex_password,
            mii_data: user.mii_data.replace('\n', "").replace('\r', ""),
        })
    }

    async fn user_by_pid(pid: i32, context: &Context) -> Option<UserInfo> {
        if context.api_key.as_deref() != Some(&*API_KEY) {
            eprintln!("Rejected request: invalid API key");
            return None;
        }

        let user = sqlx::query!(
            "SELECT username, account_level, nex_password, mii_data FROM users WHERE pid = $1",
            pid
        )
        .fetch_one(&context.pool)
        .await
        .ok()?;

        let nex_password = format!("{:a>16}",user.nex_password);

        Some(UserInfo {
            username: user.username,
            account_level: user.account_level,
            nex_password: user.nex_password,
            mii_data: user.mii_data,
        })
    }

    async fn user_by_username(username: String, context: &Context) -> Option<UserInfoWithPId> {
        if context.api_key.as_deref() != Some(&*API_KEY) {
            eprintln!("Rejected request: invalid API key");
            return None;
        }

        let user = sqlx::query!(
            "SELECT pid, username, account_level, nex_password, mii_data FROM users WHERE username = $1",
            username,
        )
        .fetch_one(&context.pool)
        .await
        .ok()?;

        Some(UserInfoWithPId {
            username: user.username,
            account_level: user.account_level,
            nex_password: user.nex_password,
            mii_data: user.mii_data,
            pid: user.pid,
        })
    }
}


/*
struct Mutation;


#[graphql_object]
#[graphql(
    context = Context,
    // If we need to use `ScalarValue` parametrization explicitly somewhere
    // in the object definition (like here in `FieldResult`), we could
    // declare an explicit type parameter for that, and specify it.
    scalar = S: ScalarValue + Display,
)]
impl Mutation {
}
*/

// #[rocket::get("/graphiql")]
// pub fn graphiql() -> RawHtml<String> {
//     juniper_rocket::graphiql_source("/graphql", None)
// }
//
//
// #[rocket::get("/playground")]
// pub fn playground() -> RawHtml<String> {
//     juniper_rocket::playground_source("/graphql", None)
// }

#[rocket::get("/graphql?<request..>")]
pub async fn get_graphql(
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
    context: Context
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, &context).await
}

#[rocket::post("/graphql", data = "<request>")]
pub async fn post_graphql(
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
    context: Context
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, &context).await
}
