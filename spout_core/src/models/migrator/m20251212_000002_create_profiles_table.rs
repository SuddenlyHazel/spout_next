use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000002_create_profiles_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Profile::Table)
                    .col(
                        ColumnDef::new(Profile::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Profile::Name).string().not_null())
                    .col(ColumnDef::new(Profile::Desc).string().not_null())
                    .col(ColumnDef::new(Profile::Picture).binary().null())
                    .to_owned(),
            )
            .await?;

        // Create unique index on id
        manager
            .create_index(
                Index::create()
                    .name("idx_profiles_id")
                    .table(Profile::Table)
                    .col(Profile::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create unique index on name
        manager
            .create_index(
                Index::create()
                    .name("idx_profiles_name")
                    .table(Profile::Table)
                    .col(Profile::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Profile::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Profile {
    Table,
    Id,
    Name,
    Desc,
    Picture,
}
