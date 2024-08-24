use crate::controllers::product_controller;
use actix_web::web;
//use crate::middlewares::token_validation; // Import your middleware

pub fn configure_product_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/products")
            //.wrap(token_validation::Authentication) 
            .route(web::get().to(product_controller::translated_products_handler)),
    );
}