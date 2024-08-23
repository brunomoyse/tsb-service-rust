use actix_web::{web, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use crate::schema::users::dsl::*; // Import the `users` table under the `dsl` module
use diesel::insert_into;
use crate::models::user::{User, UserForm, NewUser};

use serde_json::json;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};


type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn get_all_users(pool: web::Data<DbPool>) -> HttpResponse {
    let connection_result = pool.get();

    let mut connection = match connection_result {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Error getting DB connection from pool"})),
    };

    match User::find_all(&mut connection) {
        Ok(all_users) => HttpResponse::Ok().json(all_users),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Error getting users from the database"})),
    }
}

pub async fn sign_up(pool: web::Data<DbPool>, user: web::Json<UserForm>) -> HttpResponse {
    // Get a database connection from the pool
    let connection_result = pool.get();

    let mut connection = match connection_result {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Error getting DB connection from pool"})),
    };

    // Generate a salt for hashing the password
    let generated_salt = SaltString::generate(&mut OsRng);

    // Use Argon2 to hash the password
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(user.password.clone().as_bytes(), &generated_salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Error hashing password"})),
    };

    // Create a new NewUser instance for inserting into the database
    let new_user = NewUser {
        name: &user.name,
        email: &user.email,
        password: &password_hash,
        salt: &generated_salt.to_string(),

    };

    // Insert the new user into the database
    match insert_into(users).values(&new_user).execute(&mut connection) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "User created successfully"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Error inserting user into the database"})),
    }
}