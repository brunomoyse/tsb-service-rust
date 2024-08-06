extern crate diesel;

pub mod schema;
pub mod models;
pub mod controllers;
pub mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http::header};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use dotenv::dotenv;
use std::env;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn create_database_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = create_database_pool().await;
    let app_url: String = env::var("API_URL").expect("API_URL must be set");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db_pool.clone()))
            .configure(routes::configure)
    })
        .bind(app_url)?
        .run()
        .await
}
