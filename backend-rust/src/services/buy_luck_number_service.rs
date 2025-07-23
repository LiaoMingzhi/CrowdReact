use std::str::FromStr;
use chrono::{DateTime, Datelike, Utc};
use sea_orm::prelude::Decimal;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, EntityTrait, JsonValue, QueryOrder, Set, Statement, TransactionTrait};
use serde::Serialize;
use std::sync::Arc;
use crate::models::buy_luck_number_model::{self, ActiveModel, Entity as BuyLuckNumber, Model as LuckNumber};
use crate::services::amount_service::AmountService;
use rust_decimal::prelude::ToPrimitive;
use time::OffsetDateTime;
use uuid::Uuid;
use web3::types::U256;
use crate::services::week_action_service::{WinnerInfo, WinnerType};
use rand::Rng;
use std::collections::HashMap;
use crate::entities::bet_records::Entity;
use crate::models::agent_model::{self, Entity as Agent, Model as AgentModel};

#[derive(Debug, Serialize)]
pub struct BuyLuckNumberResult {
    pub numbers: Vec<LuckNumber>,
    pub transaction_hash: Option<String>,
    pub total_amount: Decimal,
}

#[cfg_attr(test, cfg(test))]
pub trait BuyLuckNumberServiceTrait {
    async fn buy_luck_number(
        &self,
        user_address: String,
        amount: Decimal,
        level_one_agent: Option<String>,
        level_two_agent: Option<String>,
        common_agent: Option<String>,
    ) -> Result<String, DbErr>;

    async fn get_user_luck_numbers(&self, user_address: &str) -> Result<Vec<LuckNumber>, DbErr>;
    async fn get_agent_luck_numbers(&self, agent_address: &str) -> Result<Vec<LuckNumber>, DbErr>;
    async fn get_all_luck_numbers(&self, amount: U256) -> Result<Vec<LuckNumber>, DbErr>;
    async fn get_top_100_users(&self) -> Result<Vec<String>, DbErr>;
    async fn get_top_1000_users(&self) -> Result<Vec<String>, DbErr>;
}

#[derive(Clone)]
pub struct BuyLuckNumberService {
    db: Arc<DatabaseConnection>,
    amount_service: Arc<AmountService>,
}

impl BuyLuckNumberService {
    pub fn new(db: Arc<DatabaseConnection>, amount_service: Arc<AmountService>) -> Self {
        Self { db, amount_service }
    }
    
    
    pub async fn get_user_luck_numbers(&self) -> Result<Vec<LuckNumber>, DbErr> {
        // Get all luck numbers that haven't won yet
        let all_numbers = BuyLuckNumber::find()
            .filter(
                Condition::all()
                    .add(buy_luck_number_model::Column::IsWinner.eq(false))
            )
            .order_by_asc(buy_luck_number_model::Column::CreatedAt)
            .all(&*self.db)
            .await?;

        // Get all agents (one, two, common)
        let agents = Agent::find()
            .filter(
                Condition::any()
                    .add(agent_model::Column::LevelAgent.eq("one"))
                    .add(agent_model::Column::LevelAgent.eq("two"))
                    .add(agent_model::Column::LevelAgent.eq("common"))
            )
            .all(&*self.db)
            .await?;

        let agent_addresses: Vec<String> = agents
            .into_iter()
            .map(|agent| agent.user_address)
            .collect();

        // Partition numbers into non-agent and agent numbers
        let (not_agent_numbers, agent_numbers): (Vec<_>, Vec<_>) = all_numbers
            .into_iter()
            .partition(|number| !agent_addresses.contains(&number.user_address));

        // Return all numbers in the order: not-agent numbers first, then agent numbers
        Ok([not_agent_numbers, agent_numbers].concat())
    }

    pub async fn get_all_luck_numbers(
        &self,
        user_address: String,
        transaction_hash: String,
        amount: U256,
    ) -> Result<Vec<LuckNumber>, DbErr> {
        // Generate luck numbers
        let numbers = self.generate_luck_numbers(amount);
        
        // Insert each luck number and collect the results
        let mut luck_numbers = Vec::with_capacity(numbers.len());
        
        for number in numbers {
            let buy_luck_number = buy_luck_number_model::ActiveModel {
                user_address: Set(user_address.clone()),
                luck_number: Set(number),
                transaction_hash: Set(Some(transaction_hash.clone())),
                is_winner: Set(false),
                prize_level: Set(None),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
                ..Default::default()
            };
            
            // Insert and get the model back
            let model = buy_luck_number.insert(&*self.db).await?;
            luck_numbers.push(model);
        }
        
        Ok(luck_numbers)
    }
    

    pub async fn get_top_100_users(&self) -> Result<Vec<String>, DbErr> {
        // Get all bet records
        let bet_records = Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT 
                    account_address,
                    SUM(CAST(amount AS DECIMAL)) as total_amount,
                    MIN(created_at) as earliest_bet
                FROM bet_records
                WHERE status = 'confirmed'
                GROUP BY account_address
                ORDER BY total_amount DESC, earliest_bet ASC
                LIMIT 100
                "#,
                vec![],
            ))
            .into_model::<JsonValue>()
            .all(&*self.db)
            .await?;

        // Extract addresses from results
        Ok(bet_records
            .into_iter()
            .map(|record| record["account_address"].as_str().unwrap().to_string())
            .collect())
    }

    pub async fn get_top_1000_users(&self) -> Result<Vec<String>, DbErr> {
        // Get all bet records
        let bet_records = Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT 
                    account_address,
                    SUM(CAST(amount AS DECIMAL)) as total_amount,
                    MIN(created_at) as earliest_bet
                FROM bet_records
                WHERE status = 'confirmed'
                AND account_address NOT IN (
                    SELECT user_address 
                    FROM agents 
                    WHERE level_agent = 'one'
                )
                GROUP BY account_address
                ORDER BY total_amount DESC, earliest_bet ASC
                LIMIT 1000
                "#,
                vec![],
            ))
            .into_model::<JsonValue>()
            .all(&*self.db)
            .await?;

        // Extract addresses from results
        Ok(bet_records
            .into_iter()
            .map(|record| record["account_address"].as_str().unwrap().to_string())
            .collect())
    }

    pub async fn get_common_agent_users(&self) -> Result<Vec<String>, DbErr> {
        // Get all bet records with total amount > 0.1 ETH (0.1 * 10^18 Wei)
        let eth_amount = 0.1; // 0.1 ETH  f64

        let bet_records = Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                SELECT 
                    account_address,
                    SUM(CAST(amount AS DECIMAL)) as total_amount
                FROM bet_records
                WHERE status = 'confirmed'
                AND account_address NOT IN (
                    SELECT user_address 
                    FROM agents 
                    WHERE level_agent IN ('one', 'two')
                )
                GROUP BY account_address
                HAVING SUM(CAST(amount AS DECIMAL)) >= $1
                ORDER BY total_amount DESC
                "#,
                vec![eth_amount.into()], // Convert to sea-orm Value
            ))
            .into_model::<JsonValue>()
            .all(&*self.db)
            .await?;

        // Extract addresses from results
        Ok(bet_records
            .into_iter()
            .map(|record| record["account_address"].as_str().unwrap().to_string())
            .collect())
    }

    pub async fn get_not_agent_luck_numbers(&self) -> Result<Vec<LuckNumber>, DbErr> {
        // Get all agents (not_agent)
        let agents = Agent::find()
            .filter(
                Condition::any()
                    .add(agent_model::Column::LevelAgent.eq("not_agent"))
            )
            .all(&*self.db)
            .await?;

        let not_agent_addresses: Vec<String> = agents
            .into_iter()
            .map(|agent| agent.user_address)
            .collect();

        // Get all non-agent luck numbers that haven't won yet
        let not_agent_numbers = BuyLuckNumber::find()
            .filter(
                Condition::all()
                    .add(buy_luck_number_model::Column::IsWinner.eq(false))
                    .add(buy_luck_number_model::Column::UserAddress.is_in(not_agent_addresses))
            )
            .order_by_asc(buy_luck_number_model::Column::CreatedAt)
            .all(&*self.db)
            .await?;

        Ok(not_agent_numbers)
    }

    pub async fn get_agent_luck_numbers(&self) -> Result<Vec<LuckNumber>, DbErr> {
        // Get all agents (one, two, common)
        let agents = Agent::find()
            .filter(
                Condition::any()
                    .add(agent_model::Column::LevelAgent.eq("one"))
                    .add(agent_model::Column::LevelAgent.eq("two"))
                    .add(agent_model::Column::LevelAgent.eq("common"))
            )
            .all(&*self.db)
            .await?;

        let agent_addresses: Vec<String> = agents
            .into_iter()
            .map(|agent| agent.user_address)
            .collect();

        // Get all agent luck numbers that haven't won yet
        let agent_numbers = BuyLuckNumber::find()
            .filter(
                Condition::all()
                    .add(buy_luck_number_model::Column::IsWinner.eq(false))
                    .add(buy_luck_number_model::Column::UserAddress.is_in(agent_addresses))
            )
            .order_by_asc(buy_luck_number_model::Column::CreatedAt)
            .all(&*self.db)
            .await?;

        Ok(agent_numbers)
    }

    fn generate_luck_numbers(&self, amount: U256) -> Vec<String> {
        // Convert amount from Wei to ETH (1 ETH = 10^18 Wei)
        let eth_amount = amount.as_u128() as f64 / 1_000_000_000_000_000_000.0;
        
        // Calculate number of luck numbers (1 number per 0.001 ETH)
        let num_numbers = (eth_amount / 0.001).round() as usize;
        
        // Generate unique luck numbers
        let mut numbers = Vec::with_capacity(num_numbers);
        for _ in 0..num_numbers {
            numbers.push(Uuid::new_v4().to_string());
        }
        
        numbers
    }
}



pub const STATUS_INITIAL: i32 = 0;
pub const STATUS_NORMAL_BET: i32 = 1;
pub const STATUS_LEVEL_TWO_COMPETITION: i32 = 2;
pub const STATUS_NORMAL_COMPETITION: i32 = 3;
pub const STATUS_REGULAR_BET: i32 = 4;

pub const PRIZE_LEVEL_FIRST: &str = "first";
pub const PRIZE_LEVEL_SECOND: &str = "second";
pub const PRIZE_LEVEL_THIRD: &str = "third";
