use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use chrono::format::Numeric;
use ethers::core::k256::sha2::digest::typenum::Integer;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_transaction_gas")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_address: String,
    pub from_address: String,
    pub amount_wei: Decimal,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}