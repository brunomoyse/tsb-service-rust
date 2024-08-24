use actix_web::{web, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use diesel::{PgConnection, QueryDsl};
use diesel::RunQueryDsl;
use crate::schema::users::dsl::*;  // Import the DSL for the users table
use diesel::prelude::*; use diesel::insert_into;
use crate::models::user::{User, UserForm, NewUser, UserConnectionForm, Claims, get_secret_key};
use serde::Deserialize;

use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::Utc;  // For managing expiration times

use serde_json::json;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

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

pub async fn sign_in(pool: web::Data<DbPool>, connection_form: web::Json<UserConnectionForm>) -> HttpResponse {
    // Get a database connection from the pool
    let connection_result = pool.get();

    let mut connection = match connection_result {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Error getting DB connection from pool"})),
    };

    // Find the user by email
    let found_user_result = users
        .filter(email.eq(&connection_form.email))
        .first::<User>(&mut connection);

    let found_user = match found_user_result {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"})),
    };

    // Parse the stored password hash
    let parsed_hash = match PasswordHash::new(&found_user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Invalid password hash format"})),
    };

    // Verify the password using Argon2
    let password_verification = Argon2::default().verify_password(
        connection_form.password.as_bytes(),
        &parsed_hash,
    );

    if password_verification.is_err() {
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"}));
    }

    // Create JWT claims
    let access_token_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))  // Access token valid for 15 minutes
        .expect("valid timestamp")
        .timestamp() as usize;

    let refresh_token_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(7))  // Refresh token valid for 7 days
        .expect("valid timestamp")
        .timestamp() as usize;

    let access_claims = Claims {
        sub: found_user.email.clone(),  // Use the user's email as the subject
        exp: access_token_expiration,
    };

    let refresh_claims = Claims {
        sub: found_user.email.clone(),  // Use the user's email as the subject
        exp: refresh_token_expiration,
    };

    // Encode the tokens
    let secret_key = get_secret_key();  // Retrieve the secret key from the environment

    let access_token = match encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Access token creation error"})),
    };

    let refresh_token = match encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret_key.as_ref()),  // Replace with your secret key
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Refresh token creation error"})),
    };

    // Return the tokens in the response
    HttpResponse::Ok().json(json!({
        "message": "Sign in successful",
        "access_token": access_token,
        "refresh_token": refresh_token
    }))
}

pub async fn refresh_token(req: web::Json<RefreshTokenRequest>) -> HttpResponse {
    let refresh_token = &req.refresh_token;
    let secret_key = get_secret_key();

    let token_data = match jsonwebtoken::decode::<Claims>(
        &refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(secret_key.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"error": "Invalid refresh token"})),
    };

    let new_access_token_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))  // New access token valid for 15 minutes
        .expect("valid timestamp")
        .timestamp() as usize;

    let new_access_claims = Claims {
        sub: token_data.claims.sub,  // Keep the same subject
        exp: new_access_token_expiration,
    };

    let new_access_token = match encode(
        &Header::default(),
        &new_access_claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "New access token creation error"})),
    };

    HttpResponse::Ok().json(json!({
        "access_token": new_access_token
    }))
}