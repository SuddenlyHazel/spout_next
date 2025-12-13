use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000005_create_group_banned_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupBanned::Table)
                    .col(ColumnDef::new(GroupBanned::GroupId).string().not_null())
                    .col(ColumnDef::new(GroupBanned::IdentityId).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(GroupBanned::GroupId)
                            .col(GroupBanned::IdentityId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_banned_group_id")
                            .from(GroupBanned::Table, GroupBanned::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on identity_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_banned_identity_id")
                    .table(GroupBanned::Table)
                    .col(GroupBanned::IdentityId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupBanned::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum GroupBanned {
    Table,
    GroupId,
    IdentityId,
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
}
