mod config;
mod handlers;
mod models;
mod routes;
mod services;
mod entities;
mod utils;
mod middleware;

use actix_web::{http, web, App, HttpServer};
use actix_web::middleware::Logger;
use crate::config::config::Config;
use routes::{bet_route, user_route};
use sea_orm::{ConnectionTrait, Database};
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};
use services::{
    amount_service::AmountService,
    buy_luck_number_service::BuyLuckNumberService,
    user_service::UserService,
    week_action_service::{WeekActionConfig, WeekActionService},
    auth_service::AuthService,
    agent_service::AgentService,
    scheduler_service::start_scheduled_tasks,
};
use std::sync::Arc;
use web3::transports::Http;
use web3::Web3;
use crate::handlers::{bet_handler, user_handler};
use crate::handlers::bet_handler::get_transaction_params;
use std::fs;
use std::path::Path;
use std::time::Duration;
use ::config::{ConfigBuilder, Environment, File};
use ::config::builder::DefaultState;
use config::{Config as ConfigCrate};
use actix_cors::Cors;
use sea_orm::Statement;
use log::{info, error};
use dotenv::dotenv;
use crate::middleware::ip_filter::IpFilter;
use crate::middleware::rate_limit::RateLimiter;
use web3::types::H160;

async fn run_migrations(db: &sea_orm::DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let migrations_dir = Path::new("migrations");
    
    // 检查目录是否存在
    if !migrations_dir.exists() {
        error!("Migrations directory not found");
        return Ok(());
    }

    // 读取并排序所有 .sql 文件
    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "sql" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    // 按文件名排序
    migration_files.sort();

    // 执行每个迁移文件
    for path in migration_files {
        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
            
        // 检查是否已执行过这个迁移
        let version = file_name.to_string();
        let check_sql = "SELECT version FROM schema_migrations WHERE version = $1";
        let result = db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            check_sql,
            vec![version.clone().into()]
        )).await;

        // 如果查询失败，说明 schema_migrations 表可能还不存在
        if result.is_err() {
            info!("Migrations table not found, executing all migrations");
        } else if result.unwrap().rows_affected() > 0 {
            info!("Migration {} already applied, skipping", file_name);
            continue;
        }
            
        info!("Executing migration: {}", file_name);
        
        // 读取并执行 SQL 文件
        let sql = fs::read_to_string(&path)?;
        for statement in sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                match db.execute(Statement::from_string(
                    db.get_database_backend(),
                    statement.to_owned(),
                )).await {
                    Ok(_) => info!("Successfully executed statement in {}", file_name),
                    Err(e) => {
                        error!("Error executing statement in {}: {}", file_name, e);
                        return Err(Box::new(e));
                    }
                }
            }
        }

        // 记录已执行的迁移
        let insert_sql = "INSERT INTO schema_migrations (version) VALUES ($1)";
        if let Err(e) = db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            insert_sql,
            vec![version.into()]
        )).await {
            error!("Failed to record migration {}: {}", file_name, e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

pub async fn start_server(config: Config) -> std::io::Result<()> {
    env_logger::init();
    info!("Starting server on http://127.0.0.1:9080");

    // 不需要再次创建配置，直接使用传入的 config
    let config = Arc::new(config);

    // 从配置中获取数据库连接信息
    let database_url = &config.database.url;
    
    // 连接数据库
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    // 运行迁移
    info!("Running SQL migrations...");
    match run_migrations(&db).await {
        Ok(_) => info!("Database migrations completed successfully"),
        Err(e) => error!("Migration Error: {}", e),
    }

    let db = Arc::new(db);
    
    // 初始化 UserService
    let user_service = Arc::new(UserService::new(Arc::clone(&db)));

    // 创建 Web3 transport 和 client
    let transport = Http::new(&config.web3.url)
        .expect("Failed to create HTTP transport");
    let web3 = Web3::new(transport);

    // 创建 AmountService，使用 Arc::clone(&config)
    let amount_service = Arc::new(AmountService::new(
        Arc::clone(&db),
        web3,
        config.contract.address.clone(),
        config.chain.id.to_string(),
        config.week_action.prize_pool_account.clone()
    ));

    let buy_luck_number_service = Arc::new(BuyLuckNumberService::new(
        Arc::clone(&db),
        Arc::clone(&amount_service)
    ));

    // 创建 WeekActionService
    let week_action_service = Arc::new(WeekActionService::new(
        Arc::clone(&db),
        Arc::clone(&buy_luck_number_service),
        Arc::clone(&amount_service),
        WeekActionConfig::from_env("default").expect("Failed to load WeekAction config")
    ));

    // 创建 AuthService 实例
    let auth_service = web::Data::new(AuthService::with_config(
        Arc::clone(&user_service),
        Arc::clone(&db),
        &config.redis.url,
        7 * 24 * 60 * 60  // token expiration in seconds (7 days)
    ).expect("Failed to create AuthService"));

    let ip_filter = Arc::new(IpFilter::new(
        &config.geoip.database_path,
        config.geoip.blocked_countries.clone()
    ));
    let ip_filter_data = web::Data::new(ip_filter);

    let agent_service = Arc::new(AgentService::new(Arc::clone(&db)));
    let agent_service_data = web::Data::new(agent_service);

    // 在 tokio::spawn 之前克隆需要的资源
    let scheduler_db = Arc::clone(&db);
    let scheduler_buy_luck_number_service = Arc::clone(&buy_luck_number_service);
    let scheduler_amount_service = Arc::clone(&amount_service);

    // 启动调度器，在后台运行调度器任
    tokio::spawn(async move {
        if let Err(e) = start_scheduled_tasks(
            scheduler_db,
            scheduler_buy_luck_number_service,
            scheduler_amount_service
        ).await {
            error!("Scheduler error: {}", e);
        }
    });

    info!("Starting server with routes:");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("https://www.luckgame123.com")
            .allowed_origin("https://luckgame123.com")
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::new("%r %s %b %{Referer}i %a %T"))
            .wrap(RateLimiter)
            .app_data(web::Data::new(Arc::clone(&db)))
            .app_data(web::Data::new(Arc::clone(&user_service)))
            .app_data(web::Data::new(Arc::clone(&week_action_service)))
            .app_data(web::Data::new(Arc::clone(&config)))
            .app_data(web::Data::clone(&auth_service))
            .app_data(web::Data::clone(&ip_filter_data))
            .app_data(web::Data::clone(&agent_service_data))
            .configure(user_route::user_config)
            .configure(bet_route::bet_config)
    })
    .workers(num_cpus::get() * 2)// 根据CPU核心数设置工作线程
    .backlog(2048) // 设置等待队列大小
    .keep_alive(Duration::from_secs(30))
    .client_request_timeout(Duration::from_secs(60))
    .bind(("127.0.0.1", 9080))? // 127.0.0.1 改成 0.0.0.0
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 首先加载 .env 文件
    dotenv().ok();
    
    let config = ConfigBuilder::<DefaultState>::default()
        .add_source(File::with_name("config/default.toml"))
        .add_source(File::with_name("config/development.toml"))
        .add_source(Environment::with_prefix("APP"))
        .build()
        .expect("Failed to load config")
        .try_deserialize::<Config>()
        .expect("Failed to deserialize config");

    // 添加日志输出，帮助调试
    info!("Loaded configuration: {:?}", config);
    
    start_server(config).await
}
