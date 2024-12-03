use crate::controllers::order_controller;
use actix_web::web;

pub fn configure_order_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/orders").route(web::post().to(order_controller::create_order_handler).wrap(auth_middleware)));

}