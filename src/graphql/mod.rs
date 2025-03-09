use std::fmt::Display;
use chrono::NaiveDateTime;
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode, ScalarValue};
use rocket::response::content::RawHtml;
use rocket::State;
use crate::account::account::{read_basic_auth_token, read_bearer_auth_token};
use crate::nnid::oauth::TokenData;
use crate::Pool;



pub type Schema = RootNode<
    'static,
    Query,
    EmptyMutation<Context>,
    EmptySubscription<Context>
>;


pub struct Context(pub Pool);
impl juniper::Context for Context{}

#[derive(GraphQLObject)]
#[graphql(description = "Data inside of a token")]
struct TokenInfo {
    pid: i32,
    expire_date: NaiveDateTime,
    title_id: Option<String>
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
                fetch_one(&context.0).await.ok()?;

        Some(TokenInfo{
            pid: data.pid,
            expire_date: token_info.expires,
            title_id: token_info.title_id,
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

#[rocket::get("/graphiql")]
pub fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}


#[rocket::get("/playground")]
pub fn playground() -> RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
}

#[rocket::get("/graphql?<request..>")]
pub async fn get_graphql(
    db: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}

#[rocket::post("/graphql", data = "<request>")]
pub async fn post_graphql(
    db: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}