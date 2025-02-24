use diesel::{Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::user)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub hashed_pw: String
}