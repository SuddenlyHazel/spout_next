use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251212_000004_create_group_admins_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupAdmin::Table)
                    .col(ColumnDef::new(GroupAdmin::GroupId).string().not_null())
                    .col(ColumnDef::new(GroupAdmin::IdentityId).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(GroupAdmin::GroupId)
                            .col(GroupAdmin::IdentityId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_admin_group_id")
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

#[derive(Iden)]
pub enum GroupAdmin {
    Table,
    GroupId,
    IdentityId,
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
}
