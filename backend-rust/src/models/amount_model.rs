use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use std::str::FromStr;
use std::sync::Arc;
use web3::types::U256;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "amounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub transaction_hash: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub from_address: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub to_address: String,
    #[sea_orm(column_type = "Decimal(Some((20, 18)))")]
    pub amount: Decimal,
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub transaction_type: String, // "bet", "prize", "commission"
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub status: String, // "pending", "completed", "failed"
    #[sea_orm(column_type = "DateTime")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "DateTime", nullable)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn before_save<'life0, 'async_trait, C>(
        mut self,
        _db: &'life0 C,
        insert: bool,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Self, DbErr>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        C: ConnectionTrait,
    {
        Box::pin(async move {
            if insert {
                self.created_at = Set(Utc::now());
            }
            self.updated_at = Set(Some(Utc::now()));
            Ok(self)
        })
    }
}

impl Model {
    pub fn new(
        transaction_hash: String,
        from_address: String,
        to_address: String,
        amount: U256,
        transaction_type: String,
    ) -> Self {
        Self {
            id: 0, // 数据库自动生成
            transaction_hash,
            from_address,
            to_address,
            amount: U256_to_decimal(amount),
            transaction_type,
            status: "pending".to_string(),
            created_at: Utc::now(),
            updated_at: Option::from(Utc::now()),
        }
    }

    pub fn amount_as_u256(&self) -> U256 {
        decimal_to_U256(self.amount)
    }

    pub fn update_status(&mut self, status: String) {
        self.status = status;
        self.updated_at = Some(Utc::now());
    }
}

// 辅助函数：将 U256 转换为 Decimal
fn U256_to_decimal(value: U256) -> Decimal {
    // 将 U256 转换为字符串，然后解析为 Decimal
    let value_string = value.to_string();
    Decimal::from_str(&value_string).unwrap_or_default()
}

// 辅助函数：将 Decimal 转换为 U256
fn decimal_to_U256(value: Decimal) -> U256 {
    // 将 Decimal 转换为字符串，然后解析为 U256
    let value_string = value.to_string();
    U256::from_dec_str(&value_string).unwrap_or_default()
}

// 交易类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Bet,
    Prize,
    Commission,
}

impl ToString for TransactionType {
    fn to_string(&self) -> String {
        match self {
            TransactionType::Bet => "bet",
            TransactionType::Prize => "prize",
            TransactionType::Commission => "commission",
        }
        .to_string()
    }
}

// 交易状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
}

impl ToString for TransactionStatus {
    fn to_string(&self) -> String {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Completed => "completed",
            TransactionStatus::Failed => "failed",
        }
        .to_string()
    }
}

// 查询辅助结构体
#[derive(Debug)]
pub struct AmountQuery {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_decimal_conversion() {
        let original = U256::from(1000000000000000000u64); // 1 ETH
        let decimal = U256_to_decimal(original);
        let back_to_u256 = decimal_to_U256(decimal);
        assert_eq!(original, back_to_u256);
    }

    #[test]
    fn test_transaction_type_to_string() {
        assert_eq!(TransactionType::Bet.to_string(), "bet");
        assert_eq!(TransactionType::Prize.to_string(), "prize");
        assert_eq!(TransactionType::Commission.to_string(), "commission");
    }

    #[test]
    fn test_transaction_status_to_string() {
        assert_eq!(TransactionStatus::Pending.to_string(), "pending");
        assert_eq!(TransactionStatus::Completed.to_string(), "completed");
        assert_eq!(TransactionStatus::Failed.to_string(), "failed");
    }
}
