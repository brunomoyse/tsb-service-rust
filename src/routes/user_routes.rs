use crate::controllers::user_controller;
use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    //cfg.service(web::resource("/users").route(web::get().to(user_controller::get_all_users)));
    cfg.service(web::resource("/sign-up").route(web::post().to(user_controller::sign_up)));
}