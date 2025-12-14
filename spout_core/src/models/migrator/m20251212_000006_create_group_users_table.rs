use sea_orm_migration::{prelude::*, schema::*};

use super::m20251212_000002_create_profiles_table::Profile;
use super::m20251212_000003_create_groups_table::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupUser::Table)
                    .col(pk_uuid(GroupUser::Id))
                    .col(uuid(GroupUser::GroupId))
                    .col(uuid(GroupUser::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-user-group_id")
                            .from(GroupUser::Table, GroupUser::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-user-profile_id")
                            .from(GroupUser::Table, GroupUser::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on (group_id, profile_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_group_users_group_profile_unique")
                    .table(GroupUser::Table)
                    .col(GroupUser::GroupId)
                    .col(GroupUser::ProfileId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on group_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_users_group_id")
                    .table(GroupUser::Table)
                    .col(GroupUser::GroupId)
                    .to_owned(),
            )
            .await?;

        // Create index on profile_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_users_profile_id")
                    .table(GroupUser::Table)
                    .col(GroupUser::ProfileId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GroupUser {
    Table,
    Id,
    GroupId,
    ProfileId,
}
