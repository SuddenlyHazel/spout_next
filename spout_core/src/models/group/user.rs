use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any};
use thiserror::Error;

use crate::ids::{GroupId, ProfileId, UserId};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(try_from = "String")]
    pub id: UserId,
    #[sqlx(try_from = "String")]
    pub group_id: GroupId,
    #[sqlx(try_from = "String")]
    pub profile_id: ProfileId,
}

impl User {
    pub async fn add<'a, E>(
        group_id: GroupId,
        profile_id: ProfileId,
        conn: E,
    ) -> Result<User, UserError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = UserId::new();

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

    pub async fn remove<'a, E>(
        group_id: GroupId,
        profile_id: ProfileId,
        conn: E,
    ) -> Result<(), UserError>
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
        group_id: &GroupId,
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
            let id = UserId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                GroupId::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id = ProfileId::parse_str(&profile_id_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            users.push(User {
                id,
                group_id,
                profile_id,
            });
        }

        Ok(users)
    }

    pub async fn list_for_profile(
        profile_id: &ProfileId,
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
            let id = UserId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                GroupId::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id = ProfileId::parse_str(&profile_id_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

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

        let group_id = GroupId::new();
        let profile_id1 = ProfileId::new();
        let profile_id2 = ProfileId::new();

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

        let group_id1 = GroupId::new();
        let group_id2 = GroupId::new();
        let profile_id = ProfileId::new();

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
        let other_profile = ProfileId::new();
        let empty = User::list_for_profile(&other_profile, &mut conn)
            .await
            .unwrap();
        assert!(empty.is_empty());
    }
}
