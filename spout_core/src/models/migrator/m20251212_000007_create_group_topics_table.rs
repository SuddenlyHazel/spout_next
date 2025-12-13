use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000007_create_group_topics_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupTopic::Table)
                    .col(
                        ColumnDef::new(GroupTopic::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupTopic::GroupId).string().not_null())
                    .col(ColumnDef::new(GroupTopic::ProfileId).string().not_null())
                    .col(ColumnDef::new(GroupTopic::CreatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_topic_group_id")
                            .from(GroupTopic::Table, GroupTopic::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_topic_profile_id")
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

#[derive(Iden)]
pub enum GroupTopic {
    Table,
    Id,
    GroupId,
    ProfileId,
    CreatedAt,
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
}

#[derive(Iden)]
enum Profile {
    Table,
    Id,
}
