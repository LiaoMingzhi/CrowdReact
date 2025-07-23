use actix_web::{error::ResponseError, HttpResponse};
use sea_orm::DbErr;
use std::fmt;
use serde_json::json;

#[derive(Debug)]
pub enum ServiceError {
    DbError(DbErr),
    NotFound,
    Unauthorized,
    BadRequest(String),
    InternalServerError(String),
    TooManyRequests,
    Web3Error(String),
    InsufficientFunds(String),
    TransactionFailed(String),
    TransactionTimeout(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServiceError::DbError(e) => write!(f, "Database error: {}", e),
            ServiceError::NotFound => write!(f, "Not found"),
            ServiceError::Unauthorized => write!(f, "Unauthorized"),
            ServiceError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ServiceError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            ServiceError::TooManyRequests => write!(f, "Too many requests"),
            ServiceError::Web3Error(msg) => write!(f, "Web3 error: {}", msg),
            ServiceError::InsufficientFunds(msg) => write!(f, "Insufficient funds: {}", msg),
            ServiceError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            ServiceError::TransactionTimeout(msg) => write!(f, "Transaction timeout: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::DbError(e) => {
                log::error!("Database error: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Internal Server Error",
                    "error_code": "DB_ERROR"
                }))
            }
            ServiceError::NotFound => {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Resource not found",
                    "error_code": "NOT_FOUND"
                }))
            }
            ServiceError::Unauthorized => {
                HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Unauthorized",
                    "error_code": "UNAUTHORIZED"
                }))
            }
            ServiceError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": msg,
                    "error_code": "BAD_REQUEST"
                }))
            }
            ServiceError::InternalServerError(msg) => {
                log::error!("Internal server error: {}", msg);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": msg,
                    "error_code": "INTERNAL_SERVER_ERROR"
                }))
            }
            ServiceError::TooManyRequests => {
                HttpResponse::TooManyRequests().json(json!({
                    "status": "error",
                    "message": "Too many requests",
                    "error_code": "TOO_MANY_REQUESTS"
                }))
            }
            ServiceError::Web3Error(msg) => {
                log::error!("Web3 error: {}", msg);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": msg,
                    "error_code": "WEB3_ERROR"
                }))
            }
            ServiceError::InsufficientFunds(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": msg
                }))
            }
            ServiceError::TransactionFailed(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": msg
                }))
            }
            ServiceError::TransactionTimeout(msg) => {
                HttpResponse::GatewayTimeout().json(json!({
                    "error": msg
                }))
            }
        }
    }
}

impl From<DbErr> for ServiceError {
    fn from(err: DbErr) -> ServiceError {
        ServiceError::DbError(err)
    }
}