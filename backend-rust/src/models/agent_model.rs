// src/models/agent_model.rs
use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use sea_orm::{NotSet, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(column_type = "String(StringLen::N(255))", unique)]
    pub user_address: String,
    #[sea_orm(column_type = "String(StringLen::N(20))", default_value = "not_agent")]
    #[sea_orm(check_constraint = r#"level_agent IN ('one', 'two', 'common', 'not_agent')"#)]
    pub level_agent: String,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    #[sea_orm(foreign_key = "agents(user_address)", on_delete = "SetNull")]
    pub superior_address: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone", default_value = "CURRENT_TIMESTAMP")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone", default_value = "CURRENT_TIMESTAMP")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub is_frozen: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "Entity")]
    SubAgents,
    #[sea_orm(belongs_to = "Entity", from = "Column::SuperiorAddress", to = "Column::UserAddress")]
    Superior,
}


impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            id: NotSet,
            user_address: NotSet,
            level_agent: Set("not_agent".to_string()),
            superior_address: NotSet,
            created_at: Set(now),
            updated_at: Set(now),
            is_frozen: Set(false),
        }
    }

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
            if !insert {
                self.updated_at = Set(Utc::now());
            }
            Ok(self)
        })
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubAgents.def()
    }

    fn via() -> Option<RelationDef> {
        Some(Relation::Superior.def())
    }
} 