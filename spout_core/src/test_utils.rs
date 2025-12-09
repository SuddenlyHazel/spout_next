use sqlx::{any::install_drivers, sqlite, AnyPool};

/// Initialize SQLx drivers for testing. This should be called once before using any database connections.
/// It's safe to call multiple times as it will only install drivers once.
pub fn init_test_drivers() {
    install_drivers(&[sqlite::any::DRIVER]).ok();
}

/// Create a new in-memory SQLite database pool for testing.
/// Each call creates a fresh, isolated database instance.
///
/// # Example
/// ```
/// use spout_core::test_utils;
///
/// #[tokio::test]
/// async fn my_test() {
///     test_utils::init_test_drivers();
///     let pool = test_utils::create_test_db().await;
///     // Run migrations and tests...
/// }
/// ```
pub async fn create_test_db() -> AnyPool {
    // Using file::memory:?cache=shared allows multiple connections to share the same in-memory database
    // Honestly, I'm not sure why we suddenly need this. In the past the ::sqlite:memory:: string has worked fine.
    // My best guess it something to do with using the sqlx::Any drivers..
    AnyPool::connect("sqlite:file::memory:?cache=shared")
        .await
        .expect("Failed to create test database")
}

/// Create a new in-memory SQLite database pool with migrations already applied.
/// This is a convenience function for tests that need a fully set up database.
///
/// # Example
/// ```
/// use spout_core::test_utils;
///
/// #[tokio::test]
/// async fn my_test() {
///     test_utils::init_test_drivers();
///     let pool = test_utils::create_test_db_with_migrations().await;
///     // Database is ready to use!
/// }
/// ```
pub async fn create_test_db_with_migrations() -> AnyPool {
    let pool = create_test_db().await;

    // Run all migrations
    crate::profile::migrate_up(pool.clone())
        .await
        .expect("Failed to run profile migrations");

    crate::identity::migrate_up(pool.clone())
        .await
        .expect("Failed to run identity migrations");

    pool
}
