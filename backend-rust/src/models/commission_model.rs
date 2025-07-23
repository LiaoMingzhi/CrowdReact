use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "commissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_address: String,
    pub from_address: String,
    pub commission: Decimal,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {} 