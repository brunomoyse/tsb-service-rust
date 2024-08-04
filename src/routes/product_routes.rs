use crate::controllers::product_controller;
use actix_web::web;

pub fn configure_product_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/products").route(web::get().to(product_controller::translated_products_handler)));
}