// src/models/week_action_model.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::prelude::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "week_actions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub action_type: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub user_address: String,
    #[sea_orm(column_type = "Decimal(Some((20, 8)))")]
    pub amount: Decimal,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub level_one_agent: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub level_two_agent: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub common_agent: Option<String>,
    #[sea_orm(column_type = "Boolean", default_value = "false")]
    pub is_processed: bool,
    #[sea_orm(column_type = "DateTime")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "DateTime", nullable)]
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 活动类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    MondayCompetition,
    TuesdayCompetition,
    WednesdayRegistration,
    WeekdayBetting,
    SundayLottery,
}

impl ToString for ActionType {
    fn to_string(&self) -> String {
        match self {
            ActionType::MondayCompetition => "monday_competition",
            ActionType::TuesdayCompetition => "tuesday_competition",
            ActionType::WednesdayRegistration => "wednesday_registration",
            ActionType::WeekdayBetting => "weekday_betting",
            ActionType::SundayLottery => "sunday_lottery",
        }
        .to_string()
    }
}

impl Model {
    pub fn new(
        action_type: ActionType,
        user_address: String,
        amount: Decimal,
        level_one_agent: Option<String>,
        level_two_agent: Option<String>,
        common_agent: Option<String>,
    ) -> Self {
        Self {
            id: 0, // 数据库自动生成
            action_type: action_type.to_string(),
            user_address,
            amount,
            level_one_agent,
            level_two_agent,
            common_agent,
            is_processed: false,
            created_at: Utc::now(),
            processed_at: None,
        }
    }

    pub fn mark_as_processed(&mut self) {
        self.is_processed = true;
        self.processed_at = Some(Utc::now());
    }
}

// 查询辅助结构体
#[derive(Debug)]
pub struct WeekActionQuery {
    pub action_type: Option<ActionType>,
    pub user_address: Option<String>,
    pub is_processed: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_week_action() {
        let action = Model::new(
            ActionType::MondayCompetition,
            "0x123...".to_string(),
            Decimal::new(1, 0),
            None,
            None,
            None,
        );

        assert_eq!(action.action_type, "monday_competition");
        assert!(!action.is_processed);
        assert!(action.processed_at.is_none());
    }

    #[test]
    fn test_mark_as_processed() {
        let mut action = Model::new(
            ActionType::MondayCompetition,
            "0x123...".to_string(),
            Decimal::new(1, 0),
            None,
            None,
            None,
        );

        action.mark_as_processed();
        assert!(action.is_processed);
        assert!(action.processed_at.is_some());
    }
}
