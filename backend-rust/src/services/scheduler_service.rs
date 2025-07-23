use std::ptr::null;
use crate::services::amount_service::AmountService;
use crate::services::buy_luck_number_service::BuyLuckNumberService;
use crate::services::week_action_service::{WeekActionConfig, WeekActionService};
use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    DatabaseConnection,
    DbErr,
    EntityTrait,
    QueryFilter,
    QuerySelect,
    Set,
    Value,
};
use std::sync::Arc;
// use futures_util::future::try_join_all;
use log::{info, error};
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::config::Config;
//use crate::entities::{prelude::*, *};
use futures::future::try_join_all as futures_join;
// use chrono_tz::Tz;

use crate::models::lottery_distribution_detail_model::{self, Entity as LotteryDistributionDetail};
use crate::entities::users::Entity as User;
use crate::entities::bet_records::Entity as BetRecords;
use crate::models::buy_luck_number_model::Entity as BuyLuckNumber;
use crate::models::agent_model::{self, Entity as Agent, Column as agentColumn};
use crate::models::commission_model::Entity as Commission;
use crate::models::platform_prize_pool_model::Entity as PlatformPrizePool;
use crate::models::platform_funds_flow_model::Entity as PlatformFundsFlow;
use crate::models::platform_transaction_gas_model::Entity as PlatformTransactionGas;

pub async fn start_scheduled_tasks(
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
) -> Result<(), DbErr> {
    let mut scheduler = JobScheduler::new();

    // 创建定时任务
    create_tuesday_task(&mut scheduler, Arc::clone(&db), Arc::clone(&buy_luck_number_service), Arc::clone(&amount_service)).await?;
    create_wednesday_task(&mut scheduler, Arc::clone(&db), Arc::clone(&buy_luck_number_service), Arc::clone(&amount_service)).await?;
    create_thursday_task(&mut scheduler, Arc::clone(&db), Arc::clone(&buy_luck_number_service), Arc::clone(&amount_service)).await?;
    create_sunday_task(&mut scheduler, Arc::clone(&db), Arc::clone(&buy_luck_number_service), Arc::clone(&amount_service)).await?;
    create_truncate_table_task(&mut scheduler, Arc::clone(&db)).await?;
    // 启动调度器
    if let Err(e) = scheduler.start().await {
        log::error!("Failed to start scheduler: {}", e);
        return Err(DbErr::Custom(format!("Scheduler error: {}", e)));
    }

    Ok(())
}

async fn create_tuesday_task(
    scheduler: &mut JobScheduler,
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
) -> Result<(), DbErr> {
    // Load configuration
    let config = Config::from_env().map_err(|e| DbErr::Custom(format!("Config error: {}", e)))?;
    
    info!("Current week_today: {}", config.week_today.today);
    info!("Confirm level one agents: {}", config.week_action.confirm_level_one_agents);
    
    if config.week_today.today == "tuesday" {
        info!("Today is Tuesday, checking agent confirmation settings");
        if config.week_action.confirm_level_one_agents {
            info!("Starting direct agent confirmation process");
            let service = WeekActionService::new(
                Arc::clone(&db),
                Arc::clone(&buy_luck_number_service),
                Arc::clone(&amount_service),
                WeekActionConfig::from_env("default").unwrap(),
            );
            info!("WeekActionService created, starting confirmation");
            service.confirm_level_one_agents().await?;
            info!("Agent confirmation completed");
        } else {
            info!("Not Tuesday, skipping agent confirmation");
        }
    } else {
        info!("Scheduling agent confirmation task for Tuesday");
        // Schedule task when confirm_level_one_agents is false
        let job = Job::new_async("0 0 0 * * TUE", move |_uuid, _l| {
            let db = Arc::clone(&db);
            let buy_luck_number_service = Arc::clone(&buy_luck_number_service);
            let amount_service = Arc::clone(&amount_service);
            
            Box::pin(async move {
                tokio::spawn(async move {
                    let service = WeekActionService::new(
                        db,
                        buy_luck_number_service,
                        amount_service,
                        WeekActionConfig::from_env("default").unwrap(),
                    );
                    if let Err(e) = service.confirm_level_one_agents().await {
                        eprintln!("Error confirming level one agents: {}", e);
                    }
                });
            })
        }).map_err(|e| DbErr::Custom(format!("Job creation error: {:?}", e)))?;

        scheduler.add(job).map_err(|e| DbErr::Custom(format!("Job scheduling error: {:?}", e)))?;
    }
    
    Ok(())
}

async fn create_wednesday_task(
    scheduler: &mut JobScheduler,
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
) -> Result<(), DbErr> {
    // Load configuration
    let config = Config::from_env().map_err(|e| DbErr::Custom(format!("Config error: {}", e)))?;
    
    info!("Current week_today: {}", config.week_today.today);
    info!("Confirm level two agents: {}", config.week_action.confirm_level_two_agents);
    
    if config.week_today.today == "wednesday" {
        info!("Today is Wednesday, checking agent confirmation settings");
        if config.week_action.confirm_level_two_agents {
            info!("Starting direct agent confirmation process");
            let service = WeekActionService::new(
                Arc::clone(&db),
                Arc::clone(&buy_luck_number_service),
                Arc::clone(&amount_service),
                WeekActionConfig::from_env("default").unwrap(),
            );
            info!("WeekActionService created, starting confirmation");
            service.confirm_level_two_agents().await?;
            info!("Agent confirmation completed");
        } else {
            info!("Not Wednesday, skipping agent confirmation");
        }
    } else {
        info!("Scheduling agent confirmation task for Wednesday");
        let job = Job::new_async("0 0 0 * * WED", move |_uuid, _l| {
            let db = Arc::clone(&db);
            let buy_luck_number_service = Arc::clone(&buy_luck_number_service);
            let amount_service = Arc::clone(&amount_service);
            
            Box::pin(async move {
                tokio::spawn(async move {
                    let service = WeekActionService::new(
                        db,
                        buy_luck_number_service,
                        amount_service,
                        WeekActionConfig::from_env("default").unwrap(),
                    );
                    if let Err(e) = service.confirm_level_two_agents().await {
                        eprintln!("Error confirming level two agents: {}", e);
                    }
                });
            })
        }).map_err(|e| DbErr::Custom(format!("Job creation error: {:?}", e)))?;

        scheduler.add(job).map_err(|e| DbErr::Custom(format!("Job scheduling error: {:?}", e)))?;
    }
    
    Ok(())
}

async fn create_thursday_task(
    scheduler: &mut JobScheduler,
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
) -> Result<(), DbErr> {
    // Load configuration
    let config = Config::from_env().map_err(|e| DbErr::Custom(format!("Config error: {}", e)))?;
    
    info!("Current week_today: {}", config.week_today.today);
    info!("Confirm common agents: {}", config.week_action.confirm_level_common_agents);
    
    if config.week_today.today == "thursday" {
        info!("Today is Thursday, checking agent confirmation settings");
        if config.week_action.confirm_level_common_agents {
            info!("Starting direct agent confirmation process");
            let service = WeekActionService::new(
                Arc::clone(&db),
                Arc::clone(&buy_luck_number_service),
                Arc::clone(&amount_service),
                WeekActionConfig::from_env("default").unwrap(),
            );
            info!("WeekActionService created, starting confirmation");
            service.confirm_level_common_agents().await?;
            info!("Agent confirmation completed");
        } else {
            info!("Not Thursday, skipping agent confirmation");
        }
    } else {
        info!("Scheduling agent confirmation task for Thursday");
            let job = Job::new_async("0 0 0 * * THU", move |_uuid, _l| {
                let db = Arc::clone(&db);
                let buy_luck_number_service = Arc::clone(&buy_luck_number_service);
                let amount_service = Arc::clone(&amount_service);
                
                Box::pin(async move {
                    tokio::spawn(async move {
                        let service = WeekActionService::new(
                            db,
                            buy_luck_number_service,
                            amount_service,
                            WeekActionConfig::from_env("default").unwrap(),
                        );
                        if let Err(e) = service.confirm_level_common_agents().await {
                            eprintln!("Error confirming common agents: {}", e);
                        }
                    });
                })
            }).map_err(|e| DbErr::Custom(format!("Job creation error: {:?}", e)))?;

            scheduler.add(job).map_err(|e| DbErr::Custom(format!("Job scheduling error: {:?}", e)))?;
    }
    
    Ok(())
}

async fn create_sunday_task(
    scheduler: &mut JobScheduler,
    db: Arc<DatabaseConnection>,
    buy_luck_number_service: Arc<BuyLuckNumberService>,
    amount_service: Arc<AmountService>,
) -> Result<(), DbErr> {
    // Load configuration
    let config = Config::from_env().map_err(|e| DbErr::Custom(format!("Config error: {}", e)))?;
    
    info!("Current week_today: {}", config.week_today.today);
    info!("Sunday lottery distribution: {}", config.week_action.sunday_lottery_distribution);
    
    if config.week_today.today == "sunday" {
        info!("Today is Sunday, checking lottery distribution settings");
        if config.week_action.sunday_lottery_distribution {
            info!("Starting direct lottery distribution process");
            let service = WeekActionService::new(
                Arc::clone(&db),
                Arc::clone(&buy_luck_number_service),
                Arc::clone(&amount_service),
                WeekActionConfig::from_env("default").unwrap(),
            );
            info!("WeekActionService created, starting lottery distribution");
            service.sunday_lottery_distribution().await?;
            info!("Lottery distribution completed");
        } else {
            info!("Not Sunday, skipping lottery distribution");
        }
    } else {
        info!("Scheduling lottery distribution task for Sunday");
        let job = Job::new_async("0 0 0 * * SUN", move |_uuid, _l| {
            let db = Arc::clone(&db);
            let buy_luck_number_service = Arc::clone(&buy_luck_number_service);
            let amount_service = Arc::clone(&amount_service);
            
            Box::pin(async move {
                tokio::spawn(async move {
                    let service = WeekActionService::new(
                        db,
                        buy_luck_number_service,
                        amount_service,
                        WeekActionConfig::from_env("default").unwrap(),
                    );
                    if let Err(e) = service.sunday_lottery_distribution().await {
                        eprintln!("Error in Sunday lottery distribution: {}", e);
                    }
                });
            })
        }).map_err(|e| DbErr::Custom(format!("Job creation error: {:?}", e)))?;

        scheduler.add(job).map_err(|e| DbErr::Custom(format!("Job scheduling error: {:?}", e)))?;
    }
    
    Ok(())
}

async fn create_truncate_table_task(
    scheduler: &mut JobScheduler,
    db: Arc<DatabaseConnection>,
) -> Result<(), DbErr> {
    let job = Job::new_async("0 55 23 * * SUN", move |_uuid, _l| {
        let db = Arc::clone(&db);
        
        Box::pin(async move {
            tokio::spawn(async move {
                info!("Starting weekly data cleanup task");
                
                // Handle each deletion separately
                let cleanup_results = vec![
                    LotteryDistributionDetail::delete_many().exec(&*db).await,
                    PlatformTransactionGas::delete_many().exec(&*db).await,
                    PlatformPrizePool::delete_many().exec(&*db).await,
                    PlatformFundsFlow::delete_many().exec(&*db).await,
                    Commission::delete_many().exec(&*db).await,
                    BuyLuckNumber::delete_many().exec(&*db).await,
                    BetRecords::delete_many().exec(&*db).await,
                    //Agent::delete_many().exec(&*db).await,
                    //User::delete_many().exec(&*db).await,
                ];

                // Update Agents table without using ? operator
                match Agent::update_many()
                    .col_expr(agentColumn::LevelAgent, "not_agent".into())
                    .col_expr(agentColumn::SuperiorAddress, Option::<String>::None.into())
                    .exec(&*db)
                    .await
                {
                    Ok(_) => info!("Successfully updated agent levels to not_agent"),
                    Err(e) => error!("Error updating agent levels: {}", e),
                }

                // Log results for each operation
                for (idx, result) in cleanup_results.into_iter().enumerate() {
                    match result {
                        Ok(_) => info!("Cleanup successful for table {}", idx),
                        Err(e) => error!("Error cleaning up table {}: {}", idx, e),
                    }
                }
            });
        })
    }).map_err(|e| DbErr::Custom(format!("Job creation error: {:?}", e)))?;

    scheduler.add(job).map_err(|e| DbErr::Custom(format!("Job scheduling error: {:?}", e)))?;
    
    Ok(())
}