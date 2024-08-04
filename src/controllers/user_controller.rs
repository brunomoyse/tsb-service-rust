use actix_web::{web, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use crate::models::user::User;
use serde_json::json;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn get_all_users(pool: web::Data<DbPool>) -> HttpResponse {
    let connection_result = pool.get();

    let mut connection = match connection_result {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Error getting DB connection from pool"})),
    };

    let result = web::block(move || User::find_all(&mut *connection))
        .await;

    match result {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Server error"}))
    }
}
