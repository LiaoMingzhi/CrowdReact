use crate::models::user_token_model::{self, Entity as UserToken};
use crate::services::user_service::UserService;
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Utc};
use config::{Config, ConfigError, Environment, File};
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use metrics::{counter, gauge};
use redis::AsyncCommands;
use sea_orm::{
    ActiveModelTrait, 
    ColumnTrait,
    DatabaseConnection, 
    DbErr, 
    EntityTrait, 
    QueryFilter, 
    Set, 
    TransactionTrait
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
// 身份验证相关的服务逻辑
// src/services/auth_service.rs
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration as StdDuration;
use thiserror::Error;
use tracing::{error, info, instrument, warn};
use crate::models::user_model::{self, Entity as User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: i32, // user_id，用户ID
    exp: i64, // expiration time，过期时间
    iat: i64, // issued at，签发时间
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token blacklisted")]
    TokenBlacklisted,
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Health check failed")]
    HealthCheckError,
}

pub struct AuthService {
    user_service: Arc<UserService>,
    db: Arc<DatabaseConnection>,
    redis_pool: RedisPool,
    token_expiration: i32,
    metrics: Arc<RwLock<Metrics>>, // 添加 metrics 字段
    jwt_secret: Vec<u8>,           // 添加 JWT 密钥字段
    config: AuthConfig,
}

#[derive(Default)]
struct Metrics {
    counters: HashMap<String, AtomicU64>,
    gauges: HashMap<String, AtomicU64>,
    latencies: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub redis_url: String,
    pub token_expiration: i32,
    pub jwt_secret: String,
    pub token_duration_hours: i64,
    pub redis_pool: RedisPoolConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            token_expiration: 3600,                       // 1 hour in seconds
            jwt_secret: "default_secret_key".to_string(), // 仅用于开发环境
            token_duration_hours: 24,                     // 24 hours
            redis_pool: RedisPoolConfig::default(),
        }
    }
}

impl AuthConfig {
    pub fn from_env(env: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", env)))
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        Ok(Self {
            redis_url: config.get_string("redis.url")?,
            token_expiration: config.get_int("auth.token_expiration")? as i32,
            jwt_secret: config.get_string("auth.jwt_secret")?,
            token_duration_hours: config.get_int("auth.token_duration_hours")? as i64,
            redis_pool: RedisPoolConfig {
                max_size: config.get_int("redis.pool.max_size")? as usize,
                connection_timeout: StdDuration::from_secs(
                    config.get_int("redis.pool.connection_timeout_secs")? as u64,
                ),
                idle_timeout: Some(StdDuration::from_secs(
                    config.get_int("redis.pool.idle_timeout_secs")? as u64,
                )),
                queue_mode: match config.get_string("redis.pool.queue_mode")?.as_str() {
                    "lifo" => deadpool::managed::QueueMode::Lifo,
                    _ => deadpool::managed::QueueMode::Fifo,
                },
            },
        })
    }
}

impl AuthService {
    pub fn new(
        user_service: Arc<UserService>,
        db: Arc<DatabaseConnection>,
        config: AuthConfig,
    ) -> Result<Self, AuthError> {
        // 验证 JWT secret 是否有效
        if config.jwt_secret.starts_with("${") || config.jwt_secret.len() < 32 {
            panic!("Invalid JWT secret. Please set a valid JWT_SECRET environment variable");
        }
        
        // 创建可变的 Redis 配置
        let mut cfg = RedisConfig::from_url(&config.redis_url);

        // 应用连接池配置
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: config.redis_pool.max_size,
            timeouts: deadpool_redis::Timeouts {
                wait: Some(config.redis_pool.connection_timeout),
                create: Some(config.redis_pool.connection_timeout),
                recycle: config.redis_pool.idle_timeout,
            },
            queue_mode: config.redis_pool.queue_mode,
        });

        // 创建 Redis 连接池
        let redis_pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        Ok(Self {
            user_service,
            db,
            redis_pool,
            token_expiration: config.token_expiration,
            metrics: Arc::new(RwLock::new(Metrics::default())),
            jwt_secret: config.jwt_secret.clone().into_bytes(),
            config,
        })
    }

    /// 使用默认配置创建 AuthService 实例
    pub fn with_defaults(
        user_service: Arc<UserService>,
        db: Arc<DatabaseConnection>,
    ) -> Result<Self, AuthError> {
        Self::new(user_service, db, AuthConfig::default())
    }

    /// 使用自定义配置创建 AuthService 实例
    pub fn with_config(
        user_service: Arc<UserService>,
        db: Arc<DatabaseConnection>,
        redis_url: &str,
        token_expiration: i32,
    ) -> Result<Self, AuthError> {
        // 优先从 .env 获取 JWT_SECRET
        let jwt_secret = std::env::var("JWT_SECRET").or_else(|_| {
            // 如果环境变量不存在，从配置文件获取
            let config = config::Config::builder()
                .add_source(config::File::with_name("config/development.toml"))
                .build()
                .map_err(|e| AuthError::ConfigError(e.to_string()))?;
                
            config.get_string("auth.jwt_secret")
                .map_err(|e| AuthError::ConfigError(e.to_string()))
        })?;

        let config = AuthConfig {
            redis_url: redis_url.to_string(),
            token_expiration,
            jwt_secret,
            token_duration_hours: 24,
            redis_pool: RedisPoolConfig::default(),
        };

        Self::new(user_service, db, config)
    }

    /// 验证用户凭证
    pub async fn validate_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<i32, AuthError> {
        // 使用不区分大小写的比较
        let user = User::find()
            .filter(user_model::Column::Username.eq(&username.to_lowercase()))
            .one(&*self.db)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        // 添加更详细的日志
        info!(
            "Validating credentials for user: {} (stored as: {})",
            username,
            user.username
        );
        
        if !self.user_service.verify_password(password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)? 
        {
            warn!("Password verification failed for user {}", username);
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user.id)
    }

    /// 生成 JWT token
    pub fn generate_token(&self, user_id: i32) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = (now + ChronoDuration::hours(self.config.token_duration_hours)).timestamp();
        let claims = Claims {
            sub: user_id,
            exp,
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// 验证 JWT token
    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data)
    }

    /// 登录处理
    pub async fn login(&self, email: &str, password: &str) -> Result<String, AuthError> {
        let user_id = self.validate_credentials(email, password).await?;
        self.generate_token(user_id)
    }

    /// 退出处理
    #[instrument(skip(self, token), err)]
    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        // 开启数据库事务
        let mut transaction = self.db.begin().await.map_err(AuthError::DatabaseError)?;

        // 1. 在数据库中标记 token 为已效
        if let Some(token_record) = UserToken::find()
            .filter(user_token_model::Column::Token.eq(token))
            .one(&transaction)
            .await
            .map_err(AuthError::DatabaseError)?
        {
            let mut token_model: user_token_model::ActiveModel = token_record.into();
            token_model.is_valid = Set(false);
            token_model.updated_at = Set(Utc::now());
            token_model
                .update(&transaction)
                .await
                .map_err(AuthError::DatabaseError)?;
        }

        // 2. 将 token 加入 Redis 黑名单
        let mut redis_conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        let blacklist_key = format!("token_blacklist:{}", token);
        redis_conn
            .set_ex(&blacklist_key, "logged_out", self.token_expiration as u64)
            .await
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        // 3. 清理用户缓存
        let user_cache_key = format!("user_cache:{}", token);
        redis_conn
            .del(&user_cache_key)
            .await
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        // 4. 提交事务
        transaction
            .commit()
            .await
            .map_err(AuthError::DatabaseError)?;

        info!("User logged out successfully with token: {}", token);
        Ok(())
    }

    // 辅助方法：检查 token 是否在黑名单中
    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool, AuthError> {
        let mut redis_conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        let blacklist_key = format!("token_blacklist:{}", token);
        let exists: bool = redis_conn
            .exists(&blacklist_key)
            .await
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        Ok(exists)
    }

    // 添加配置验证
    fn validate_config(redis_url: &str, token_expiration: i32) -> Result<(), AuthError> {
        if token_expiration <= 0 {
            return Err(AuthError::ConfigError(
                "Token expiration must be positive".into(),
            ));
        }

        if !redis_url.starts_with("redis://") {
            return Err(AuthError::ConfigError("Invalid Redis URL format".into()));
        }

        Ok(())
    }

    // 添加带连接池配置的构造函数
    pub fn with_pool_config(
        user_service: Arc<UserService>,
        db: Arc<DatabaseConnection>,
        redis_url: &str,
        token_expiration: i32,
        pool_config: RedisPoolConfig,
    ) -> Result<Self, AuthError> {
        Self::validate_config(redis_url, token_expiration)?;

        // 创建基本的 Redis 配置
        let mut cfg = RedisConfig::from_url(redis_url);

        // 应用连接池配置
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: pool_config.max_size,
            timeouts: deadpool_redis::Timeouts {
                wait: Some(pool_config.connection_timeout),
                create: Some(pool_config.connection_timeout),
                recycle: pool_config.idle_timeout,
            },
            queue_mode: deadpool::managed::QueueMode::Fifo, // 添加队列模式
        });

        // 创建 Redis 连接池
        let redis_pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AuthError::RedisError(e.to_string()))?;

        Ok(Self {
            user_service,
            db,
            redis_pool,
            token_expiration,
            metrics: Arc::new(RwLock::new(Metrics::default())),
            jwt_secret: "default_secret_key".to_string().into_bytes(),
            config: AuthConfig::default(),
        })
    }

    // 添加重试机制
    async fn redis_operation_with_retry<F, T>(&self, operation: F) -> Result<T, AuthError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AuthError>> + Send>>,
    {
        let mut retries = 0;
        let max_retries = 3;
        let base_delay = StdDuration::from_millis(100);

        loop {
            match operation().await {
                Ok(result) => {
                    counter!("auth_service_redis_success", 1);
                    return Ok(result);
                }
                Err(e) => {
                    retries += 1;
                    counter!("auth_service_redis_retry", 1);

                    if retries >= max_retries {
                        error!(
                            "Redis operation failed after {} retries: {:?}",
                            max_retries, e
                        );
                        counter!("auth_service_redis_failure", 1);
                        return Err(e);
                    }

                    let delay = base_delay * 2u32.pow(retries as u32);
                    warn!("Redis operation failed, retrying in {:?}: {:?}", delay, e);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    // 添加健康检查
    pub async fn check_health(&self) -> Result<HealthStatus, AuthError> {
        let redis_start = std::time::Instant::now();
        let redis_status = self.check_redis_health().await;
        let redis_latency = redis_start.elapsed();

        let db_start = std::time::Instant::now();
        let db_status = self.check_db_health().await;
        let db_latency = db_start.elapsed();

        // 记录监控指标
        gauge!(
            "auth_service_redis_latency",
            redis_latency.as_millis() as f64
        );
        gauge!("auth_service_db_latency", db_latency.as_millis() as f64);

        Ok(HealthStatus {
            redis_health: redis_status,
            db_health: db_status,
            redis_latency,
            db_latency,
        })
    }

    async fn check_redis_health(&self) -> bool {
        let mut conn = match self.redis_pool.get().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };

        redis::cmd("PING")
            .query_async::<_>(&mut *conn)
            .await
            .map(|response: String| response == "PONG")
            .unwrap_or(false)
    }

    async fn check_db_health(&self) -> bool {
        self.db.ping().await.is_ok()
    }

    /// 增加计数器
    pub fn increment_counter(&self, name: &str) {
        info!("Incrementing counter: {}", name);

        let metrics = self.metrics.read().unwrap();
        if let Some(counter) = metrics.counters.get(name) {
            counter.fetch_add(1, Ordering::Relaxed);
        } else {
            drop(metrics); // 释放读锁
            let mut metrics = self.metrics.write().unwrap();
            metrics
                .counters
                .entry(name.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 记录延迟时间
    pub fn record_latency(&self, name: &str, duration: StdDuration) {
        let duration_ms = duration.as_millis() as u64;
        info!("Recording latency: {} = {}ms", name, duration_ms);

        let mut metrics = self.metrics.write().unwrap();
        metrics
            .latencies
            .entry(name.to_string())
            .or_default()
            .push(duration_ms);

        // 保持最近 1000 个样本
        if metrics.latencies[name].len() > 1000 {
            metrics.latencies.get_mut(name).unwrap().remove(0);
        }
    }

    /// 设置仪表盘值
    pub fn record_gauge(&self, name: &str, value: f64) {
        let value_raw = (value * 1000.0) as u64; // 存储带3位小数的精度
        info!("Recording gauge: {} = {}", name, value);

        let metrics = self.metrics.read().unwrap();
        if let Some(gauge) = metrics.gauges.get(name) {
            gauge.store(value_raw, Ordering::Relaxed);
        } else {
            drop(metrics); // 释放读锁
            let mut metrics = self.metrics.write().unwrap();
            metrics
                .gauges
                .entry(name.to_string())
                .or_insert_with(|| AtomicU64::new(value_raw));
        }
    }

    /// 获取计数器值
    pub fn get_counter(&self, name: &str) -> u64 {
        let metrics = self.metrics.read().unwrap();
        metrics
            .counters
            .get(name)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 获取延迟统计信息
    pub fn get_latency_stats(&self, name: &str) -> Option<LatencyStats> {
        let metrics = self.metrics.read().unwrap();
        let latencies = metrics.latencies.get(name)?;

        if latencies.is_empty() {
            return None;
        }

        let mut values = latencies.clone();
        values.sort_unstable();

        Some(LatencyStats {
            count: values.len() as u64,
            min: *values.first().unwrap(),
            max: *values.last().unwrap(),
            avg: values.iter().sum::<u64>() / values.len() as u64,
            p50: values[values.len() / 2],
            p95: values[(values.len() * 95) / 100],
            p99: values[(values.len() * 99) / 100],
        })
    }

    /// 获取仪表盘值
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.read().unwrap();
        metrics
            .gauges
            .get(name)
            .map(|gauge| gauge.load(Ordering::Relaxed) as f64 / 1000.0)
    }

    pub async fn create(
        user_service: Arc<UserService>,
        db: Arc<DatabaseConnection>,
        env: &str,
    ) -> Result<Self, AuthError> {
        let config =
            AuthConfig::from_env(env).map_err(|e| AuthError::ConfigError(e.to_string()))?;

        Self::new(user_service, db, config)
    }
}

// 更新 RedisPoolConfig 结构体
#[derive(Debug, Clone)]
pub struct RedisPoolConfig {
    pub max_size: usize,
    pub connection_timeout: StdDuration,
    pub idle_timeout: Option<StdDuration>,
    pub queue_mode: deadpool::managed::QueueMode, // 添加队��模式字段
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 16,
            connection_timeout: StdDuration::from_secs(5),
            idle_timeout: Some(StdDuration::from_secs(300)),
            queue_mode: deadpool::managed::QueueMode::Fifo, // 默认使用 FIFO 模式
        }
    }
}

// 添加健康状态结构体
#[derive(Debug)]
pub struct HealthStatus {
    pub redis_health: bool,
    pub db_health: bool,
    pub redis_latency: StdDuration,
    pub db_latency: StdDuration,
}

/// 延迟统计信息
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub count: u64,
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool::managed::QueueMode;
    use sea_orm::{DatabaseBackend, MockDatabase};

    // 辅助函数：创建测试服务
    async fn create_test_services() -> (Arc<UserService>, AuthService) {
        let db = Arc::new(
            MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results(vec![Vec::<user_token_model::Model>::new()])
                .into_connection(),
        );

        let user_service = Arc::new(UserService::new(Arc::clone(&db)));

        let auth_service =
            AuthService::create(Arc::clone(&user_service), Arc::clone(&db), "testing")
                .await
                .expect("Failed to create AuthService");

        (user_service, auth_service)
    }

    #[tokio::test]
    async fn test_health_check() {
        let (_, auth_service) = create_test_services().await;
        let health = auth_service.check_health().await.unwrap();
        assert!(health.redis_health);
        assert!(health.db_health);
        assert!(health.redis_latency.as_millis() >= 0);
        assert!(health.db_latency.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_redis_retry() {
        let (_, auth_service) = create_test_services().await;
        let result = auth_service
            .redis_operation_with_retry(|| Box::pin(async { Ok::<_, AuthError>(()) }))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_token_generation() {
        let (_, auth_service) = create_test_services().await;
        let token = auth_service.generate_token(1).unwrap();
        assert!(!token.is_empty());

        // 验证 token 过期时间
        let token_data = auth_service.verify_token(&token).unwrap();
        let now = Utc::now().timestamp();
        assert!(token_data.claims.exp > now);
        assert!(token_data.claims.exp <= now + auth_service.config.token_duration_hours * 3600);
    }

    #[tokio::test]
    async fn test_custom_pool_config() {
        let (user_service, _) = create_test_services().await;
        let db = Arc::new(
            MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results(vec![Vec::<user_token_model::Model>::new()])
                .into_connection(),
        );

        let pool_config = RedisPoolConfig {
            max_size: 32,
            connection_timeout: StdDuration::from_secs(10),
            idle_timeout: Some(StdDuration::from_secs(600)),
            queue_mode: QueueMode::Lifo,
        };

        let service = AuthService::with_pool_config(
            Arc::clone(&user_service),
            Arc::clone(&db),
            "redis://127.0.0.1:6379",
            3600,
            pool_config,
        );

        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let (_, auth_service) = create_test_services().await;

        // 测试计数器
        auth_service.increment_counter("test_counter");

        // 测试延迟记录
        auth_service.record_latency("test_latency", StdDuration::from_millis(100));

        // 测试仪表盘
        auth_service.record_gauge("test_gauge", 42.0);
    }

    // 添加更多测试场景
    #[tokio::test]
    async fn test_auth_service_initialization() {
        let (_, auth_service) = create_test_services().await;

        // 验证服务初始化
        assert_eq!(auth_service.token_expiration, 3600);

        // 验证 Redis 连接池
        let mut conn = auth_service.redis_pool.get().await.unwrap();
        let ping_result: String = redis::cmd("PING").query_async(&mut conn).await.unwrap();
        assert_eq!(ping_result, "PONG");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (_, auth_service) = create_test_services().await;
        let auth_service = Arc::new(auth_service);

        let mut handles = vec![];

        for i in 0..10 {
            let service_clone = Arc::clone(&auth_service);
            let handle = tokio::spawn(async move {
                let token = service_clone.generate_token(i).unwrap();
                assert!(!token.is_empty());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_metrics() {
        let (_, auth_service) = create_test_services().await;

        // 测试计数器
        auth_service.increment_counter("test_counter");
        auth_service.increment_counter("test_counter");
        assert_eq!(auth_service.get_counter("test_counter"), 2);

        // 测试延迟记录
        auth_service.record_latency("test_latency", StdDuration::from_millis(100));
        auth_service.record_latency("test_latency", StdDuration::from_millis(200));
        let stats = auth_service.get_latency_stats("test_latency").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.min, 100);
        assert_eq!(stats.max, 200);
        assert_eq!(stats.avg, 150);

        // 测试仪表盘
        auth_service.record_gauge("test_gauge", 42.5);
        assert_eq!(auth_service.get_gauge("test_gauge").unwrap(), 42.5);
    }

    #[tokio::test]
    async fn test_concurrent_metrics() {
        let (_, auth_service) = create_test_services().await;
        let auth_service = Arc::new(auth_service);

        let mut handles = vec![];

        // 并发增加计数器
        for _ in 0..100 {
            let service_clone = Arc::clone(&auth_service);
            let handle = tokio::spawn(async move {
                service_clone.increment_counter("concurrent_counter");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(auth_service.get_counter("concurrent_counter"), 100);
    }
}

// 修改 create_auth_service 函数
pub async fn create_auth_service(
    user_service: Arc<UserService>,
    db: Arc<DatabaseConnection>,
    redis_url: &str,
) -> Result<AuthService, AuthError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("JWT_SECRET not found in environment, using default value");
            "default_secret_key".to_string()
        });

    let config = AuthConfig {
        redis_url: redis_url.to_string(),
        token_expiration: 3600,
        jwt_secret,
        token_duration_hours: 24,
        redis_pool: RedisPoolConfig::default(),
    };

    AuthService::new(user_service, db, config)
}

// 使用示例：
#[cfg(test)]
mod examples {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    async fn example_usage() -> Result<(), AuthError> {
        // 创建模拟数据库连接
        let db = Arc::new(
            MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results(vec![Vec::<user_token_model::Model>::new()])
                .into_connection(),
        );

        let user_service = Arc::new(UserService::new(Arc::clone(&db)));

        // 使用默认配置
        let auth_service = AuthService::with_defaults(Arc::clone(&user_service), Arc::clone(&db))?;

        // 或者使用自定义配置
        let auth_service_custom = AuthService::with_config(
            Arc::clone(&user_service),
            Arc::clone(&db),
            "redis://custom-host:6379",
            7200, // 2 hours
        )?;

        Ok(())
    }

    // 添加一个实际连接数据库的示例
    #[cfg(not(test))]
    async fn real_database_example() -> Result<(), AuthError> {
        use sea_orm::Database;

        // 从环境变量或配置文件获取数据库 URL
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/dbname".to_string());

        // 创建实际的数据库连接
        let db = Arc::new(
            Database::connect(&database_url)
                .await
                .expect("Failed to connect to database"),
        );

        let user_service = Arc::new(UserService::new(Arc::clone(&db)));

        let auth_service = AuthService::with_defaults(Arc::clone(&user_service), Arc::clone(&db))?;

        Ok(())
    }

    // 添加一个使用配置的示例
    async fn configured_example() -> Result<(), AuthError> {
        use config::{Config, Environment, File};
        use sea_orm::Database;

        // 创建配置
        let settings = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("APP"))
            .build()
            .expect("Failed to build config");

        // 从配置中获取数据库连接信息
        let database_url = settings
            .get_string("database.url")
            .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/dbname".to_string());

        let redis_url = settings
            .get_string("redis.url")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let token_expiration = settings.get_int("auth.token_expiration").unwrap_or(3600) as i32;

        // 创建数��库连接
        let db = Arc::new(
            Database::connect(&database_url)
                .await
                .expect("Failed to connect to database"),
        );

        let user_service = Arc::new(UserService::new(Arc::clone(&db)));

        // 使用配置创建 AuthService
        let auth_service = AuthService::with_config(
            Arc::clone(&user_service),
            Arc::clone(&db),
            &redis_url,
            token_expiration,
        )?;

        Ok(())
    }
}

impl From<AuthError> for actix_web::Error {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => actix_web::error::ErrorUnauthorized(err.to_string()),
            AuthError::TokenExpired => actix_web::error::ErrorUnauthorized(err.to_string()),
            AuthError::TokenBlacklisted => actix_web::error::ErrorUnauthorized(err.to_string()),
            AuthError::InvalidToken => actix_web::error::ErrorUnauthorized(err.to_string()),
            AuthError::DatabaseError(e) => actix_web::error::ErrorInternalServerError(e.to_string()),
            AuthError::JwtError(e) => actix_web::error::ErrorInternalServerError(e.to_string()),
            AuthError::RedisError(e) => actix_web::error::ErrorInternalServerError(e),
            AuthError::ConfigError(e) => actix_web::error::ErrorInternalServerError(e),
            AuthError::HealthCheckError => actix_web::error::ErrorInternalServerError("Health check failed"),
        }
    }
}
