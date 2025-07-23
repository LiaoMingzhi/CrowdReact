// src/config/config.rs
use config::{Config as ConfigBuilder, ConfigError, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PrizeDistributionConfig {
    pub user_pool_percentage: f64,
    pub first_prize_percentage: f64,
    pub second_prize_percentage: f64,
    pub third_prize_percentage: f64,
    pub level_one_agent_percentage: f64,
    pub level_two_agent_percentage: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Web3Config {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContractConfig {
    pub address: String,
    pub owner: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainConfig {
    pub id: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisPoolConfig {
    pub max_size: usize,
    pub connection_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub queue_mode: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool: RedisPoolConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeekActionConfig {
    pub platform_account: String,
    pub prize_pool_account: String,
    pub confirm_level_one_agents: bool,
    pub confirm_level_two_agents: bool,
    pub confirm_level_common_agents: bool,
    pub sunday_lottery_distribution: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchedulerConfig {
    pub draw_lottery_cron: String,
    pub process_rewards_cron: String,
    pub cleanup_expired_cron: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiration: i32,
    pub token_duration_hours: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeekTodayConfig {
    pub today: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeoIpConfig {
    pub database_path: String,
    pub blocked_countries: Vec<String>,
    pub block_message: String,
    pub block_status_code: u16,
    pub block_headers: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub web3: Web3Config,
    pub contract: ContractConfig,
    pub week_action: WeekActionConfig,
    pub prize_distribution: PrizeDistributionConfig,
    pub chain: ChainConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub week_today: WeekTodayConfig,
    pub geoip: GeoIpConfig
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "default".to_string());
        
        let builder = ConfigBuilder::builder()
            .add_source(File::with_name("src/config/default.yaml"))
            .add_source(File::with_name(&format!("src/config/{}", env)).required(false))
            .add_source(Environment::with_prefix("APP").try_parsing(true));

        builder.build()?.try_deserialize()
    }
}
