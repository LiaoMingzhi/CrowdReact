use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bet_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_address: String,
    pub amount: String,
    pub transaction_hash: String,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::models::agent_model::Entity",
        from = "Column::AccountAddress",
        to = "crate::models::agent_model::Column::UserAddress"
    )]
    Agents,
}

impl ActiveModelBehavior for ActiveModel {}