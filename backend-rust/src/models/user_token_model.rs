use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    #[sea_orm(column_type = "Boolean", default_value = "true")]
    pub is_valid: bool,
    #[sea_orm(column_type = "DateTime")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "DateTime")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "DateTime")]
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
