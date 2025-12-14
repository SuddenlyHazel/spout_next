use sea_orm_migration::{prelude::*, schema::*};

use super::m20251212_000003_create_groups_table::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupBanned::Table)
                    .col(uuid(GroupBanned::GroupId))
                    .col(uuid(GroupBanned::IdentityId))
                    .primary_key(
                        Index::create()
                            .col(GroupBanned::GroupId)
                            .col(GroupBanned::IdentityId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-banned-group_id")
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

#[derive(DeriveIden)]
pub enum GroupBanned {
    Table,
    GroupId,
    IdentityId,
}
