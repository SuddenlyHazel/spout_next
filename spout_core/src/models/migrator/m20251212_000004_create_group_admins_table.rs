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
                    .table(GroupAdmin::Table)
                    .col(uuid(GroupAdmin::GroupId))
                    .col(uuid(GroupAdmin::IdentityId))
                    .primary_key(
                        Index::create()
                            .col(GroupAdmin::GroupId)
                            .col(GroupAdmin::IdentityId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-admin-group_id")
                            .from(GroupAdmin::Table, GroupAdmin::GroupId)
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
                    .name("idx_group_admins_identity_id")
                    .table(GroupAdmin::Table)
                    .col(GroupAdmin::IdentityId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupAdmin::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GroupAdmin {
    Table,
    GroupId,
    IdentityId,
}
