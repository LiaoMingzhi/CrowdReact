// src/services/week_action_service.rs
use crate::models::{
    agent_model::{self, Model as Agent},
    buy_luck_number_model::{self, Model as LuckNumber},
};
use crate::services::auth_service::AuthError;
use crate::services::buy_luck_number_service::BuyLuckNumberServiceTrait;
use crate::services::{
    agent_service::AgentService,
    amount_service::AmountService,
    buy_luck_number_service::{BuyLuckNumberResult, BuyLuckNumberService},
};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use chrono_tz::Asia::Shanghai;
use chrono_tz::Tz;
use config::{Config, ConfigError, Environment, File};
use rand::prelude::SliceRandom;
use sea_orm::prelude::Decimal;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use log::{info, error};
use rust_decimal::prelude::FromPrimitive;
use tokio_cron_scheduler::{Job, JobScheduler};
use web3::types::{H160, U256};
use crate::routes::config;

#[derive(Clone)]
pub struct WeekActionService {
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
    agent_service: AgentService,
    platform_account: String,   // 平台账户地址
    prize_pool_account: String, // 奖金池账户地址
    owner_account: String,
    config: WeekActionConfig,
}

// 定义统一的返回类型
#[derive(Debug, Serialize)]
pub struct BetResult {
    pub success: bool,
    pub message: String,
    pub result: BuyLuckNumberResult,
    pub additional_info: Option<serde_json::Value>,
}

// 添加配置结构体
#[derive(Debug, Clone)]
pub struct WeekActionConfig {
    pub platform_account: String,
    pub prize_pool_account: String,
    pub owner_account: String,
    pub prize_distribution: PrizeDistribution,
}

#[derive(Debug, Clone)]
pub struct PrizeDistribution {
    pub user_pool_percentage: f64,       // 普通用户奖池百分比
    pub first_prize_percentage: f64,     // 一等奖百分比
    pub second_prize_percentage: f64,    // 二等奖百分比
    pub third_prize_percentage: f64,     // 三等奖百分比
    pub level_one_agent_percentage: f64, // 一级代理奖励百分比
    pub level_two_agent_percentage: f64, // 二级代理奖励百分比
}

impl WeekActionConfig {
    pub fn from_env(env: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", env)))
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        Ok(Self {
            platform_account: config.get_string("week_action.platform_account")?,
            prize_pool_account: config.get_string("week_action.prize_pool_account")?,
            owner_account: config.get_string("contract.owner")?,
            prize_distribution: PrizeDistribution {
                user_pool_percentage: config.get_float("prize_distribution.user_pool_percentage")?,
                first_prize_percentage: config.get_float("prize_distribution.first_prize_percentage")?,
                second_prize_percentage: config.get_float("prize_distribution.second_prize_percentage")?,
                third_prize_percentage: config.get_float("prize_distribution.third_prize_percentage")?,
                level_one_agent_percentage: config.get_float("prize_distribution.level_one_agent_percentage")?,
                level_two_agent_percentage: config.get_float("prize_distribution.level_two_agent_percentage")?,
            },
        })
    }
}

impl Default for WeekActionConfig {
    fn default() -> Self {
        Self {
            platform_account: "0xDefaultPlatformAccount".to_string(),
            prize_pool_account: "0xDefaultPrizePoolAccount".to_string(),
            owner_account: "0xDefaultOwnerAccount".to_string(),
            prize_distribution: PrizeDistribution {
                user_pool_percentage: 0.7,       
                first_prize_percentage: 0.5,     
                second_prize_percentage: 0.3,  
                third_prize_percentage: 0.2,     
                level_one_agent_percentage: 0.15, 
                level_two_agent_percentage: 0.1,
            },
        }
    }
}

impl Default for PrizeDistribution {
    fn default() -> Self {
        Self {
            user_pool_percentage: 0.7,
            first_prize_percentage: 0.5,
            second_prize_percentage: 0.3,
            third_prize_percentage: 0.2,
            level_one_agent_percentage: 0.15,
            level_two_agent_percentage: 0.1,
        }
    }
}

impl WeekActionService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        buy_luck_number_service: Arc<BuyLuckNumberService>,
        amount_service: Arc<AmountService>,
        config: WeekActionConfig,
    ) -> Self {
        let agent_service = AgentService::new(Arc::clone(&db));
        
        Self {
            db,
            buy_luck_number_service,
            amount_service,
            agent_service,
            platform_account: config.platform_account.clone(),
            prize_pool_account: config.prize_pool_account.clone(),
            owner_account: config.owner_account.clone(),
            config,
        }
    }

    pub async fn create(
        db: Arc<DatabaseConnection>,
        buy_luck_number_service: Arc<BuyLuckNumberService>,
        amount_service: Arc<AmountService>,
        env: &str,
    ) -> Result<Self, AuthError> {
        let config =
            WeekActionConfig::from_env(env).map_err(|e| AuthError::ConfigError(e.to_string()))?;

        Ok(Self::new(
            db,
            buy_luck_number_service,
            amount_service,
            config,
        ))
    }

    // 周一：一级代理竞争
    pub async fn monday_level_one_competition(
        &self,
        user_address: String,
        amount: U256, // 以 Wei 为单位的金额
        transaction_hash: String,
    ) -> Result<BetResult, DbErr> {
        info!("monday_level_one_competition, user_address: {}, amount: {}", user_address, amount);
        // 1. 分配金额
        // Convert U256 amount to Decimal for validation (in ETH)
        let decimal_amount = Decimal::from_str(&format!("{}", amount))
            .map_err(|e| DbErr::Custom(format!("Failed to parse amount: {}", e)))?;
        let eth_amount = decimal_amount / Decimal::from(10_u64.pow(18));
        
        info!("monday_level_one_competition, eth_amount: {}", eth_amount);// 以 Eth 为单位的金额

        // Validate minimum amount (0.001 ETH)
        let min_amount = Decimal::from_str("0.001")
            .map_err(|e| DbErr::Custom(format!("Failed to parse minimum amount: {}", e)))?;
        if eth_amount < min_amount {
            return Err(DbErr::Custom("Amount must be at least 0.001 ETH".to_string()));
        }

        // Calculate distribution amounts (in Wei)
        let platform_amount = amount * U256::from(40) / U256::from(100);    // 20% to platform
        // let owner_amount = amount * U256::from(10) / U256::from(100);   // 10% to owner
        let pool_amount = amount - platform_amount;
        
        info!("monday_level_one_competition, platform_amount: {}", platform_amount);
        info!("monday_level_one_competition, pool_amount: {}", pool_amount);

        // Convert U256 amounts to Decimal (keeping them in Wei)
        let platform_decimal = Decimal::from_str(&platform_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert platform amount: {}", e)))?;
        // let owner_decimal = Decimal::from_str(&owner_amount.to_string())
        //     .map_err(|e| DbErr::Custom(format!("Failed to convert owner amount: {}", e)))?;
        let pool_decimal = Decimal::from_str(&pool_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert pool amount: {}", e)))?;

        // Process transfers
        let platform_tx = self.amount_service
            .process_regular_bet(&self.platform_account, platform_decimal)
            .await?;
        // let owner_tx = self.amount_service
        //     .process_regular_bet(&self.owner_account, owner_decimal)
        //     .await?;

        // 记录平台资金流
        self.amount_service.record_platform_funds_flow(
            self.platform_account.clone(),
            user_address.clone(),
            platform_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;

        // 记录平台奖金池
        self.amount_service.record_platform_prize_pool(
            user_address.clone(),
            pool_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;

        // 2. 获取所有幸运号码
        let numbers = self.buy_luck_number_service.get_all_luck_numbers(user_address, transaction_hash.clone() ,amount).await?;
        
        let luck_numbers = BuyLuckNumberResult {
            numbers: numbers,
            transaction_hash: Some(transaction_hash),
            total_amount: eth_amount,
        };
        
        Ok(BetResult {
            success: true,
            message: "Monday level one competition processed successfully".to_string(),
            result: luck_numbers,
            additional_info: Some(json!({
                "platform_tx": platform_tx,
                "owner_tx": platform_tx
            })),
        })
    }

    // 周二：确认一级代理人名单
    pub async fn confirm_level_one_agents(&self) -> Result<(), DbErr> {
        // 获取当前时间
        // let now = self.get_current_time();
        // let tuesday_start = self.get_tuesday_start();
        // let wednesday_start = self.get_wednesday_start();
        // 
        // // 检查是否在周二的0点0分0秒
        // if now < tuesday_start || now >= wednesday_start {
        //     return Err(DbErr::Custom(
        //         "Freeze operation only available on Tuesday at 00:00:00".to_string(),
        //     ));
        // }
        
        // 计算前100名投注最多的用户地址，如果排名相同的，则选择时间最早的。
        let top_100_user_address = self.buy_luck_number_service.get_top_100_users().await?;
        //info!("top_100_user_address: {:?}", top_100_user_address);
        let top_100_user_address_count = top_100_user_address.len();
        info!("top_100_user_address_count: {}", top_100_user_address_count);
        // 确认并冻结更新一级代理人名单
        for user_address in top_100_user_address {
            self.agent_service
                .freeze_level_one_agent(user_address)
                .await?;
        }
        
        Ok(())
    }

    // 周二：二级代理竞争
    pub async fn tuesday_level_two_competition(
        &self,
        user_address: String,
        amount: U256,
        transaction_hash: String,
        specified_level_one_agent: Option<String>,
    ) -> Result<BetResult, DbErr> {
        info!("tuesday_level_two_competition, specified_level_one_agent: {:?}", specified_level_one_agent);
        info!("tuesday_level_two_competition, user_address: {}, amount: {}", user_address, amount);

        // Convert amount to Decimal for validation (in ETH)
        let decimal_amount = Decimal::from_str(&format!("{}", amount))
            .map_err(|e| DbErr::Custom(format!("Failed to parse amount: {}", e)))?;
        let eth_amount = decimal_amount / Decimal::from(10_u64.pow(18));

        // Calculate distribution amounts (in Wei)
        let platform_amount = amount * U256::from(40) / U256::from(100); // 20% to platform
        //let owner_amount = amount * U256::from(10) / U256::from(100);   // 10% to owner
        let agent_amount = amount * U256::from(20) / U256::from(100);   // 20% to level one agent
        let pool_amount = amount - platform_amount - agent_amount;       // Remainder to prize pool

        // Convert U256 amounts to Decimal (keeping them in Wei)
        let platform_decimal = Decimal::from_str(&platform_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert platform amount: {}", e)))?;
        // let owner_decimal = Decimal::from_str(&owner_amount.to_string())
        //     .map_err(|e| DbErr::Custom(format!("Failed to convert owner amount: {}", e)))?;
        let agent_decimal = Decimal::from_str(&agent_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert agent amount: {}", e)))?;
        let pool_decimal = Decimal::from_str(&pool_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert pool amount: {}", e)))?;

        // Process transfers
        let platform_tx = self.amount_service
            .process_regular_bet(&self.platform_account, platform_decimal)
            .await?;

        // let owner_tx = self.amount_service
        //     .process_regular_bet(&self.owner_account, owner_decimal)
        //     .await?;

        // 记录平台资金流
        self.amount_service.record_platform_funds_flow(
            self.platform_account.clone(),
            user_address.clone(),
            platform_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;

        // 记录平台奖金池
        self.amount_service.record_platform_prize_pool(
            user_address.clone(),
            pool_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;    

        let mut agent_tx = None;
        if let Some(agent_address) = specified_level_one_agent.clone() {
            // Verify if the specified address is a level one agent
            if self.agent_service.is_level_one_agent(&agent_address).await? {
                agent_tx = Some(self.amount_service
                    .process_regular_bet(&agent_address, agent_decimal)
                    .await?);
            } else {
                info!("Invalid level one agent address: {}", agent_address);
            }
        }

        // Record commission distribution (updated parameters)
        if let Some(agent_address) = &specified_level_one_agent {
            // Verify if the specified address is a level one agent
            if self.agent_service.is_level_one_agent(&agent_address).await? {
                self.amount_service.record_commission(
                    agent_address.clone(),  // 代理人地址（接收佣金的地址）
                    user_address.clone(),   // 用户地址（佣金来源地址）
                    agent_decimal / Decimal::from(10_u64.pow(18)), // 转换为 ETH 单位
                    transaction_hash.clone(),
                ).await?;
            } else {
                info!("Invalid level one agent address: {}", agent_address);
            }
        }

        // Get lucky numbers
        let numbers = self.buy_luck_number_service
            .get_all_luck_numbers(user_address, transaction_hash.clone(), amount)
            .await?;

        Ok(BetResult {
            success: true,
            message: "Tuesday level two competition processed successfully".to_string(),
            result: BuyLuckNumberResult {
                numbers,
                transaction_hash: Some(transaction_hash),
                total_amount: eth_amount,
            },
            additional_info: Some(json!({
                "platform_tx": platform_tx,
                "agent_tx": agent_tx,
                "owner_tx": platform_tx
            })),
        })
    }

    // 周三，确认二级代理名单
    pub async fn confirm_level_two_agents(&self) -> Result<(), DbErr>{
        // 除了一级代理人，计算前1000名投注最多的用户地址，如果排名相同的，则选择时间最早的。
        let top_1000_user_address = self.buy_luck_number_service.get_top_1000_users().await?;
        // Get all level one agents
        let level_one_agents = agent_model::Entity::find()
            .filter(agent_model::Column::LevelAgent.eq("one"))
            .filter(agent_model::Column::IsFrozen.eq(true))  // Only get frozen (confirmed) agents
            .order_by_asc(agent_model::Column::CreatedAt)    // Order by creation time
            .all(&*self.db)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch level one agents: {}", e)))?;

        if level_one_agents.is_empty() {
            // return Err(DbErr::Custom("No confirmed level one agents available".to_string()));
            info!("No confirmed level one agents available");
        } else {
            info!("level_one_agents: {:?}", level_one_agents);
        }

        let agent_count = level_one_agents.len();
        info!("agent_count: {}", agent_count);
        let top_1000_user_address_count = top_1000_user_address.len();
        info!("top_1000_user_address_count: {}", top_1000_user_address_count);
        // 确认并冻结更新二级代理人名单
        for (index, user_address) in top_1000_user_address.into_iter().enumerate() {
            // Round-robin selection of level one agent
            if agent_count > 0 {    
                let agent_index = index % agent_count;
                let selected_level_one = &level_one_agents[agent_index];

                self.agent_service
                    .freeze_level_two_agent(user_address, selected_level_one.user_address.clone())
                    .await?;
            } else {
                info!("No confirmed level one agents available");
                self.agent_service
                    .freeze_level_two_agent(user_address, String::new())
                    .await?;
            }
        }

        Ok(())
    }

    // 周三：普通代理竞争
    pub async fn wednesday_normal_competition(
        &self,
        user_address: String,
        amount: U256,
        transaction_hash: String,
        specified_level_two_agent: Option<String>,
    ) -> Result<BetResult, DbErr> {
        info!("wednesday_normal_competition, specified_level_two_agent: {:?}", specified_level_two_agent);
        info!("wednesday_normal_competition, user_address: {}, amount: {}", user_address, amount);

        // Convert amount to Decimal for validation (in ETH)
        let decimal_amount = Decimal::from_str(&format!("{}", amount))
            .map_err(|e| DbErr::Custom(format!("Failed to parse amount: {}", e)))?;
        let eth_amount = decimal_amount / Decimal::from(10_u64.pow(18));

        // Calculate distribution amounts (in Wei)
        let platform_amount = amount * U256::from(40) / U256::from(100);    // 20% to platform
        //let owner_amount = amount * U256::from(10) / U256::from(100);      // 10% to owner
        let level_one_amount = amount * U256::from(20) / U256::from(100);   // 20% to level one agent
        let level_two_amount = amount * U256::from(20) / U256::from(100);   // 20% to level two agent
        let pool_amount = amount - platform_amount - level_one_amount - level_two_amount; // Remainder to prize pool

        // Convert U256 amounts to Decimal (keeping them in Wei)
        let platform_decimal = Decimal::from_str(&platform_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert platform amount: {}", e)))?;
        // let owner_decimal = Decimal::from_str(&owner_amount.to_string())
        //     .map_err(|e| DbErr::Custom(format!("Failed to convert owner amount: {}", e)))?;
        let level_one_decimal = Decimal::from_str(&level_one_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert level one amount: {}", e)))?;
        let level_two_decimal = Decimal::from_str(&level_two_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert level two amount: {}", e)))?;
        let pool_decimal = Decimal::from_str(&pool_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert pool amount: {}", e)))?;

        // Process platform transfer
        let platform_tx = self.amount_service
            .process_regular_bet(&self.platform_account, platform_decimal)
            .await?;

        // let owner_tx = self.amount_service
        //     .process_regular_bet(&self.owner_account, owner_decimal)
        //     .await?;

        // 记录平台资金流
        self.amount_service.record_platform_funds_flow(
            self.platform_account.clone(),
            user_address.clone(),
            platform_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;

        // 记录平台奖金池
        self.amount_service.record_platform_prize_pool(
            user_address.clone(),
            pool_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;    

        // Process level two agent transfer and get level one agent
        let (level_two_tx, level_one_tx) = if let Some(level_two_address) = specified_level_two_agent.clone() {
            info!("Processing commission for level two agent: {}", level_two_address);
            
            // Verify if the specified address is a level two agent
            if !self.agent_service.is_level_two_agent(&level_two_address).await? {
                info!("Invalid level two agent address: {}", level_two_address);
            }

            let level_one_address = match self.agent_service
                .get_level_one_agent_for_level_two(&level_two_address)
                .await? {
                    Some(agent) => agent,
                    None => {
                        info!("Level one agent not found for level two agent: {}", level_two_address);
                        String::new()
                    }
            };

            info!("Associated level one agent: {}", level_one_address);

            let l2_tx = if !level_two_address.is_empty() {
                self.amount_service
                    .process_regular_bet(&level_two_address, level_two_decimal)
                    .await?
            } else {
                String::new()
            };
            let l1_tx = if !level_one_address.is_empty() {
                self.amount_service
                    .process_regular_bet(&level_one_address, level_one_decimal)
                    .await?
            } else {
                String::new()
            };
            
            
            // Record level two commission
            if !level_two_address.is_empty() {
                match self.amount_service.record_commission(
                    level_two_address.clone(),
                    user_address.clone(),
                level_two_decimal / Decimal::from(10_u64.pow(18)),
                    transaction_hash.clone(),
                ).await {
                    Ok(_) => info!("Successfully recorded level two commission"),
                    Err(e) => {
                        error!("Failed to record level two commission: {}", e);
                        return Err(e);
                    }
                }
            }

            // Record level one commission
            if !level_one_address.is_empty() {
                match self.amount_service.record_commission(
                    level_one_address.clone(),
                    user_address.clone(), 
                    level_one_decimal / Decimal::from(10_u64.pow(18)),
                    transaction_hash.clone(),
            ).await {
                Ok(_) => info!("Successfully recorded level one commission"),
                Err(e) => {
                        error!("Failed to record level one commission: {}", e);
                        return Err(e);
                    }
                }
            }
            
            (Some(l2_tx), Some(l1_tx))
        } else {
            (None, None)
        };

        // Get lucky numbers
        let numbers = self.buy_luck_number_service
            .get_all_luck_numbers(user_address, transaction_hash.clone(), amount)
            .await?;

        Ok(BetResult {
            success: true,
            message: "Wednesday normal competition processed successfully".to_string(),
            result: BuyLuckNumberResult {
                numbers,
                transaction_hash: Some(transaction_hash),
                total_amount: eth_amount,
            },
            additional_info: Some(json!({
                "platform_tx": platform_tx,
                "level_one_tx": level_one_tx,
                "level_two_tx": level_two_tx,
                "owner_tx": platform_tx
            })),
        })
    }

    // 周四，确认普通代理人名单
    pub async fn confirm_level_common_agents(&self) -> Result<(), DbErr> {
        info!("start confirm_level_common_agents");
        // 除了一级代理人和二级代理人，计算投注总金额大于1ETH的账号地址。
        let common_agent_user_address = self.buy_luck_number_service.get_common_agent_users().await?;
        // info!("common_agent_user_address: {:?}", common_agent_user_address);
        // 获取所有的二级代理人
        let level_two_agents = agent_model::Entity::find()
            .filter(agent_model::Column::LevelAgent.eq("two"))
            .filter(agent_model::Column::IsFrozen.eq(true))  // Only get frozen (confirmed) agents
            .order_by_asc(agent_model::Column::CreatedAt)    // Order by creation time
            .all(&*self.db)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to fetch level two agents: {}", e)))?;

        if level_two_agents.is_empty() {
            //return Err(DbErr::Custom("No confirmed level two agents available".to_string()));
            info!("No confirmed level two agents available");
        }

        let agent_count = level_two_agents.len();
        info!("agent_count: {}", agent_count);
        let common_agent_user_address_count = common_agent_user_address.len();
        info!("common_agent_user_address_count: {}", common_agent_user_address_count);
        // 确认并冻结更新普通代理人名单，使用轮询分配二级代理
        for (index, user_address) in common_agent_user_address.into_iter().enumerate() {
            // Round-robin selection of level two agent
            if agent_count > 0 {
                let agent_index = index % agent_count;
                let selected_level_two = &level_two_agents[agent_index];

            self.agent_service
                    .freeze_level_common_agent(user_address, selected_level_two.user_address.clone())
                    .await?;
            } else {
                info!("No confirmed level two agents available");
                self.agent_service
                    .freeze_level_common_agent(user_address, String::new())
                    .await?;
            }
        }

        Ok(())
    }

    // 周四到周六处理普通投注
    pub async fn process_bet(
        &self,
        user_address: String,
        amount: U256,
        transaction_hash: String,
        specified_common_agent: Option<String>,
    ) -> Result<BetResult, DbErr> {
        info!("process_bet, specified_common_agent: {:?}", specified_common_agent);
        info!("process_bet, user_address: {}, amount: {}", user_address, amount);

        // Convert amount to Decimal for validation (in ETH)
        let decimal_amount = Decimal::from_str(&format!("{}", amount))
            .map_err(|e| DbErr::Custom(format!("Failed to parse amount: {}", e)))?;
        let eth_amount = decimal_amount / Decimal::from(10_u64.pow(18));

        // 1. Get agents
        let (common_agent, level_two_agent, level_one_agent) = if let Some(common_agent_address) = specified_common_agent {
            // Verify if the specified address is a common agent
            if !self.agent_service.is_common_agent(&common_agent_address).await? {
                info!("Invalid common agent address: {}", common_agent_address);
            }

            let level_two_agent = match self.agent_service
                .get_level_two_agent_for_common(&common_agent_address)
                .await? {
                    Some(agent) => agent,
                    None => {
                        info!("Level two agent not found for common agent: {}", common_agent_address);
                        String::new()
                    }
            };

            let level_one_agent = match self.agent_service
                .get_level_one_agent_for_level_two(&level_two_agent)
                .await? {
                    Some(agent) => agent,
                    None => {
                        info!("Level one agent not found for level two agent: {}", level_two_agent);
                        String::new()
                    }
            };

            (common_agent_address, level_two_agent, level_one_agent)
        } else {
            return Err(DbErr::Custom("Common agent must be specified".to_string()));
        };

        // 2. Calculate distribution amounts (in Wei)
        let platform_amount = amount * U256::from(40) / U256::from(100);    // 20% to platform
        // let owner_amount = amount * U256::from(10) / U256::from(100);      // 10% to owner
        let level_one_amount = amount * U256::from(10) / U256::from(100);   // 10% to level one agent
        let level_two_amount = amount * U256::from(10) / U256::from(100);   // 10% to level two agent
        let common_agent_amount = amount * U256::from(20) / U256::from(100); // 20% to common agent
        let pool_amount = amount - platform_amount - level_one_amount - level_two_amount - common_agent_amount; // Remainder to prize pool

        // Convert U256 amounts to Decimal (keeping them in Wei)
        let platform_decimal = Decimal::from_str(&platform_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert platform amount: {}", e)))?;
        // let owner_decimal = Decimal::from_str(&owner_amount.to_string())
        //     .map_err(|e| DbErr::Custom(format!("Failed to convert owner amount: {}", e)))?;
        let level_one_decimal = Decimal::from_str(&level_one_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert level one amount: {}", e)))?;
        let level_two_decimal = Decimal::from_str(&level_two_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert level two amount: {}", e)))?;
        let common_agent_decimal = Decimal::from_str(&common_agent_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert common agent amount: {}", e)))?;
        let pool_decimal = Decimal::from_str(&pool_amount.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to convert pool amount: {}", e)))?;

        // Process transfers
        let platform_tx = self.amount_service
            .process_regular_bet(&self.platform_account, platform_decimal)
            .await?;

        // let owner_tx = self.amount_service
        //     .process_regular_bet(&self.owner_account, owner_decimal)
        //     .await?;
        
        // 记录平台资金流
        self.amount_service.record_platform_funds_flow(
            self.platform_account.clone(),
            user_address.clone(),
            platform_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;

        // 记录平台奖金池
        self.amount_service.record_platform_prize_pool(
            user_address.clone(),
            pool_decimal / Decimal::from(10_u64.pow(18)),
            transaction_hash.clone(),
        ).await?;    

        let level_one_tx = if !level_one_agent.is_empty() {
            self.amount_service
                .process_regular_bet(&level_one_agent, level_one_decimal)
                .await?
        } else {
            String::new()
        };

        let level_two_tx = if !level_two_agent.is_empty() {
            self.amount_service
                .process_regular_bet(&level_two_agent, level_two_decimal)
                .await?
        } else {
            String::new()
        };

        let common_agent_tx = if !common_agent.is_empty() {
            self.amount_service
                .process_regular_bet(&common_agent, common_agent_decimal)
                .await?
        } else {
            String::new()
        };


        // 3. Record commission distribution
        if !common_agent.is_empty() {
            self.amount_service.record_commission(
                common_agent.clone(),
                user_address.clone(),
                common_agent_decimal / Decimal::from(10_u64.pow(18)),
                transaction_hash.clone(),
            ).await?;
        }

        if !level_two_agent.is_empty() {
            self.amount_service.record_commission(
                level_two_agent.clone(),
                user_address.clone(),
                level_two_decimal / Decimal::from(10_u64.pow(18)),
                transaction_hash.clone(),
            ).await?;
        }

        if !level_one_agent.is_empty() {    
            self.amount_service.record_commission(
                level_one_agent.clone(),
                user_address.clone(),
                level_one_decimal / Decimal::from(10_u64.pow(18)),
                transaction_hash.clone(),
            ).await?;
        }

        // 4. Get lucky numbers
        let numbers = self.buy_luck_number_service
            .get_all_luck_numbers(user_address, transaction_hash.clone(), amount)
            .await?;

        Ok(BetResult {
            success: true,
            message: "Bet processed successfully".to_string(),
            result: BuyLuckNumberResult {
                numbers,
                transaction_hash: Some(transaction_hash),
                total_amount: eth_amount,
            },
            additional_info: Some(json!({
                "platform_tx": platform_tx,
                "level_one_tx": level_one_tx,
                "level_two_tx": level_two_tx,
                "common_agent_tx": common_agent_tx,
                "owner_tx": platform_tx
            })),
        })
    }

    // 周日：中奖分配程序
    pub async fn sunday_lottery_distribution(&self) -> Result<Vec<(String, String, Decimal, String)>, DbErr> {
        // 获取当前时间
        // let now = self.get_current_time();
        // let sunday_start = self.get_sunday_start();

        // 检查是否在周日
        // if now < sunday_start {
        //     return Err(DbErr::Custom(
        //         "Lottery only available on Sunday".to_string(),
        //     ));
        // }

        // 获取总奖金池
        let total_prize_pool = self.calculate_total_prize_pool_from_platform_prize_pool().await?;
        info!("total_prize_pool: {} ETH", total_prize_pool);
        let config = &self.config.prize_distribution;
        
        let user_pool_percentage = Decimal::from_f64(config.user_pool_percentage)
            .unwrap_or(Decimal::new(0, 0));
        let first_prize_percentage = Decimal::from_f64(config.first_prize_percentage)
            .unwrap_or(Decimal::new(0, 0));
        let second_prize_percentage = Decimal::from_f64(config.second_prize_percentage)
            .unwrap_or(Decimal::new(0, 0));
        let third_prize_percentage = Decimal::from_f64(config.third_prize_percentage)
            .unwrap_or(Decimal::new(0, 0));

        // 使用配置的百分比计算奖金
        let user_prize_pool = total_prize_pool * user_pool_percentage;
        info!("user_pool_percentage: {}", user_pool_percentage);
        info!("user_prize_pool: {} ETH", user_prize_pool);
        let first_prize = user_prize_pool * first_prize_percentage;
        let second_prize = user_prize_pool * second_prize_percentage;
        let third_prize = user_prize_pool * third_prize_percentage;
        info!("first_prize: {} ETH, second_prize: {} ETH, third_prize: {} ETH", first_prize, second_prize, third_prize);

        
        // 获取非代理人的幸运号码
        let not_agent_numbers = self.buy_luck_number_service.get_not_agent_luck_numbers().await?;
        // 获取代理人的幸运号码
        let agent_numbers = self.buy_luck_number_service.get_agent_luck_numbers().await?;
        

        // 随机选择普通用户中奖者
        let first_winner = self.random_select_winner(not_agent_numbers.clone()).await?;
        let second_winner = self.random_select_winner(not_agent_numbers.clone()).await?;
        let third_winner = self.random_select_winner(not_agent_numbers.clone()).await?;

        let level_one_agent_percentage = Decimal::from_f64(config.level_one_agent_percentage)
            .unwrap_or(Decimal::new(0, 0));
        let level_two_agent_percentage = Decimal::from_f64(config.level_two_agent_percentage)
            .unwrap_or(Decimal::new(0, 0));

        // 计算代理人中奖金额
        let remaining_prize_pool = total_prize_pool - user_prize_pool; // 剩余奖金池
        let level_one_agent_prize = remaining_prize_pool * level_one_agent_percentage; // 一级代理 20%
        let level_two_agent_prize = remaining_prize_pool * level_two_agent_percentage; // 二级代理 10%
        let common_agent_prize =
            remaining_prize_pool - (level_one_agent_prize + level_two_agent_prize); // 剩余金额
        info!("level_one_agent_prize: {} ETH, level_two_agent_prize: {} ETH, common_agent_prize: {} ETH", level_one_agent_prize, level_two_agent_prize, common_agent_prize);
        // 随机选择代理人中奖者
        let level_one_winner = self.random_select_winner(agent_numbers.clone()).await?;
        let level_two_winner = self.random_select_winner(agent_numbers.clone()).await?;
        let common_agent_winner = self.random_select_winner(agent_numbers.clone()).await?;

        

        
        let winners = vec![
            (first_winner.0, first_winner.1, first_prize, "first_prize".to_string()),
            (second_winner.0, second_winner.1, second_prize, "second_prize".to_string()),
            (third_winner.0, third_winner.1, third_prize, "third_prize".to_string()),
            (level_one_winner.0, level_one_winner.1, level_one_agent_prize, "level_one_agent".to_string()),
            (level_two_winner.0, level_two_winner.1, level_two_agent_prize, "level_two_agent".to_string()),
            (common_agent_winner.0, common_agent_winner.1, common_agent_prize, "level_common_agent".to_string()),
        ];

        info!("winners: {:?}", winners);
        // 过滤掉中奖者为0x0000000000000000000000000000000000000000的记录   
        let filtered_winners: Vec<(String, String, Decimal, String)> = winners
            .into_iter()
            .filter(|(user_address, _, _, _)| user_address != "0x0000000000000000000000000000000000000000")
            .collect();
        // 分配奖金给中奖者
        self.amount_service.distribute_prize_to_winners(filtered_winners.clone()).await?;
        // 记录中奖者, 数据记录到数据库表lottery_distribution_detail
        self.amount_service.record_winners(filtered_winners.clone()).await?;

        Ok(filtered_winners.clone())
    }

    // 随机选择一个中奖者
    async fn random_select_winner(&self, numbers: Vec<LuckNumber>) -> Result<(String, String),DbErr> {
        match numbers.choose(&mut rand::thread_rng()) {
            Some(winner) => {
                info!("Selected winner: {} with lucky number {}", winner.user_address, winner.luck_number);
                Ok((winner.user_address.clone(), winner.luck_number.clone()))
            }
            None => {
                info!("No participants available for selection");
                Ok(("0x0000000000000000000000000000000000000000".to_string(), "0".to_string()))
            }
        }
    }

    // 查询总奖金池
    async fn calculate_total_prize_pool_from_prize_pool(&self) -> Result<f64, DbErr> {
        // 从奖金池地址查询余额
        let prize_pool_balance = self
            .amount_service
            .get_balance(&self.prize_pool_account)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get prize pool balance: {}", e)))?;

        // 将 Decimal 转换为 f64
        let balance_f64 = prize_pool_balance
            .to_string()
            .parse::<f64>()
            .map_err(|e| DbErr::Custom(format!("Failed to convert balance to f64: {}", e)))?;

        // 将 Wei 转换为 ETH (1 ETH = 10^18 Wei)
        let balance_eth = balance_f64 / 1_000_000_000_000_000_000.0;

        Ok(balance_eth)
    }

    // 查询总奖金池
    async fn calculate_total_prize_pool_from_platform_prize_pool(&self) -> Result<Decimal, DbErr> {
        // 1. 奖金池总额，从platform_prize_pool汇总计算获取，user_address是contract_address的值，汇总计算的是amount的值。
        let prize_pool_balance = self.amount_service.get_total_prize_pool_from_platform_prize_pool().await?;

    Ok(prize_pool_balance)
}
    
}


// 添加中奖信息的数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct WinnerInfo {
    user_address: String,
    prize_amount: f64,
    prize_level: String,
    winner_type: WinnerType,
}

impl WinnerInfo {
    pub fn new(
        user_address: String,
        prize_amount: f64,
        prize_level: String,
        winner_type: WinnerType,
    ) -> Self {
        Self {
            user_address,
            prize_amount,
            prize_level,
            winner_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WinnerType {
    NormalUser,
    LevelOneAgent,
    LevelTwoAgent,
    CommonAgent,
}

impl WinnerInfo {
    pub fn get_user_address(&self) -> &str {
        &self.user_address
    }

    pub fn get_prize_amount(&self) -> f64 {
        self.prize_amount
    }
}
