// 用户相关的服务逻辑
// src/services/user_service.rs

// 假设有一个数据库模块
use crate::models::user_model::{self, ActiveModel, Entity as User, Model as UserModel};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, DecodingKey, EncodingKey, Header, Validation,
};
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set, ColumnTrait, ConnectionTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use log::{info, error};
use crate::utils::auth::generate_token;
use crate::config::Config;

// JWT Claims 结构
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32, // user_id
    exp: usize,
}

#[derive(Clone)]
pub struct UserService {
    db: Arc<DatabaseConnection>,
    config: Config,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let config = Config::from_env()
            .expect("Failed to load configuration");
        Self { db, config }
    }

    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<UserModel, DbErr> {
        // 密码加密
        let password_hash = self
            .hash_password(&password)
            .map_err(|e| DbErr::Custom(format!("Password hashing error: {}", e)))?;

        let current_time = Utc::now();
        let user = user_model::ActiveModel {
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            created_at: Set(current_time),
            updated_at: Set(Some(current_time)),
            ..Default::default()
        };

        user.insert(&*self.db).await
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<UserModel>, DbErr> {
        User::find_by_id(user_id).one(&*self.db).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserModel>, DbErr> {
        User::find()
            .filter(user_model::Column::Email.eq(email))
            .one(&*self.db)
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserModel>, DbErr> {
        User::find()
            .filter(user_model::Column::Username.eq(username))
            .one(&*self.db)
            .await
    }

    pub async fn get_all_users(&self) -> Result<Vec<UserModel>, DbErr> {
        User::find().all(&*self.db).await
    }

    pub async fn update_user(
        &self,
        user_id: i32,
        username: Option<String>,
        email: Option<String>,
        password: Option<String>,
    ) -> Result<UserModel, DbErr> {
        let user = self
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))?;

        let mut user: user_model::ActiveModel = user.into();

        if let Some(username) = username {
            user.username = Set(username);
        }
        if let Some(email) = email {
            user.email = Set(email);
        }
        if let Some(password) = password {
            let password_hash = self
                .hash_password(&password)
                .map_err(|e| DbErr::Custom(format!("Password hashing error: {}", e)))?;
            user.password_hash = Set(password_hash);
        }

        user.updated_at = Set(Some(Utc::now()));
        user.update(&*self.db).await
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<(), DbErr> {
        User::delete_by_id(user_id).exec(&*self.db).await?;
        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    pub fn verify_password(
        &self,
        password: &str,
        hash: &str,
    ) -> Result<bool, argon2::password_hash::Error> {
        info!(
            "Password verification attempt:\n\
             Input password length: {}\n\
             Input password: {}\n\
             Password hash length: {}\n\
             Stored hash: {}\n\
             Hash format valid: {}",
            password.len(),
            password,
            hash.len(),
            hash,
            hash.starts_with("argon2")
        );
        
        let parsed_hash = PasswordHash::new(hash)?;
        let result = Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        
        info!(
            "Password verification details: input_bytes={:?}, result={}",
            password.as_bytes(),
            result
        );
        
        Ok(result)
    }

    // 注册新用户
    pub async fn register(
        &self,
        username: String,
        password: String,
        email: String,
    ) -> Result<UserModel, DbErr> {
        // 检查用户名或邮箱是否已存在
        if let Some(_) = self.get_user_by_email(&email).await? {
            return Err(DbErr::Custom("Email already exists".to_string()));
        }

        // 创建新用户
        self.create_user(username, email, password).await
        
    }

    // 用户登录
    pub async fn login(&self, username: String, password: String) -> Result<String, DbErr> {
        // 查找用户 - 不区分大小写
        let user = User::find()
            .filter(user_model::Column::Username.eq(username.to_lowercase()))
            .one(&*self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))?;

        // 验证密码
        if !self.verify_password(&password, &user.password_hash)
            .map_err(|_| DbErr::Custom("Password verification failed".to_string()))? {
            return Err(DbErr::Custom("Invalid password".to_string()));
        }

        // 生成 token
        Ok(generate_token(user.id)
            .map_err(|e| DbErr::Custom(format!("Token generation failed: {}", e)))?)
    }

    // 从 token 获取用户资料
    pub async fn get_profile_from_token(&self, token: &str) -> Result<UserModel, DbErr> {
        let user_id = self
            .verify_token(token)
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        self.get_user_by_id(user_id)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))
    }

    // 更新用户资料
    pub async fn update_profile(
        &self,
        token: &str,
        email: Option<String>,
        password: Option<String>,
    ) -> Result<UserModel, DbErr> {
        let user_id = self
            .verify_token(token)
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        self.update_user(user_id, None, email, password).await
    }

    // 获取用户余额
    pub async fn get_balance(&self, token: &str) -> Result<Decimal, DbErr> {
        let user_id = self
            .verify_token(token)
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // 这里需要实现实际的余额查询逻辑
        // 暂时返回模拟数据
        Ok(Decimal::new(1000, 2)) // 返回 10.00
    }

    // 生成 JWT token
    fn generate_token(&self, user_id: i32) -> Result<String, JwtError> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.auth.jwt_secret.as_bytes()),
        )
    }

    // 验证 JWT token
    fn verify_token(&self, token: &str) -> Result<i32, JwtError> {
        info!("Verifying token: {}", token);
        info!("Token validation details: length={}, format={}", token.len(), token.starts_with("Bearer "));
        let validation = Validation::default();
        
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.auth.jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(token_data) => {
                info!("Token successfully decoded for user: {}", token_data.claims.sub);
                Ok(token_data.claims.sub)
            }
            Err(e) => {
                error!("Token validation error: {:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn create_test_service() -> (UserService, Arc<DatabaseConnection>) {
        let db = Arc::new(
            MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results(vec![vec![UserModel {
                    id: 1,
                    username: "test_user".to_string(),
                    email: "tests@example.com".to_string(),
                    password_hash: "hashed_password".to_string(),
                    created_at: Utc::now(),
                    updated_at: None,
                }]])
                .append_exec_results(vec![MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                }])
                .into_connection(),
        );

        let service = UserService::new(Arc::clone(&db));
        (service, db)
    }

    #[tokio::test]
    async fn test_create_user() {
        let (service, _) = create_test_service();

        let result = service
            .create_user(
                "test_user".to_string(),
                "tests@example.com".to_string(),
                "password123".to_string(),
            )
            .await;

        assert!(result.is_ok());
        if let Ok(user) = result {
            assert_eq!(user.username, "test_user");
            assert_eq!(user.email, "tests@example.com");
        }
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let (service, _) = create_test_service();

        let result = service.get_user_by_email("tests@example.com").await;

        assert!(result.is_ok());
        if let Ok(Some(user)) = result {
            assert_eq!(user.email, "tests@example.com");
            assert_eq!(user.username, "test_user");
        }
    }

    #[test]
    fn test_password_hash_and_verify() {
        let (service, _) = create_test_service();

        let password = "test_password";
        let hash_result = service.hash_password(password);
        assert!(hash_result.is_ok());

        if let Ok(hash) = hash_result {
            let verify_result = service.verify_password(password, &hash);
            assert!(verify_result.is_ok());
            assert!(verify_result.unwrap());

            // 测试错误密码
            let wrong_verify = service.verify_password("wrong_password", &hash);
            assert!(wrong_verify.is_ok());
            assert!(!wrong_verify.unwrap());
        }
    }
}
