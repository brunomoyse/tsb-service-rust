// src/routes/mod.rs

//pub mod order_routes;
pub mod product_routes;
pub mod user_routes;
pub mod head_routes;

//pub use self::order_routes::configure_organization_routes;
pub use self::product_routes::configure_product_routes;
pub use self::user_routes::configure_user_routes;
pub use self::head_routes::configure_head_routes;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    //configure_order_routes(cfg);
    configure_product_routes(cfg);
    configure_user_routes(cfg);
    configure_head_routes(cfg);

}