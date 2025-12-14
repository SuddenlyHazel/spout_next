use sea_orm_migration::{prelude::*, schema::*};

use super::m20251212_000006_create_group_users_table::GroupUser;
use super::m20251212_000007_create_group_topics_table::GroupTopic;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupPost::Table)
                    .col(pk_uuid(GroupPost::Id))
                    .col(uuid(GroupPost::UserId))
                    .col(uuid(GroupPost::TopicId))
                    .col(uuid_null(GroupPost::ParentPostId)) // For threaded replies
                    .col(string(GroupPost::Title))
                    .col(string(GroupPost::Body))
                    .col(timestamp(GroupPost::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-post-user_id")
                            .from(GroupPost::Table, GroupPost::UserId)
                            .to(GroupUser::Table, GroupUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-post-topic_id")
                            .from(GroupPost::Table, GroupPost::TopicId)
                            .to(GroupTopic::Table, GroupTopic::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group-post-parent_id")
                            .from(GroupPost::Table, GroupPost::ParentPostId)
                            .to(GroupPost::Table, GroupPost::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on topic_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_posts_topic_id")
                    .table(GroupPost::Table)
                    .col(GroupPost::TopicId)
                    .to_owned(),
            )
            .await?;

        // Create index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_group_posts_user_id")
                    .table(GroupPost::Table)
                    .col(GroupPost::UserId)
                    .to_owned(),
            )
            .await?;

        // Create index on created_at
        manager
            .create_index(
                Index::create()
                    .name("idx_group_posts_created_at")
                    .table(GroupPost::Table)
                    .col(GroupPost::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Create index on parent_post_id for efficient reply lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_group_posts_parent_post_id")
                    .table(GroupPost::Table)
                    .col(GroupPost::ParentPostId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupPost::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GroupPost {
    Table,
    Id,
    UserId,
    TopicId,
    ParentPostId,
    Title,
    Body,
    CreatedAt,
}
