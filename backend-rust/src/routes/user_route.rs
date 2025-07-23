// 用户相关的路由
// src/routes/user_model
use crate::handlers::user_handler;
use actix_web::{web, Scope};

pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(user_handler::register))
            .route("/login", web::post().to(user_handler::login))
            .route("/check-user/{address}", web::get().to(user_handler::check_user))
    )
    .service(
        web::scope("/user")
            .route("/profile", web::get().to(user_handler::get_profile))
            .route("/update", web::put().to(user_handler::update_profile))
            .route("/change-password", web::post().to(user_handler::change_password))
            .route("/balance", web::get().to(user_handler::get_balance))
            .route("/users", web::get().to(user_handler::get_all_users))
            .route("/users/{id}", web::get().to(user_handler::get_user_by_id))
            .route("/users", web::post().to(user_handler::create_user))
    );
}

pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/register", web::post().to(user_handler::register))
        .route("/login", web::post().to(user_handler::login))
        .route("/profile", web::get().to(user_handler::get_profile))
        .route("/update", web::put().to(user_handler::update_profile))
        .route("/change-password", web::post().to(user_handler::change_password))
        .route("/balance", web::get().to(user_handler::get_balance))
        .route("/users", web::get().to(user_handler::get_all_users))
        .route("/users/{id}", web::get().to(user_handler::get_user_by_id))
        .route("/users", web::post().to(user_handler::create_user))
}


