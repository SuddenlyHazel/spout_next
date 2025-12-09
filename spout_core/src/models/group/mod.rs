pub mod user;

use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any, AnyPool};
use thiserror::Error;
use uuid::Uuid;

use crate::error::MigrationError;

#[derive(Debug, Error)]
pub enum GroupError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub profile_id: Uuid,
    #[sqlx(skip)]
    pub admin_identities: Vec<Uuid>,
    #[sqlx(skip)]
    pub banned_identities: Vec<Uuid>,
    #[sqlx(skip)]
    pub users: Vec<user::User>,
}

impl Group {
    pub async fn create<'a, E>(profile_id: Uuid, conn: E) -> Result<Group, GroupError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = Uuid::now_v7();

        sqlx::query(
            r#"
      INSERT INTO groups (id, profile_id)
      VALUES (?, ?)
      "#,
        )
        .bind(id.to_string())
        .bind(profile_id.to_string())
        .execute(conn)
        .await?;

        Ok(Group {
            id,
            profile_id,
            admin_identities: Vec::new(),
            banned_identities: Vec::new(),
            users: Vec::new(),
        })
    }

    pub async fn by_id(
        id: &Uuid,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Option<Group>, GroupError> {
        let row = sqlx::query(
            r#"
      SELECT id, profile_id
      FROM groups
      WHERE id = ?
      "#,
        )
        .bind(id.to_string())
        .fetch_optional(&mut **conn)
        .await?;

        let group = match row {
            Some(row) => {
                let id_str: String = row.try_get("id")?;
                let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                let profile_id_str: String = row.try_get("profile_id")?;
                let profile_id = Uuid::parse_str(&profile_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                // Load admin identities
                let admin_rows = sqlx::query(
                    r#"
          SELECT identity_id
          FROM group_admins
          WHERE group_id = ?
          "#,
                )
                .bind(id.to_string())
                .fetch_all(&mut **conn)
                .await?;

                let mut admin_identities = Vec::new();
                for row in admin_rows {
                    let identity_id_str: String = row.try_get("identity_id")?;
                    let identity_id = Uuid::parse_str(&identity_id_str)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                    admin_identities.push(identity_id);
                }

                // Load banned identities
                let banned_rows = sqlx::query(
                    r#"
          SELECT identity_id
          FROM group_banned
          WHERE group_id = ?
          "#,
                )
                .bind(id.to_string())
                .fetch_all(&mut **conn)
                .await?;

                let mut banned_identities = Vec::new();
                for row in banned_rows {
                    let identity_id_str: String = row.try_get("identity_id")?;
                    let identity_id = Uuid::parse_str(&identity_id_str)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                    banned_identities.push(identity_id);
                }

                // Load users
                let users = user::User::list_for_group(&id, conn)
                    .await
                    .map_err(|e| match e {
                        user::UserError::DatabaseError(db_err) => GroupError::DatabaseError(db_err),
                        user::UserError::InvalidUuid(uuid_err) => GroupError::InvalidUuid(uuid_err),
                    })?;

                Some(Group {
                    id,
                    profile_id,
                    admin_identities,
                    banned_identities,
                    users,
                })
            }
            None => None,
        };

        Ok(group)
    }

    pub async fn add_admin<'a, E>(
        group_id: Uuid,
        identity_id: Uuid,
        conn: E,
    ) -> Result<(), GroupError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      INSERT INTO group_admins (group_id, identity_id)
      VALUES (?, ?)
      "#,
        )
        .bind(group_id.to_string())
        .bind(identity_id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn remove_admin<'a, E>(
        group_id: Uuid,
        identity_id: Uuid,
        conn: E,
    ) -> Result<(), GroupError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      DELETE FROM group_admins
      WHERE group_id = ? AND identity_id = ?
      "#,
        )
        .bind(group_id.to_string())
        .bind(identity_id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn add_banned<'a, E>(
        group_id: Uuid,
        identity_id: Uuid,
        conn: E,
    ) -> Result<(), GroupError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      INSERT INTO group_banned (group_id, identity_id)
      VALUES (?, ?)
      "#,
        )
        .bind(group_id.to_string())
        .bind(identity_id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn remove_banned<'a, E>(
        group_id: Uuid,
        identity_id: Uuid,
        conn: E,
    ) -> Result<(), GroupError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      DELETE FROM group_banned
      WHERE group_id = ? AND identity_id = ?
      "#,
        )
        .bind(group_id.to_string())
        .bind(identity_id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn list_for_identity(
        identity_id: &Uuid,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<Group>, GroupError> {
        // Find all groups where the identity is an admin
        let rows = sqlx::query(
            r#"
      SELECT DISTINCT g.id, g.profile_id
      FROM groups g
      INNER JOIN group_admins ga ON g.id = ga.group_id
      WHERE ga.identity_id = ?
      "#,
        )
        .bind(identity_id.to_string())
        .fetch_all(&mut **conn)
        .await?;

        let mut groups = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id =
                Uuid::parse_str(&profile_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            // Load admin identities
            let admin_rows = sqlx::query(
                r#"
        SELECT identity_id
        FROM group_admins
        WHERE group_id = ?
        "#,
            )
            .bind(id.to_string())
            .fetch_all(&mut **conn)
            .await?;

            let mut admin_identities = Vec::new();
            for admin_row in admin_rows {
                let admin_id_str: String = admin_row.try_get("identity_id")?;
                let admin_id =
                    Uuid::parse_str(&admin_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                admin_identities.push(admin_id);
            }

            // Load banned identities
            let banned_rows = sqlx::query(
                r#"
        SELECT identity_id
        FROM group_banned
        WHERE group_id = ?
        "#,
            )
            .bind(id.to_string())
            .fetch_all(&mut **conn)
            .await?;

            let mut banned_identities = Vec::new();
            for banned_row in banned_rows {
                let banned_id_str: String = banned_row.try_get("identity_id")?;
                let banned_id = Uuid::parse_str(&banned_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                banned_identities.push(banned_id);
            }

            // Load users
            let users = user::User::list_for_group(&id, conn)
                .await
                .map_err(|e| match e {
                    user::UserError::DatabaseError(db_err) => GroupError::DatabaseError(db_err),
                    user::UserError::InvalidUuid(uuid_err) => GroupError::InvalidUuid(uuid_err),
                })?;

            groups.push(Group {
                id,
                profile_id,
                admin_identities,
                banned_identities,
                users,
            });
        }

        Ok(groups)
    }
}

pub async fn migrate_up(conn: AnyPool) -> Result<(), MigrationError> {
    let mut conn = conn.acquire().await?;
    migrations::create_groups_table(&mut conn).await?;
    migrations::create_group_admins_table(&mut conn).await?;
    migrations::create_group_banned_table(&mut conn).await?;
    migrations::create_group_users_table(&mut conn).await?;

    Ok(())
}

mod migrations {
    use sqlx::{pool::PoolConnection, Any};

    use crate::error::MigrationError;

    pub async fn create_groups_table(conn: &mut PoolConnection<Any>) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS groups (
        id TEXT PRIMARY KEY NOT NULL,
        profile_id TEXT NOT NULL
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE INDEX IF NOT EXISTS idx_groups_profile_id ON groups(profile_id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }

    pub async fn create_group_admins_table(
        conn: &mut PoolConnection<Any>,
    ) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS group_admins (
        group_id TEXT NOT NULL,
        identity_id TEXT NOT NULL,
        PRIMARY KEY (group_id, identity_id)
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE INDEX IF NOT EXISTS idx_group_admins_identity_id ON group_admins(identity_id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }

    pub async fn create_group_banned_table(
        conn: &mut PoolConnection<Any>,
    ) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS group_banned (
        group_id TEXT NOT NULL,
        identity_id TEXT NOT NULL,
        PRIMARY KEY (group_id, identity_id)
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE INDEX IF NOT EXISTS idx_group_banned_identity_id ON group_banned(identity_id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }

    pub async fn create_group_users_table(
        conn: &mut PoolConnection<Any>,
    ) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS group_users (
        id TEXT PRIMARY KEY NOT NULL,
        group_id TEXT NOT NULL,
        profile_id TEXT NOT NULL,
        UNIQUE(group_id, profile_id)
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE INDEX IF NOT EXISTS idx_group_users_group_id ON group_users(group_id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE INDEX IF NOT EXISTS idx_group_users_profile_id ON group_users(profile_id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[tokio::test]
    async fn creates_group_and_manages_admins() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let profile_id = Uuid::now_v7();
        let identity_id1 = Uuid::now_v7();
        let identity_id2 = Uuid::now_v7();

        // Create a group
        let group = Group::create(profile_id, &mut *conn).await.unwrap();
        assert_eq!(group.profile_id, profile_id);
        assert!(group.admin_identities.is_empty());

        // Add first admin
        Group::add_admin(group.id, identity_id1, &mut *conn)
            .await
            .unwrap();

        // Verify admin was added
        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.admin_identities.len(), 1);
        assert!(loaded_group.admin_identities.contains(&identity_id1));

        // Add second admin
        Group::add_admin(group.id, identity_id2, &mut *conn)
            .await
            .unwrap();

        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.admin_identities.len(), 2);
        assert!(loaded_group.admin_identities.contains(&identity_id1));
        assert!(loaded_group.admin_identities.contains(&identity_id2));

        // Remove first admin
        Group::remove_admin(group.id, identity_id1, &mut *conn)
            .await
            .unwrap();

        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.admin_identities.len(), 1);
        assert!(!loaded_group.admin_identities.contains(&identity_id1));
        assert!(loaded_group.admin_identities.contains(&identity_id2));
    }

    #[tokio::test]
    async fn manages_banned_identities() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let profile_id = Uuid::now_v7();
        let identity_id1 = Uuid::now_v7();
        let identity_id2 = Uuid::now_v7();

        let group = Group::create(profile_id, &mut *conn).await.unwrap();

        // Add banned identities
        Group::add_banned(group.id, identity_id1, &mut *conn)
            .await
            .unwrap();
        Group::add_banned(group.id, identity_id2, &mut *conn)
            .await
            .unwrap();

        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.banned_identities.len(), 2);
        assert!(loaded_group.banned_identities.contains(&identity_id1));
        assert!(loaded_group.banned_identities.contains(&identity_id2));

        // Remove one banned identity
        Group::remove_banned(group.id, identity_id1, &mut *conn)
            .await
            .unwrap();

        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.banned_identities.len(), 1);
        assert!(!loaded_group.banned_identities.contains(&identity_id1));
        assert!(loaded_group.banned_identities.contains(&identity_id2));
    }

    #[tokio::test]
    async fn lists_groups_for_identity() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let profile_id1 = Uuid::now_v7();
        let profile_id2 = Uuid::now_v7();
        let identity_id = Uuid::now_v7();

        // Create two groups
        let group1 = Group::create(profile_id1, &mut *conn).await.unwrap();
        let group2 = Group::create(profile_id2, &mut *conn).await.unwrap();

        // Add identity as admin to both groups
        Group::add_admin(group1.id, identity_id, &mut *conn)
            .await
            .unwrap();
        Group::add_admin(group2.id, identity_id, &mut *conn)
            .await
            .unwrap();

        // List groups for identity
        let groups = Group::list_for_identity(&identity_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(groups.len(), 2);
        assert!(groups.iter().any(|g| g.id == group1.id));
        assert!(groups.iter().any(|g| g.id == group2.id));

        // Remove identity from one group
        Group::remove_admin(group1.id, identity_id, &mut *conn)
            .await
            .unwrap();

        let groups = Group::list_for_identity(&identity_id, &mut conn)
            .await
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert!(groups.iter().any(|g| g.id == group2.id));
        assert!(!groups.iter().any(|g| g.id == group1.id));

        // Test with identity that has no groups
        let other_identity = Uuid::now_v7();
        let empty_groups = Group::list_for_identity(&other_identity, &mut conn)
            .await
            .unwrap();
        assert!(empty_groups.is_empty());
    }

    #[tokio::test]
    async fn manages_group_users() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let profile_id = Uuid::now_v7();
        let user_profile1 = Uuid::now_v7();
        let user_profile2 = Uuid::now_v7();

        // Create a group
        let group = Group::create(profile_id, &mut *conn).await.unwrap();

        // Add users
        user::User::add(group.id, user_profile1, &mut *conn)
            .await
            .unwrap();
        user::User::add(group.id, user_profile2, &mut *conn)
            .await
            .unwrap();

        // Load group and verify users
        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.users.len(), 2);
        assert!(loaded_group
            .users
            .iter()
            .any(|u| u.profile_id == user_profile1));
        assert!(loaded_group
            .users
            .iter()
            .any(|u| u.profile_id == user_profile2));

        // Remove one user
        user::User::remove(group.id, user_profile1, &mut *conn)
            .await
            .unwrap();

        let loaded_group = Group::by_id(&group.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(loaded_group.users.len(), 1);
        assert!(!loaded_group
            .users
            .iter()
            .any(|u| u.profile_id == user_profile1));
        assert!(loaded_group
            .users
            .iter()
            .any(|u| u.profile_id == user_profile2));
    }
}
