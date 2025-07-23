// src/handlers/bet_handler.rs
use actix_web::{web, HttpResponse, get, Result, post, Error, error::ResponseError, http::StatusCode};
use sea_orm::{DatabaseConnection, QueryOrder, QuerySelect, DatabaseBackend, JoinType};
use sea_orm::sea_query::{Expr, Func, Alias};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::services::week_action_service::{WeekActionConfig, WeekActionService};
use sea_orm::prelude::Decimal;
use std::str::FromStr;
use chrono::{Datelike, Local};
use serde_json::json;
use ethers::types::U256;
use ethers::{
    types::Address,
    utils::parse_ether,
};
use crate::config::config::Config as AppConfig;
use log::{info, error, debug};
use web3::Web3;
use web3::transports::Http;
use crate::services::contract_service::verify_contract;
use web3::contract::{Contract, Options};
use web3::types::H160;
use sea_orm::{ActiveModelTrait, Set, EntityTrait};
use crate::entities::{bet_records, bet_records::Entity as BetRecords};
use sea_orm::prelude::*;  // 添加这行来导入 DateTimeWithTimeZone
use chrono::{DateTime, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use time::{UtcOffset, Weekday};
use web3::types::H256;
use crate::services::amount_service::AmountService;
use crate::services::buy_luck_number_service::BuyLuckNumberService;
use crate::models::agent_model::{self, Entity as Agents};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::models::buy_luck_number_model::{self, Entity as BuyLuckNumbers};
use crate::models::commission_model::{self, Entity as Commissions};
use crate::routes::config;
use crate::utils::error::ServiceError;
use crate::models::platform_prize_pool_model::{self, Entity as PlatformPrizePool};
use crate::models::lottery_distribution_detail_model::{self, Entity as LotteryDistributionDetail};
use chrono::prelude::*;
use chrono_tz::Tz;

// 请求和响应结构体定义
#[derive(Deserialize)]
pub struct BetRequest {
    pub account_address: String,
    pub amount: String,
    pub transaction_hash: String,
    pub block_number: i64,
    pub block_timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoResponse {
    account_address: String,
    mnemonic: String,
    signature: String,
    role_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct BetInfoResponse {
    purchase_amount: f64,
    account_address: String,
    purchase_numbers: Vec<String>,
    purchase_time: String,
}

#[derive(Serialize, Deserialize)]
pub struct AgentInfoResponse {
    agent_role: String,
    upper_agent: String,
    lower_agents: Vec<String>,
    total_commission: f64,
    commission_rank: i32,
}

#[derive(Debug, Serialize)]
pub struct CommissionDetail {
    pub user_address: String,
    pub from_address: String,
    pub commission: String,  // Using String to control decimal places
    pub transaction_hash: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommissionDetailsResponse {
    pub user_address: String,
    pub commission_details: Vec<CommissionDetail>,
    pub total: u64,
    pub total_amount: String,
}

#[derive(Serialize, Debug)]
pub struct TransactionParams {
    to: String,
    value: String,  // in wei
    data: String,   // hex encoded contract call data
}

#[derive(Debug, Deserialize)]
pub struct BetRecord {
    pub account_address: String,
    #[serde(deserialize_with = "deserialize_string_to_decimal")]
    pub amount: Decimal,
    pub transaction_hash: String,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub agent_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BetStatus {
    Pending,    // 交易已提交，等待确认
    Confirmed,  // 交易已确认
    Failed,     // 交易失败
}

impl std::fmt::Display for BetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BetStatus::Pending => write!(f, "pending"),
            BetStatus::Confirmed => write!(f, "confirmed"),
            BetStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BetHistoryEntry {
    pub amount: Decimal,
    pub transaction_hash: String,
    pub luck_number: Vec<String>,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BetHistoryResponse {
    pub user_address: String,
    pub bet_history: Vec<BetHistoryEntry>,
    pub total: u64,
    pub total_amount: String,
}

#[derive(Debug, Serialize)]
pub struct PrizePoolPrizeExpectionResponse {
    prize_amount: String,
    level_one_agent_prize: String,
    level_two_agent_prize: String,
    common_agent_prize: String,
    first_prize: String,
    second_prize: String,
    third_prize: String,
}

#[derive(Debug, Serialize)]
struct CompetitionInfo {
    user_address: String,
    topic: String,
    rank: i64,
    competition_count: i64,
}

#[derive(Debug, Serialize)]
struct LotteryInfo {
    user_address: String,
    prize_grade: String,
    luck_number: String,
    prize_amount: f64,
    time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct CompetitionLotteryResponse {
    competition: CompetitionInfo,
    lottery: Vec<LotteryInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BetHistoryQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub account_address: Option<String>,
}

pub async fn get_bet_history(
    db: web::Data<Arc<DatabaseConnection>>,
    query: web::Query<BetHistoryQuery>,
) -> Result<HttpResponse, ServiceError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let user_address = query.account_address.clone().unwrap_or_default();
    
    info!("bet_handler, get_bet_history, user_address: {}", user_address);
    
    // 获取总记录数
    let total = BetRecords::find()
        .filter(bet_records::Column::AccountAddress.eq(&user_address))
        .count(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to count bet records: {}", e);
            ServiceError::from(e)
        })?;

    // 添加分页逻辑
    let bet_records = BetRecords::find()
        .filter(bet_records::Column::AccountAddress.eq(&user_address))
        .order_by_desc(bet_records::Column::CreatedAt)
        .paginate(db.as_ref().as_ref(), limit as u64)
        .fetch_page(page as u64 - 1)
        .await
        .map_err(|e| {
            error!("Failed to fetch bet records: {}", e);
            ServiceError::from(e)
        })?;

    // 2. For each bet record, get corresponding luck numbers
    let mut bet_history = Vec::new();
    for bet_record in bet_records {
        // Get luck numbers for this transaction
        let luck_numbers = BuyLuckNumbers::find()
            .filter(buy_luck_number_model::Column::TransactionHash.eq(bet_record.transaction_hash.clone()))
            .all(db.as_ref().as_ref())
            .await
            .map_err(|e| {
                error!("Failed to fetch luck numbers: {}", e);
                ServiceError::from(e)
            })?;

        // Parse the amount string to Decimal
        let amount = Decimal::from_str(&bet_record.amount)
            .map_err(|e| {
                error!("Failed to parse amount to Decimal: {}", e);
                ServiceError::InternalServerError("Failed to parse amount".into())
            })?;

        let luck_numbers: Vec<String> = luck_numbers.into_iter()
            .map(|ln| ln.luck_number)
            .collect::<Vec<String>>();

        bet_history.push(BetHistoryEntry {
            amount,
            transaction_hash: bet_record.transaction_hash,
            luck_number: luck_numbers,
            time: bet_record.created_at,
        });
    }

    // 计算总金额
    let total_amount = BetRecords::find()
        .filter(bet_records::Column::AccountAddress.eq(&user_address))
        .all(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch records for total amount: {}", e);
            ServiceError::from(e)
        })?
        .iter()
        .filter_map(|record| Decimal::from_str(&record.amount).ok())
        .sum::<Decimal>();

    let response = BetHistoryResponse {
        user_address,
        bet_history,
        total,
        total_amount: format!("{:.3}", total_amount)
    };
    info!("response bet_handler, get_bet_history.");
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct AgentQuery {
    pub account_address: String,
}

pub async fn get_agent_details(
    db: web::Data<Arc<DatabaseConnection>>,
    query: web::Query<AgentQuery>,
) -> Result<HttpResponse, ServiceError> {
    let user_address = query.account_address.clone();
    info!("Fetching agent details for address: {}", user_address);

    // Get current agent's details
    let current_agent = Agents::find()
        .filter(agent_model::Column::UserAddress.eq(&user_address))
        .one(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch agent details: {}", e);
            ServiceError::from(e)
        })?;

    let current_agent = match current_agent {
        Some(agent) => agent,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Agent not found",
                "data": null
            })));
        }
    };

    let level_agent = current_agent.level_agent.clone();
    let mut one_agent = None;
    let mut two_agents = Vec::new();
    let mut common_agents = Vec::new();

    match level_agent.as_str() {
        "one" => {
            one_agent = Some(current_agent.clone());
            
            // Get level two agents under this level one agent
            two_agents = Agents::find()
                .filter(agent_model::Column::LevelAgent.eq("two"))
                .filter(agent_model::Column::SuperiorAddress.eq(&user_address))
                .all(db.as_ref().as_ref())
                .await
                .map_err(|e| {
                    error!("Failed to fetch level two agents: {}", e);
                    ServiceError::from(e)
                })?;

            // Get common agents under the level two agents
            for two_agent in &two_agents {
                let common = Agents::find()
                    .filter(agent_model::Column::LevelAgent.eq("common"))
                    .filter(agent_model::Column::SuperiorAddress.eq(&two_agent.user_address))
                    .all(db.as_ref().as_ref())
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch common agents: {}", e);
                        ServiceError::from(e)
                    })?;
                common_agents.extend(common);
            }
        },
        "two" => {
            // Get level one agent (upper agent)
            one_agent = Agents::find()
                .filter(agent_model::Column::LevelAgent.eq("one"))
                .filter(agent_model::Column::UserAddress.eq(current_agent.superior_address.clone()))
                .one(db.as_ref().as_ref())
                .await
                .map_err(|e| {
                    error!("Failed to fetch level one agent: {}", e);
                    ServiceError::from(e)
                })?;

            two_agents = vec![current_agent.clone()];

            // Get common agents under this level two agent
            common_agents = Agents::find()
                .filter(agent_model::Column::LevelAgent.eq("common"))
                .filter(agent_model::Column::SuperiorAddress.eq(&user_address))
                .all(db.as_ref().as_ref())
                .await
                .map_err(|e| {
                    error!("Failed to fetch common agents: {}", e);
                    ServiceError::from(e)
                })?;
        },
        "common" => {
            // Get level two agent (upper agent)
            let two_agent = Agents::find()
                .filter(agent_model::Column::LevelAgent.eq("two"))
                .filter(agent_model::Column::UserAddress.eq(current_agent.superior_address.clone()))
                .one(db.as_ref().as_ref())
                .await
                .map_err(|e| {
                    error!("Failed to fetch level two agent: {}", e);
                    ServiceError::from(e)
                })?;

            if let Some(two) = two_agent {
                two_agents = vec![two.clone()];
                
                // Get level one agent
                one_agent = Agents::find()
                    .filter(agent_model::Column::LevelAgent.eq("one"))
                    .filter(agent_model::Column::UserAddress.eq(two.superior_address.clone()))
                    .one(db.as_ref().as_ref())
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch level one agent: {}", e);
                        ServiceError::from(e)
                    })?;
            }

            common_agents = vec![current_agent.clone()];
        },
        "not_agent" => {
            // 不是代理,只返回user_address和level_agent,其他为空
            one_agent = None;
            two_agents = Vec::new();
            common_agents = Vec::new();
        },  
        _ => {
            error!("Invalid agent level: {}", level_agent);
            return Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid agent level",
                "data": null
            })));
        }
    }

    // Update the response format to match ApiResponse
    let response = json!({
        "status": "success",
        "data": {
            "user_address": user_address,
            "level_agent": level_agent,
            "agent_details": {
                "one_agent": one_agent.map(|agent| {
                    json!({
                        "user_address": agent.user_address,
                        "level_agent": agent.level_agent,
                        "created_at": agent.created_at
                    })
                }),
                "two_agents": two_agents.iter().map(|agent| {
                    json!({
                        "user_address": agent.user_address,
                        "level_agent": agent.level_agent,
                        "created_at": agent.created_at
                    })
                }).collect::<Vec<_>>(),
                "common_agents": common_agents.iter().map(|agent| {
                    json!({
                        "user_address": agent.user_address,
                        "level_agent": agent.level_agent,
                        "created_at": agent.created_at
                    })
                }).collect::<Vec<_>>()
            }
        }
    });

    info!("Successfully retrieved agent details");
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_commission_details(
    db: web::Data<Arc<DatabaseConnection>>,
    query: web::Query<BetHistoryQuery>,  // Reuse BetHistoryQuery for pagination
) -> Result<HttpResponse, ServiceError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let user_address = query.account_address.clone().unwrap_or_default();
    
    info!("Fetching commission details for user: {}", user_address);

    // Get total count
    let total = Commissions::find()
        .filter(commission_model::Column::UserAddress.eq(&user_address))
        .count(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to count commission records: {}", e);
            ServiceError::from(e)
        })?;

    // Query commissions with pagination
    let commissions = Commissions::find()
        .filter(commission_model::Column::UserAddress.eq(&user_address))
        .order_by_desc(commission_model::Column::CreatedAt)
        .paginate(db.as_ref().as_ref(), limit as u64)
        .fetch_page(page as u64 - 1)
        .await
        .map_err(|e| {
            error!("Failed to fetch commission details: {}", e);
            ServiceError::from(e)
        })?;

    // Transform commission records into the required format
    let commission_details: Vec<CommissionDetail> = commissions
        .into_iter()
        .map(|commission| CommissionDetail {
            user_address: commission.user_address,
            from_address: commission.from_address,
            // Format commission to 3 decimal places
            commission: format!("{:.3}", commission.commission),
            transaction_hash: commission.transaction_hash,
            time: commission.created_at,
        })
        .collect();

    // 计算总佣金金额
    let total_amount = Commissions::find()
        .filter(commission_model::Column::UserAddress.eq(&user_address))
        .all(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch records for total commission: {}", e);
            ServiceError::from(e)
        })?
        .iter()
        .map(|record| record.commission)
        .sum::<Decimal>();

    let response = CommissionDetailsResponse {
        user_address,
        commission_details,
        total,
        total_amount: format!("{:.4}", total_amount)
    };

    info!("Successfully retrieved commission details");
    Ok(HttpResponse::Ok().json(response))
}

/// 处理用户投注请求的异步函数
pub async fn place_bet(
    bet_record: web::Json<BetRecord>,
    db: web::Data<Arc<DatabaseConnection>>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse> {
    info!("Recording bet: {:?}", bet_record);

    // Get weekday from config during development
    let weekday = if cfg!(debug_assertions) {
        match config.as_ref().week_today.today.to_lowercase().as_str() {
            "monday" => chrono::Weekday::Mon,
            "tuesday" => chrono::Weekday::Tue,
            "wednesday" => chrono::Weekday::Wed,
            "thursday" => chrono::Weekday::Thu,
            "friday" => chrono::Weekday::Fri,
            "saturday" => chrono::Weekday::Sat,
            "sunday" => chrono::Weekday::Sun,
            _ => {
                Utc::now().weekday()
            }
        }
    } else {
        // Use actual time in production
        Utc::now().weekday()
    };

    info!("Week_today: {:?}", weekday);

    // Create Web3 instance
    let transport = match Http::new(&config.web3.url) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create HTTP transport: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Internal server error"
            })));
        }
    };
    let web3 = Web3::new(transport);

    // Initialize WeekActionService
    let amount_service = Arc::new(AmountService::new(
        Arc::clone(&db),
        web3,
        config.contract.address.clone(),
        config.contract.owner.clone(),
        config.week_action.prize_pool_account.clone()
    ));

    let buy_luck_number_service = Arc::new(BuyLuckNumberService::new(
        Arc::clone(&db),
        Arc::clone(&amount_service)
    ));

    let week_action_service = match WeekActionConfig::from_env("default") {
        Ok(config) => Arc::new(WeekActionService::new(
            Arc::clone(&db),
            Arc::clone(&buy_luck_number_service),
            Arc::clone(&amount_service),
            config
        )),
        Err(e) => {
            error!("Failed to load WeekAction config: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Internal server error"
            })));
        }
    };

    // 1. 创建新的投注记录 ActiveModel
    info!("bet_handler, start create bet record");
    let bet = bet_records::ActiveModel {
        account_address: Set(bet_record.account_address.clone()),
        amount: Set(bet_record.amount.to_string()),
        transaction_hash: Set(bet_record.transaction_hash.clone()),
        block_number: Set(bet_record.block_number),
        block_timestamp: Set(bet_record.block_timestamp),
        created_at: Set(chrono::Utc::now().into()),
        status: Set(BetStatus::Pending.to_string()),
        ..Default::default()
    };

    // 2. 将投注记录插入数据库
    info!("bet_handler, start insert bet record");
    let saved_bet = match bet.insert(db.get_ref().as_ref()).await {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to save bet record: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to save bet record"
            })));
        }
    };

    // 3. 启动交易验证
    info!("bet_handler, start verify transaction");
    let db_clone = Arc::clone(&db);
    let web3_url = config.web3.url.clone();
    let tx_hash = bet_record.transaction_hash.clone();
    
    tokio::spawn(async move {
        match verify_transaction(&tx_hash, web3_url).await {
            Ok(true) => {
                if let Err(e) = update_bet_status(db_clone.as_ref().as_ref(), &tx_hash, BetStatus::Confirmed).await {
                    error!("Failed to update bet status: {}", e);
                }
            },
            Ok(false) => {
                if let Err(e) = update_bet_status(db_clone.as_ref().as_ref(), &tx_hash, BetStatus::Failed).await {
                    error!("Failed to update bet status: {}", e);
                }
                return;
            },
            Err(e) => {
                error!("Failed to verify transaction: {}", e);
                return;
            }
        }

        // 4. 根据星期几调用不同的处理函数
        info!("bet_handler, start parse amount");
        let decimal_amount = match Decimal::from_str(&bet_record.amount.to_string()) {
            Ok(amount) => {
                if amount < Decimal::from_str("0.001").unwrap() {
                    error!("Amount must be at least 0.001 ETH");
                    return;
                }
                amount
            },
            Err(e) => {
                error!("Failed to parse decimal amount: {}", e);
                return;
            }
        };
        info!("bet_handler, decimal_amount: {}", decimal_amount);
        // Convert to Wei using the same pattern as amount_service.rs
        let amount_u256 = match U256::from_dec_str(&(decimal_amount * Decimal::from_str("1000000000000000000").unwrap()).to_string().split('.').next().unwrap()) {
            Ok(amount) => {
                info!("bet_handler, converted amount to U256: {}", amount);
                amount
            },
            Err(e) => {
                error!("Failed to convert to U256: {}", e);
                return;
            }
        };
        info!("bet_handler,end parse amount: {}", amount_u256);

        let week_result = match weekday.num_days_from_monday() {
            0 => { // 周一
                info!("bet_handler Monday, Week_today: {:?}", amount_u256);
                week_action_service.monday_level_one_competition(
                    bet_record.account_address.clone(),
                    amount_u256,
                    bet_record.transaction_hash.clone(),
                ).await
            },
            1 => { // 周二
                week_action_service.tuesday_level_two_competition(
                    bet_record.account_address.clone(),
                    amount_u256,
                    bet_record.transaction_hash.clone(),
                    Some(bet_record.agent_address.clone()),
                ).await
            },
            2 => { // 周三
                week_action_service.wednesday_normal_competition(
                    bet_record.account_address.clone(),
                    amount_u256,
                    bet_record.transaction_hash.clone(),
                    Some(bet_record.agent_address.clone()),
                ).await
            },
            3..=5 => { // 周四到周六
                let decimal_amount = match Decimal::from_str(&bet_record.amount.to_string()) {
                    Ok(amount) => amount,
                    Err(e) => {
                        error!("Failed to parse decimal amount: {}", e);
                        return;
                    }
                };
                week_action_service.process_bet(
                    bet_record.account_address.clone(),
                    amount_u256,
                    bet_record.transaction_hash.clone(),
                    Some(bet_record.agent_address.clone()),
                ).await
            },
            _ => {
                error!("Betting not allowed on this day");
                return;
            }
        };

        if let Err(e) = week_result {
            error!("Failed to process weekly action: {:?}", e);
        }
    });

    // 5. 返回初始响应
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Bet recorded successfully",
        "data": saved_bet
    })))
}

pub async fn get_current_time() -> HttpResponse {
    let local_time = Local::now();
    let response = format!(
        "Current time: {}-{}-{} {}",
        local_time.year(),
        local_time.month(),
        local_time.day(),
        local_time.format("%A")
    );
    HttpResponse::Ok().body(response)
}

#[derive(Debug, Deserialize)]  // 添加 Debug trait
pub struct BetQuery {
    pub from: String,
    pub amount: f64,
    pub number: u64,  // 确保这个字段存在
}

pub async fn get_transaction_params(
    query: web::Query<BetQuery>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse> {
    // 使用 {:#?} 获得更好的格式化输出
    info!("Received bet query: {:#?}", query);
    
    // 单独记录每个参数
    debug!("Processing bet request:");
    debug!("  From address: {}", query.from);
    debug!("  Amount: {} ETH", query.amount);
    debug!("  Bet number: {}", query.number);
    
    let contract_address = &config.contract.address;
    
    // 创建 Web3 实例
    let transport = Http::new(&config.web3.url)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create HTTP transport: {}", e)))?;
    let web3 = Web3::new(transport);
    
    // 创建合约实例
    let contract = Contract::from_json(
        web3.eth(),
        contract_address.parse().unwrap(),
        include_bytes!("../../contracts/LuckGame.json")
    ).map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create contract: {}", e)))?;
    
    // 验证合约和参数
    let from_address = query.from.parse()
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;
    
    match verify_contract(
        &web3,
        contract_address,
        &query.from,
        query.amount
    ).await {
        Ok(true) => {
            // 使用 generate_transaction_params 生成交易参数
            match generate_transaction_params(
                from_address,
                query.amount,
                //query.number,  // 传递下注字
                &contract
            ).await {
                Ok(params) => {
                    debug!("Generated transaction params: {:?}", params);
                    Ok(HttpResponse::Ok().json(params))
                },
                Err(e) => {
                    error!("Failed to generate transaction params: {}", e);
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to generate transaction params: {}", e)
                    })))
                }
            }
        },
        Ok(false) => {
            error!("Contract verification failed");
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Contract verification failed"
            })))
        },
        Err(e) => {
            error!("Error during contract verification: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Error during contract verification: {}", e)
            })))
        }
    }
}

// 添加自定义反序列化函数，将 f64 转换为 Decimal
fn deserialize_f64_to_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let amount_str = String::deserialize(deserializer)?;
    
    // Parse the string amount directly to Decimal
    match Decimal::from_str(&amount_str) {
        Ok(decimal) => {
            // Check minimum amount (0.001)
            if decimal < Decimal::from_str("0.001").unwrap() {
                return Err(serde::de::Error::custom("Amount must be at least 0.001 ETH"));
            }
            Ok(decimal)
        },
        Err(e) => Err(serde::de::Error::custom(format!("Failed to parse amount: {}", e)))
    }
}

pub async fn generate_transaction_params(
    from: H160,
    amount: f64,
    contract: &Contract<Http>
) -> Result<TransactionParams, Box<dyn std::error::Error>> {
    // 将ETH金额转换为Wei
    let value = U256::from((amount * 1e18) as u64);
    
    // 编码合约调用数据
    let data = contract
        .abi()
        .function("placeBet")?
        .encode_input(&[ethers::abi::Token::Uint(1.into())])?;

    Ok(TransactionParams {
        to: contract.address().to_string(),
        value: format!("0x{:x}", value),
        data: format!("0x{}", hex::encode(&data)),
    })
}

pub async fn record_bet(
    bet_record: web::Json<BetRecord>,
    db: web::Data<Arc<DatabaseConnection>>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse> {
    let db_clone = Arc::clone(&db);
    let web3_url = config.web3.url.clone();
    let tx_hash = bet_record.transaction_hash.clone();
    // 验证交易
    tokio::spawn(async move {
        match verify_transaction(&tx_hash, web3_url).await {
            Ok(true) => {
                if let Err(e) = update_bet_status(db_clone.as_ref().as_ref(), &tx_hash, BetStatus::Confirmed).await {
                    error!("Failed to update bet status: {}", e);
                }
            },
            Ok(false) => {
                if let Err(e) = update_bet_status(db_clone.as_ref().as_ref(), &tx_hash, BetStatus::Failed).await {
                    error!("Failed to update bet status: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to verify transaction: {}", e);
            }
        }
    });

    info!("Recording bet: {:?}", bet_record);
    // 创建新的投注记录 ActiveModel
    let bet = bet_records::ActiveModel {
        account_address: Set(bet_record.account_address.clone()),
        amount: Set(bet_record.amount.to_string()),
        transaction_hash: Set(bet_record.transaction_hash.clone()),
        block_number: Set(bet_record.block_number),
        block_timestamp: Set(bet_record.block_timestamp),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    // 将投注记录插入数据库
    match bet.insert(db.get_ref().as_ref()).await {
        Ok(result) => {
            info!("Bet recorded successfully: {:?}", result);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": result
            })))
        }
        Err(e) => {
            error!("Failed to record bet: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to record bet: {}", e)
            })))
        }
    }
}

pub async fn update_bet_status(
    db: &DatabaseConnection,
    tx_hash: &str,
    status: BetStatus,
) -> Result<(), DbErr> {
    // 查找应的投注记录
    let bet = BetRecords::find()
        .filter(bet_records::Column::TransactionHash.eq(tx_hash))
        .one(db)
        .await?;

    if let Some(bet) = bet {
        // 创建 ActiveModel 用于更新
        let mut bet_active: bet_records::ActiveModel = bet.into();
        
        // 置新状态
        bet_active.status = Set(status.to_string());
        bet_active.updated_at = Set(Utc::now());

        // 执行更新
        bet_active.update(db).await?;
        
        info!("Updated bet status: tx_hash={}, status={:?}", tx_hash, status);
    } else {
        error!("Bet record not found for tx_hash: {}", tx_hash);
        return Err(DbErr::Custom("Bet record not found".to_string()));
    }

    Ok(())
}

// First, define a custom error type that is Send + Sync
#[derive(Debug)]
pub enum TransactionError {
    Web3Error(String),
    InvalidHash(String),
    VerificationFailed(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web3Error(msg) => write!(f, "Web3 error: {}", msg),
            Self::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
            Self::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {}

// Update verify_transaction to use our custom error type
pub async fn verify_transaction(
    tx_hash: &str,
    web3_url: String,
) -> Result<bool, TransactionError> {
    let transport = Http::new(&web3_url)
        .map_err(|e| TransactionError::Web3Error(e.to_string()))?;
    let web3 = Web3::new(transport);

    let tx_hash = H256::from_str(tx_hash)
        .map_err(|e| TransactionError::InvalidHash(e.to_string()))?;

    let receipt = web3.eth().transaction_receipt(tx_hash).await
        .map_err(|e| TransactionError::Web3Error(e.to_string()))?;

    match receipt {
        Some(receipt) => {
            // 检查交易状态
            match receipt.status {
                Some(status) => Ok(status.as_u64() == 1),
                None => Ok(false)
            }
        },
        None => Ok(false)
    }
}

fn deserialize_string_to_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let amount_str = String::deserialize(deserializer)?;
    
    match Decimal::from_str(&amount_str) {
        Ok(decimal) => {
            // Validate minimum amount
            if decimal < Decimal::from_str("0.001").unwrap() {
                return Err(serde::de::Error::custom("Amount must be at least 0.001 ETH"));
            }
            Ok(decimal)
        },
        Err(e) => Err(serde::de::Error::custom(format!("Failed to parse amount: {}", e)))
    }
}



pub async fn get_agents(
    db: web::Data<Arc<DatabaseConnection>>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse, ServiceError> {
    // Get weekday from config during development
    let weekday = if cfg!(debug_assertions) {
        match config.week_today.today.to_lowercase().as_str() {
            "monday" => chrono::Weekday::Mon,
            "tuesday" => chrono::Weekday::Tue,
            "wednesday" => chrono::Weekday::Wed,
            "thursday" => chrono::Weekday::Thu,
            "friday" => chrono::Weekday::Fri,
            "saturday" => chrono::Weekday::Sat,
            "sunday" => chrono::Weekday::Sun,
            _ => {
                Utc::now().weekday()
            }
        }
    } else {
        // Use actual time in production
        Utc::now().weekday()
    };

    info!("Week_today: {:?}", weekday);
    
    info!("Received request for get_agents");
    let level_agent = match weekday.num_days_from_monday() {
        1 => "one", // 周二
        2 => "two", // 周三
        3..=5 => "common", // 周四到周六
        _ => "unknown"
    };
    info!("level_agent: {:?}", level_agent);
    let agents = Agents::find()
        .filter(agent_model::Column::LevelAgent.eq(level_agent))
        .all(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Database error when fetching agents: {:?}", e);
            ServiceError::from(e)
        })?;
    
    info!("Found {} level one agents", agents.len());
    
    // 构造标准响应格式
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": agents.into_iter().map(|agent| {
            json!({
                "id": agent.id,
                "userAddress": agent.user_address,
                "levelAgent": agent.level_agent
            })
        }).collect::<Vec<_>>()
    })))
}

pub async fn get_prize_pool_expectation(
    db: web::Data<Arc<DatabaseConnection>>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse, ServiceError> {
    info!("Fetching total prize pool amount and prize expection");
    let contract_address = config.contract.address.clone();
    // Calculate total amount from platform_prize_pool table
    let total_amount = PlatformPrizePool::find()
        .filter(platform_prize_pool_model::Column::UserAddress.eq(contract_address.clone()))
        .all(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch prize pool records: {}", e);
            ServiceError::from(e)
        })?
        .iter()
        .fold(Decimal::from(0), |acc, record| acc + record.amount);

    
    let prize_distribution = config.prize_distribution.clone();

    // Convert f64 percentages to Decimal
    let user_pool_percentage = Decimal::from_f64(prize_distribution.user_pool_percentage)
        .unwrap_or(Decimal::new(0, 0));
    let first_prize_percentage = Decimal::from_f64(prize_distribution.first_prize_percentage)
        .unwrap_or(Decimal::new(0, 0));
    let second_prize_percentage = Decimal::from_f64(prize_distribution.second_prize_percentage)
        .unwrap_or(Decimal::new(0, 0));
    let third_prize_percentage = Decimal::from_f64(prize_distribution.third_prize_percentage)
        .unwrap_or(Decimal::new(0, 0));

    // Calculate prize pools using Decimal multiplication
    let user_prize_pool = total_amount * user_pool_percentage;
    let first_prize = user_prize_pool * first_prize_percentage;
    let second_prize = user_prize_pool * second_prize_percentage;
    let third_prize = user_prize_pool * third_prize_percentage;

    info!("first_prize: {} ETH, second_prize: {} ETH, third_prize: {} ETH", first_prize, second_prize, third_prize);

    // 计算代理人中奖金额
    let remaining_prize_pool = total_amount - user_prize_pool; // 剩余奖金池
    let level_one_agent_percentage = Decimal::from_f64(prize_distribution.level_one_agent_percentage)
        .unwrap_or(Decimal::new(0, 0));
    let level_two_agent_percentage = Decimal::from_f64(prize_distribution.level_two_agent_percentage)
        .unwrap_or(Decimal::new(0, 0));
    let level_one_agent_prize = remaining_prize_pool * level_one_agent_percentage; // 一级代理 20%
    let level_two_agent_prize = remaining_prize_pool * level_two_agent_percentage; // 二级代理 10%
    let common_agent_prize =
        remaining_prize_pool - (level_one_agent_prize + level_two_agent_prize); // 剩余金额
    info!("level_one_agent_prize: {} ETH, level_two_agent_prize: {} ETH, common_agent_prize: {} ETH", level_one_agent_prize, level_two_agent_prize, common_agent_prize);

    // Format the response with 2 decimal places
    let response = PrizePoolPrizeExpectionResponse {
        prize_amount: format!("{:.4}", total_amount),
        first_prize: format!("{:.6}", first_prize),
        second_prize: format!("{:.6}", second_prize),
        third_prize: format!("{:.6}", third_prize),
        level_one_agent_prize: format!("{:.6}", level_one_agent_prize),
        level_two_agent_prize: format!("{:.6}", level_two_agent_prize),
        common_agent_prize: format!("{:.6}", common_agent_prize),
    };
    info!("Successfully retrieved prize pool amount: {}", response.prize_amount);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_competition_lottery_info(
    db: web::Data<Arc<DatabaseConnection>>,
    query: web::Query<AgentQuery>,
    config: web::Data<Arc<AppConfig>>,
) -> Result<HttpResponse, ServiceError> {
    let user_address = query.account_address.clone();
    info!("Fetching competition and lottery info for user: {}", user_address);

    // Get weekday from config during development
    let weekday = if cfg!(debug_assertions) {
        match config.week_today.today.to_lowercase().as_str() {
            "monday" => chrono::Weekday::Mon,
            "tuesday" => chrono::Weekday::Tue,
            "wednesday" => chrono::Weekday::Wed,
            "thursday" => chrono::Weekday::Thu,
            "friday" => chrono::Weekday::Fri,
            "saturday" => chrono::Weekday::Sat,
            "sunday" => chrono::Weekday::Sun,
            _ => {
                Utc::now().weekday()
            }
        }
    } else {
        // Use actual time in production
        Utc::now().weekday()
    };

    info!("Week_today: {:?}", weekday);
    
    // Determine competition topic based on weekday
    let topic = match weekday.num_days_from_monday() {
        0 => "One Agent Competition for Top 100",
        1 => "Two Agent Competition for Top 1000",
        2 => "Common Agent Competition for greater than 0.1 ETH",
        3..=5 => "Bet Amount for Luck Prize",
        _ => "No Competition Today",
    };

    // Get user's rank based on bet amount
    // 周一到周六，获取not_agent的用户的总数，获取当前用户的投注总金额的排名rank的值，agents表中取都是level_agent=not_agent的。可以参考buy_luck_number的原始SQL的写法。从bet_records和agents表获取数据。
    // 周日，不进行排序。
    let rankings = BetRecords::find()
        .join(JoinType::InnerJoin, bet_records::Relation::Agents.def())
        .filter(agent_model::Column::LevelAgent.eq("not_agent"))
        .select_only()
        .column(bet_records::Column::AccountAddress)
        .group_by(bet_records::Column::AccountAddress)
        .column_as(
            Expr::expr(Func::cast_as(
                Expr::col(bet_records::Column::Amount),
                Alias::new("DECIMAL")
            )).sum(),
            "total_amount"
        )
        .order_by_desc(Expr::cust("total_amount"))
        .into_tuple::<(String, Decimal)>()
        .all(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch rankings: {}", e);
            ServiceError::from(e)
        })?;

    let competition_count = rankings.len() as i64;

    let rank = rankings.into_iter()
        .enumerate()
        .find(|(_, (address, _))| address == &user_address)
        .map(|(index, _)| index as i64 + 1)
        .unwrap_or(0);

    // Get lottery information if it's Sunday
    let lottery_info = if weekday == chrono::Weekday::Sun {
        LotteryDistributionDetail::find()
            .order_by_desc(lottery_distribution_detail_model::Column::CreatedAt)
            .limit(6)
            .all(db.as_ref().as_ref())
            .await
            .map_err(|e| {
                error!("Failed to fetch lottery info: {}", e);
                ServiceError::from(e)
            })?
            .into_iter()
            .map(|record| LotteryInfo {
                user_address: record.user_address,
                prize_grade: record.prize_grade,
                luck_number: record.luck_number,
                prize_amount: record.prize_amount.to_f64().unwrap_or(0.0),
                time: record.created_at,
            })
            .collect()
    } else {
        Vec::new()
    };

    let response = CompetitionLotteryResponse {
        competition: CompetitionInfo {
            user_address,
            topic: topic.to_string(),
            competition_count,
            rank,
        },
        lottery: lottery_info,
    };

    info!("Successfully retrieved competition and lottery info");
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_datetime_week() -> HttpResponse {
    // 获取UTC时间
    let utc_now = Utc::now();
    // 定义支持的时区
    let timezones = vec![
        ("UTC", utc_now.with_timezone(&Tz::UTC)),
        ("Asia/Shanghai", utc_now.with_timezone(&Tz::Asia__Shanghai)),
        ("Asia/Singapore", utc_now.with_timezone(&Tz::Asia__Singapore)),
        ("Asia/Tokyo", utc_now.with_timezone(&Tz::Asia__Tokyo)),
        ("America/New_York", utc_now.with_timezone(&Tz::America__New_York)),
        ("Europe/London", utc_now.with_timezone(&Tz::Europe__London))
    ];

    // 构建每个时区的时间信息
    let timezone_info: Vec<serde_json::Value> = timezones.iter()
        .map(|(zone_name, time)| {
            json!({
                "timezone": zone_name,
                "timezone_offset": match *zone_name {
                    "UTC" => "+00:00",
                    "Asia/Shanghai" => "+08:00",
                    "Asia/Singapore" => "+08:00",
                    "Asia/Tokyo" => "+09:00",
                    "America/New_York" => "-05:00", // 注意: 这会随夏令时变化
                    "Europe/London" => "+00:00", // 注意: 这会随夏令时变化
                    _ => "Unknown"
                },
                "year": time.year(),
                "month": time.month(),
                "day": time.day(),
                "hour": time.hour(),
                "minute": time.minute(),
                "second": time.second(),
                "weekday": time.weekday().number_from_monday(),
                "weekday_cn": match time.weekday().number_from_monday() {
                    1 => "星期一",
                    2 => "星期二",
                    3 => "星期三",
                    4 => "星期四",
                    5 => "星期五",
                    6 => "星期六",
                    7 => "星期日",
                    _ => "未知"
                },
                "weekday_en": time.weekday().to_string(),
                "datetime": time.format("%Y-%m-%d %H:%M:%S").to_string(),
                "timestamp": time.timestamp()
            })
        })
        .collect();

    // 构建响应JSON
    let response = json!({
        "current_utc": utc_now.format("%Y-%m-%d %H:%M:%S").to_string(),
        "timezones": timezone_info
    });

    HttpResponse::Ok().json(response)
}

