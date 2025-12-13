use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000006_create_group_users_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupUser::Table)
                    .col(
                        ColumnDef::new(GroupUser::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupUser::GroupId).string().not_null())
                    .col(ColumnDef::new(GroupUser::ProfileId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_user_group_id")
                            .from(GroupUser::Table, GroupUser::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_user_profile_id")
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

#[derive(Iden)]
pub enum GroupUser {
    Table,
    Id,
    GroupId,
    ProfileId,
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
