// src/routes/head_routes.rs

use actix_web::{web, HttpResponse};

pub fn configure_head_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::head().to(|| HttpResponse::Ok()))
    );
}