use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000001_create_identity_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Identity table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Identity::Table)
                    .col(ColumnDef::new(Identity::NodeId).binary().not_null())
                    .col(
                        ColumnDef::new(Identity::ProfileId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
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

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Identity::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Identity {
    Table,
    NodeId,
    ProfileId,
}
