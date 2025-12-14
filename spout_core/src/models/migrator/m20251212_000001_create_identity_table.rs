use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Identity table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Identity::Table)
                    .col(binary(Identity::NodeId))
                    .col(uuid_uniq(Identity::ProfileId))
                    .index(
                        Index::create()
                            .primary()
                            .col(Identity::NodeId)
                            .col(Identity::ProfileId),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Identity table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Identity::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Identity {
    Table,
    NodeId,
    ProfileId,
}
