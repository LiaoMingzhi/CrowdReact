use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use sea_orm::prelude::*;
use std::sync::Arc;
use sea_orm::TransactionTrait;
use crate::models::buy_luck_number_model;
use crate::services::buy_luck_number_service::BuyLuckNumberService;
use crate::models::buy_luck_number_model::Model as LuckNumber;
use crate::services::week_action_service::WinnerInfo;

pub struct LotteryService {
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
}

impl LotteryService {
    pub fn new(db: Arc<DatabaseConnection>, buy_luck_number_service: Arc<BuyLuckNumberService>) -> Self {
        Self {
            db,
            buy_luck_number_service,
        }
    }

    // pub async fn process_weekend_lottery(&self) -> Result<Vec<LuckNumber>, DbErr> {
    //     let now = Utc::now();
    //     let sunday_start = self.get_sunday_start(now);
    // 
    //     if now < sunday_start {
    //         return Err(DbErr::Custom("Lottery only available on Sunday".to_string()));
    //     }
    // 
    //     // Get all participating numbers
    //     let all_numbers = self.buy_luck_number_service
    //         .get_all_participating_numbers()
    //         .await?;
    // 
    //     // Process winners
    //     self.process_lottery_winners(all_numbers).await
    // }

    fn get_sunday_start(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        // Implementation for getting Sunday start time
        let mut sunday = now;
        while sunday.weekday().number_from_monday() != 7 {
            sunday = sunday - chrono::Duration::days(1);
        }
        sunday.date().and_hms_opt(0, 0, 0).unwrap()
    }

    async fn process_lottery_winners(&self, numbers: Vec<LuckNumber>) -> Result<Vec<LuckNumber>, DbErr> {
        // Implementation for processing lottery winners
        todo!("Implement lottery winner selection logic")
    }

    // async fn calculate_winners(&self) -> Result<Vec<WinnerInfo>, DbErr> {
    //     let mut transaction = self.db.begin().await?;
    // 
    //     // Get all luck numbers for the current week
    //     let luck_numbers = crate::models::buy_luck_number_model::Entity::find()
    //         .filter(
    //             buy_luck_number_model::Column::CreatedAt.gte(self.get_week_start())
    //                 .and(buy_luck_number_model::Column::CreatedAt.lt(self.get_week_end()))
    //         )
    //         .order_by_desc(buy_luck_number_model::Column::Amount)
    //         .all(&transaction)
    //         .await?;
    // 
    //     // Select winners based on amount and timestamp
    //     let mut winners = Vec::new();
    // 
    //     // Process winners and update their status
    //     for (index, number) in luck_numbers.iter().enumerate().take(3) {
    //         let prize_level = match index {
    //             0 => PRIZE_LEVEL_FIRST,
    //             1 => PRIZE_LEVEL_SECOND,
    //             2 => PRIZE_LEVEL_THIRD,
    //             _ => continue,
    //         };
    // 
    //         let mut winner_model: buy_luck_number_model::ActiveModel = number.clone().into();
    //         winner_model.is_winner = Set(true);
    //         winner_model.prize_level = Set(Some(prize_level.to_string()));
    //         winner_model.update(&transaction).await?;
    // 
    //         winners.push(WinnerInfo::new(
    //             number.user_address.clone(),
    //             number.amount.to_f64().unwrap(),
    //             prize_level.to_string(),
    //             WinnerType::NormalUser,
    //         ));
    //     }
    // 
    //     transaction.commit().await?;
    //     Ok(winners)
    // }
} 