use sea_orm_migration::prelude::*;

mod create_bet_records;
mod add_status_and_updated_at;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_bet_records::Migration),
            Box::new(add_status_and_updated_at::Migration),
        ]
    }
}