// src/models/user.rs

use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::schema::users;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use std::env;

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub email: String,
    pub email_verified_at: Option<NaiveDateTime>,
    pub password: String,
    pub salt: String,
    pub remember_token: Option<String>,
}

#[derive(Deserialize)]
pub struct UserForm {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub salt: &'a str,
}

#[derive(Deserialize)]

pub struct UserConnectionForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // subject (the user's email in this case)
    pub exp: usize,   // expiration time as a UNIX timestamp
}

impl User {
    pub fn find_all(connection: &mut PgConnection) -> Result<Vec<User>, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        let results = users
            .order(name.asc())
            .load::<User>(connection)?;

        Ok(results)
    }
}

pub fn get_secret_key() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}



