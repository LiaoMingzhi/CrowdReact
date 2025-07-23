use actix_web::web;
use crate::handlers::user_handler;

pub mod user_route;

pub mod bet_route;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/user")
                    .route("/change-password", web::post().to(user_handler::change_password))
            )
    );
}
