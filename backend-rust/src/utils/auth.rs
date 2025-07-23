use actix_web::{error::ErrorUnauthorized, Error, HttpRequest};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::Config;
use lazy_static::lazy_static;
use log::{error, info};

lazy_static! {
    static ref CONFIG: Config = Config::from_env()
        .expect("Failed to load configuration");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,  // user_id
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ErrorUnauthorized(e.to_string()))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ErrorUnauthorized(e.to_string()))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_token(user_id: i32) -> Result<String, Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + CONFIG.auth.token_duration_hours as usize * 3600;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.auth.jwt_secret.as_bytes()),
    )
    .map_err(|e| ErrorUnauthorized(e.to_string()))
}

pub fn get_user_id_from_token(req: &HttpRequest) -> Result<i32, String> {
    info!("Starting token validation");
    
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => {
            info!("Found Authorization header: {:?}", header);
            header
        },
        None => {
            error!("No Authorization header found");
            return Err("No Authorization header".to_string());
        }
    };

    let token = match auth_header.to_str() {
        Ok(t) => {
            let token_str = t.replace("Bearer ", "");
            info!("Extracted token (first 10 chars): {:?}...", &token_str[..10.min(token_str.len())]);
            token_str
        },
        Err(e) => {
            error!("Failed to parse Authorization header: {:?}", e);
            return Err("Invalid Authorization header".to_string());
        }
    };

    decode_token(&token)
}

pub fn decode_token(token: &str) -> Result<i32, String> {
    info!("Attempting to decode token with length: {}", token.len());
    info!("JWT Secret length: {}", CONFIG.auth.jwt_secret.len());
    
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256];

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(CONFIG.auth.jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            info!("Token successfully decoded for user_id: {}", token_data.claims.sub);
            Ok(token_data.claims.sub)
        },
        Err(e) => {
            error!("Token decode error: {:?}", e);
            error!("JWT Secret first 5 chars: {:?}", CONFIG.auth.jwt_secret.chars().take(5).collect::<String>());
            Err(format!("Token validation failed: {}", e))
        }
    }
}