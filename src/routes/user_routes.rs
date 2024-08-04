use crate::controllers::user_controller;
use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/users").route(web::get().to(user_controller::get_all_users)));
        //.service(
        //    web::resource("/users/{user_id}/training-completed")
        //        .route(web::post().to(user_controller::set_user_training_completed)),
        //);
}