// src/models/user.rs

use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::schema::users;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

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
    pub remember_token: Option<String>,
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


