use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub group_id: Uuid,
    pub profile_id: Uuid,
}

impl User {
    pub async fn add<'a, E>(group_id: Uuid, profile_id: Uuid, conn: E) -> Result<User, UserError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = Uuid::now_v7();

        sqlx::query(
            r#"
      INSERT INTO group_users (id, group_id, profile_id)
      VALUES (?, ?, ?)
      "#,
        )
        .bind(id.to_string())
        .bind(group_id.to_string())
        .bind(profile_id.to_string())
        .execute(conn)
        .await?;

        Ok(User {
            id,
            group_id,
            profile_id,
        })
    }

    pub async fn remove<'a, E>(group_id: Uuid, profile_id: Uuid, conn: E) -> Result<(), UserError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      DELETE FROM group_users
      WHERE group_id = ? AND profile_id = ?
      "#,
        )
        .bind(group_id.to_string())
        .bind(profile_id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn list_for_group(
        group_id: &Uuid,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<User>, UserError> {
        let rows = sqlx::query(
            r#"
      SELECT id, group_id, profile_id
      FROM group_users
      WHERE group_id = ?
      "#,
        )
        .bind(group_id.to_string())
        .fetch_all(&mut **conn)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                Uuid::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id =
                Uuid::parse_str(&profile_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            users.push(User {
                id,
                group_id,
                profile_id,
            });
        }

        Ok(users)
    }

    pub async fn list_for_profile(
        profile_id: &Uuid,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<User>, UserError> {
        let rows = sqlx::query(
            r#"
      SELECT id, group_id, profile_id
      FROM group_users
      WHERE profile_id = ?
      "#,
        )
        .bind(profile_id.to_string())
        .fetch_all(&mut **conn)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                Uuid::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id =
                Uuid::parse_str(&profile_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            users.push(User {
                id,
                group_id,
                profile_id,
            });
        }

        Ok(users)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[tokio::test]
    async fn adds_and_removes_users() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = Uuid::now_v7();
        let profile_id1 = Uuid::now_v7();
        let profile_id2 = Uuid::now_v7();

        // Add users
        let user1 = User::add(group_id, profile_id1, &mut *conn).await.unwrap();
        assert_eq!(user1.group_id, group_id);
        assert_eq!(user1.profile_id, profile_id1);

        let user2 = User::add(group_id, profile_id2, &mut *conn).await.unwrap();
        assert_eq!(user2.group_id, group_id);
        assert_eq!(user2.profile_id, profile_id2);

        // List users for group
        let users = User::list_for_group(&group_id, &mut conn).await.unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.profile_id == profile_id1));
        assert!(users.iter().any(|u| u.profile_id == profile_id2));

        // Remove one user
        User::remove(group_id, profile_id1, &mut *conn)
            .await
            .unwrap();

        let users = User::list_for_group(&group_id, &mut conn).await.unwrap();
        assert_eq!(users.len(), 1);
        assert!(!users.iter().any(|u| u.profile_id == profile_id1));
        assert!(users.iter().any(|u| u.profile_id == profile_id2));
    }

    #[tokio::test]
    async fn lists_groups_for_profile() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id1 = Uuid::now_v7();
        let group_id2 = Uuid::now_v7();
        let profile_id = Uuid::now_v7();

        // Add profile to multiple groups
        User::add(group_id1, profile_id, &mut *conn).await.unwrap();
        User::add(group_id2, profile_id, &mut *conn).await.unwrap();

        // List groups for profile
        let users = User::list_for_profile(&profile_id, &mut conn)
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.group_id == group_id1));
        assert!(users.iter().any(|u| u.group_id == group_id2));

        // Test empty result
        let other_profile = Uuid::now_v7();
        let empty = User::list_for_profile(&other_profile, &mut conn)
            .await
            .unwrap();
        assert!(empty.is_empty());
    }
}
