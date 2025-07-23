use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(BetRecords::Table)
                    .add_column(
                        ColumnDef::new(BetRecords::Status)
                            .string()
                            .not_null()
                            .default("Pending"),
                    )
                    .add_column(
                        ColumnDef::new(BetRecords::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(BetRecords::Table)
                    .drop_column(BetRecords::Status)
                    .drop_column(BetRecords::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
} 