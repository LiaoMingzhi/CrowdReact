// 用户相关的处理逻辑
// src/handlers/user_handler.rs
use crate::services::user_service::UserService;
use actix_web::{http::header, web, HttpRequest, HttpResponse, Responder, error, Result as ActixResult, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use log::{error, info, warn};
use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, DbErr};
use crate::entities::users::{self, Entity as Users};
use crate::utils::auth::{get_user_id_from_token, hash_password, verify_password};
use chrono::Utc;
use crate::services::auth_service::AuthService;
use crate::utils::error::ServiceError;
use urlencoding;
use crate::services::agent_service::AgentService;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    email: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn register(
    user_service: web::Data<Arc<UserService>>,
    agent_service: web::Data<Arc<AgentService>>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    info!("Received registration request for user: {}", req.username);
    
    // 检查用户名是否已存在
    if let Ok(Some(_)) = user_service.get_user_by_username(&req.username).await {
        warn!("Username already exists: {}", req.username);
        return HttpResponse::Conflict().json(json!({
            "status": "exists",
            "message": "User already exists",
            "error_code": "USERNAME_EXISTS"
        }));
    }

    // Start database transaction
    let user_service = user_service.into_inner();
    let agent_service = agent_service.into_inner();
    let username = req.username.clone();
    let email = req.email.clone();
    let password = req.password.clone();

    let result = web::block(move || {
        async move {
            let user = user_service.create_user(
                username.clone(),
                email,
                password
            ).await?;

            let agent = agent_service.create_agent(
                user.username.clone()
            ).await?;

            Ok::<_, DbErr>((user, agent))
        }
    }).await;

    match result {
        Ok(future_result) => {
            match future_result.await {
                Ok((user, agent)) => {
                    info!("User and agent created successfully: {}", req.username);
                    HttpResponse::Created().json(json!({
                        "status": "success",
                        "message": "Registration successful",
                        "data": {
                            "user_id": user.id,
                            "username": user.username,
                            "agent_level": "not_agent"
                        }
                    }))
                },
                Err(e) => {
                    error!("Registration failed for user {}: {:?}", req.username, e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to create user and agent records",
                        "error_code": "INTERNAL_ERROR"
                    }))
                }
            }
        },
        Err(e) => {
            error!("Block execution failed: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Internal server error",
                "error_code": "BLOCK_EXECUTION_FAILED"
            }))
        }
    }
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    credentials: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    info!("Received login request for user: {}", credentials.username);
    
    let token = auth_service
        .login(&credentials.username, &credentials.password)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "token": token
        }
    })))
}

pub async fn get_profile(
    user_service: web::Data<UserService>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let token = auth_header.replace("Bearer ", "");

    match user_service.get_profile_from_token(&token).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
    }
}

pub async fn update_profile(
    user_service: web::Data<UserService>,
    req: actix_web::HttpRequest,
    update_req: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let token = auth_header.replace("Bearer ", "");

    match user_service
        .update_profile(
            &token,
            update_req.email.clone(),
            update_req.password.clone(),
        )
        .await
    {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub async fn get_balance(
    user_service: web::Data<UserService>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let token = auth_header.replace("Bearer ", "");

    match user_service.get_balance(&token).await {
        Ok(balance) => HttpResponse::Ok().json(json!({ "balance": balance })),
        Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
    }
}

pub async fn get_all_users(user_service: web::Data<UserService>) -> HttpResponse {
    match user_service.get_all_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn get_user_by_id(
    user_service: web::Data<UserService>,
    user_id: web::Path<i32>,
) -> HttpResponse {
    match user_service.get_user_by_id(user_id.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().body(err.to_string()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn create_user(
    user_service: web::Data<UserService>,
    req: web::Json<CreateUserRequest>,
) -> HttpResponse {
    match user_service
        .create_user(
            req.username.to_lowercase(),
            req.email.clone(),
            req.password.clone(),
        )
        .await
    {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::BadRequest().body(err.to_string()),
    }
}

pub async fn change_password(
    req: HttpRequest,
    password_data: web::Json<ChangePasswordRequest>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, ServiceError> {
    info!("Starting password change process");
    
    let user_id = match get_user_id_from_token(&req) {
        Ok(id) => {
            info!("Successfully validated token for user_id: {}", id);
            id
        },
        Err(e) => {
            error!("Token validation failed: {}", e);
            return Ok(HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": "Invalid token"
            })));
        }
    };
    
    // 获取用户
    let user = Users::find_by_id(user_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(ServiceError::from)?
        .ok_or_else(|| {
            error!("User not found: {}", user_id);
            ServiceError::NotFound
        })?;

    info!("Found user: {}", user.username);
    info!("Verifying password for user: {}", user.username);
    
    // 验证当前密码
    match verify_password(&password_data.current_password, &user.password_hash) {
        Ok(is_valid) => {
            if !is_valid {
                info!("Password verification failed for user {}", user.username);
                return Ok(HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": "Current password is incorrect"
                })));
            }
            info!("Password verified successfully for user {}", user.username);
        },
        Err(e) => {
            error!("Password verification error: {:?}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Password verification failed"
            })));
        }
    }

    info!("Debug - Received password hash: {}", password_data.current_password);
    info!("Debug - Stored password hash: {}", user.password_hash);

    // 更新密码
    let mut user: users::ActiveModel = user.into();
    user.password_hash = Set(hash_password(&password_data.new_password)
        .map_err(|_| ServiceError::BadRequest("Invalid password format".to_string()))?);

    let current_time = Utc::now();
    user.updated_at = Set(current_time);

    match user.update(db.as_ref().as_ref()).await {
        Ok(_) => {
            info!("Password updated successfully");
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Password updated successfully"
            })))
        },
        Err(e) => {
            error!("Failed to update password: {:?}", e);
            Err(ServiceError::DbError(e))
        }
    }
}

pub async fn check_user(
    user_service: web::Data<Arc<UserService>>,
    address: web::Path<String>,
) -> impl Responder {
    match user_service.get_user_by_username(&address).await {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({
            "exists": true,
            "status": "success"
        })),
        Ok(None) => HttpResponse::Ok().json(json!({
            "exists": false,
            "status": "success"
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to check user existence"
        }))
    }
}
