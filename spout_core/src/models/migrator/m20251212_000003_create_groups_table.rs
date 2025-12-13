use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000003_create_groups_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .col(ColumnDef::new(Group::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Group::ProfileId).string().not_null())
                    .to_owned(),
            )
            .await?;

        // Create index on profile_id
        manager
            .create_index(
                Index::create()
                    .name("idx_groups_profile_id")
                    .table(Group::Table)
                    .col(Group::ProfileId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Group {
    Table,
    Id,
    ProfileId,
}
