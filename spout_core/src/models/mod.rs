use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use crate::config::SpoutConfig;

pub mod migrator;

pub async fn open_or_create_db(config: &SpoutConfig) -> DatabaseConnection {
    // Use display() to convert PathBuf to string representation
    let connection_string = format!("sqlite://{}?mode=rwc", config.database_path.display());

    Database::connect(&connection_string)
        .await
        .expect("Failed to connect to database")
}

pub async fn migrate_up(db: DatabaseConnection) {
    migrator::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
}
