use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000008_create_group_posts_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupPost::Table)
                    .col(
                        ColumnDef::new(GroupPost::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupPost::UserId).string().not_null())
                    .col(ColumnDef::new(GroupPost::TopicId).string().not_null())
                    .col(ColumnDef::new(GroupPost::Title).string().not_null())
                    .col(ColumnDef::new(GroupPost::Body).string().not_null())
                    .col(ColumnDef::new(GroupPost::CreatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_post_user_id")
                            .from(GroupPost::Table, GroupPost::UserId)
                            .to(GroupUser::Table, GroupUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_post_topic_id")
                            .from(GroupPost::Table, GroupPost::TopicId)
                            .to(GroupTopic::Table, GroupTopic::Id)
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupPost::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum GroupPost {
    Table,
    Id,
    UserId,
    TopicId,
    Title,
    Body,
    CreatedAt,
}

#[derive(Iden)]
enum GroupUser {
    Table,
    Id,
}

#[derive(Iden)]
enum GroupTopic {
    Table,
    Id,
}
