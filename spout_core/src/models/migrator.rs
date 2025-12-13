use sea_orm_migration::prelude::*;

mod m20251212_000001_create_identity_table;
mod m20251212_000002_create_profiles_table;
mod m20251212_000003_create_groups_table;
mod m20251212_000004_create_group_admins_table;
mod m20251212_000005_create_group_banned_table;
mod m20251212_000006_create_group_users_table;
mod m20251212_000007_create_group_topics_table;
mod m20251212_000008_create_group_posts_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251212_000001_create_identity_table::Migration),
            Box::new(m20251212_000002_create_profiles_table::Migration),
            Box::new(m20251212_000003_create_groups_table::Migration),
            Box::new(m20251212_000004_create_group_admins_table::Migration),
            Box::new(m20251212_000005_create_group_banned_table::Migration),
            Box::new(m20251212_000006_create_group_users_table::Migration),
            Box::new(m20251212_000007_create_group_topics_table::Migration),
            Box::new(m20251212_000008_create_group_posts_table::Migration),
        ]
    }
}

#[cfg(test)]
use sea_orm::{Database, DbErr};

#[tokio::test]
async fn test_migrations_okay() -> Result<(), DbErr> {
    let db = Database::connect("sqlite:file::memory:?cache=shared").await?;
    let schema_manager = SchemaManager::new(&db);

    Migrator::refresh(&db).await?;

    assert!(schema_manager.has_table("identity").await?);
    assert!(schema_manager.has_table("profile").await?);
    assert!(schema_manager.has_table("group").await?);
    assert!(schema_manager.has_table("group_admin").await?);
    assert!(schema_manager.has_table("group_banned").await?);
    assert!(schema_manager.has_table("group_user").await?);
    assert!(schema_manager.has_table("group_topic").await?);
    assert!(schema_manager.has_table("group_post").await?);

    Ok(())
}
