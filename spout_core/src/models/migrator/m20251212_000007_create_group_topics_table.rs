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
                    .table(GroupTopic::Table)
                    .col(pk_uuid(GroupTopic::Id))
                    .col(uuid(GroupTopic::GroupId))
                    .col(uuid(GroupTopic::ProfileId))
                    .col(timestamp(GroupTopic::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-topic-group_id")
                            .from(GroupTopic::Table, GroupTopic::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-topic-profile_id")
                            .from(GroupTopic::Table, GroupTopic::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on group_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_topics_group_id")
                    .table(GroupTopic::Table)
                    .col(GroupTopic::GroupId)
                    .to_owned(),
            )
            .await?;

        // Create index on profile_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_topics_profile_id")
                    .table(GroupTopic::Table)
                    .col(GroupTopic::ProfileId)
                    .to_owned(),
            )
            .await?;

        // Create index on created_at
        manager
            .create_index(
                Index::create()
                    .name("idx_group_topics_created_at")
                    .table(GroupTopic::Table)
                    .col(GroupTopic::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupTopic::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GroupTopic {
    Table,
    Id,
    GroupId,
    ProfileId,
    CreatedAt,
}
