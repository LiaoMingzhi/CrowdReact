use std::str::FromStr;
// src/models/buy_luck_number_model.rs
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "buy_luck_number")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing_if = "is_zero")]
    pub id: i32,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub user_address: String,
    #[sea_orm(column_type = "String(StringLen::N(36))")]
    pub luck_number: String,
    #[sea_orm(column_type = "String(StringLen::N(66))", nullable)]
    pub transaction_hash: Option<String>,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub is_winner: bool,
    #[sea_orm(column_type = "String(StringLen::N(20))", nullable)]
    pub prize_level: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(
        user_address: String,
    ) -> Self {
        Self {
            id: 0, // 数据库自动生成
            luck_number: Uuid::new_v4().to_string(),
            user_address,
            transaction_hash: None,
            is_winner: false,
            prize_level: None,
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            
        }
    }

    pub fn set_winner(&mut self, prize_level: String) {
        self.is_winner = true;
        self.prize_level = Some(prize_level);
        self.updated_at = chrono::Utc::now().into();
    }
}

// 查询辅助结构体
#[derive(Debug)]
pub struct LuckNumberQuery {
    pub user_address: Option<String>,
    pub level_one_agent: Option<String>,
    pub level_two_agent: Option<String>,
    pub common_agent: Option<String>,
    pub is_winner: Option<bool>,
    pub prize_level: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// Add helper function for serialization
fn is_zero(id: &i32) -> bool {
    *id == 0
}

// Add custom serialization functions for Decimal
fn serialize_decimal<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&decimal.to_string())
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Decimal::from_str(&s).map_err(serde::de::Error::custom)
}

