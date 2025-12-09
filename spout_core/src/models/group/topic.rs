use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any};
use thiserror::Error;

use crate::ids::{GroupId, ProfileId, TopicId};

#[derive(Debug, Error)]
pub enum TopicError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Topic {
    #[sqlx(try_from = "String")]
    pub id: TopicId,
    #[sqlx(try_from = "String")]
    pub group_id: GroupId,
    #[sqlx(try_from = "String")]
    pub profile_id: ProfileId,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct TopicView {
    pub id: TopicId,
    pub group_id: GroupId,
    pub profile_id: ProfileId,
    pub profile_name: String,
    pub profile_desc: String,
    pub created_at: DateTime<Utc>,
}

impl Topic {
    pub async fn create<'a, E>(
        group_id: GroupId,
        profile_id: ProfileId,
        conn: E,
    ) -> Result<Topic, TopicError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = TopicId::new();
        let created_at = Utc::now();

        sqlx::query(
            r#"
      INSERT INTO group_topics (id, group_id, profile_id, created_at)
      VALUES (?, ?, ?, ?)
      "#,
        )
        .bind(id.to_string())
        .bind(group_id.to_string())
        .bind(profile_id.to_string())
        .bind(created_at.to_rfc3339())
        .execute(conn)
        .await?;

        Ok(Topic {
            id,
            group_id,
            profile_id,
            created_at,
        })
    }

    pub async fn by_id(
        id: &TopicId,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Option<TopicView>, TopicError> {
        let row = sqlx::query(
            r#"
      SELECT 
        t.id,
        t.group_id,
        t.profile_id,
        p.name as profile_name,
        p.desc as profile_desc,
        t.created_at
      FROM group_topics t
      INNER JOIN profiles p ON t.profile_id = p.id
      WHERE t.id = ?
      "#,
        )
        .bind(id.to_string())
        .fetch_optional(&mut **conn)
        .await?;

        let topic = match row {
            Some(row) => {
                let id_str: String = row.try_get("id")?;
                let id =
                    TopicId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let group_id_str: String = row.try_get("group_id")?;
                let group_id = GroupId::parse_str(&group_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let profile_id_str: String = row.try_get("profile_id")?;
                let profile_id = ProfileId::parse_str(&profile_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let profile_name: String = row.try_get("profile_name")?;
                let profile_desc: String = row.try_get("profile_desc")?;

                let created_at_str: String = row.try_get("created_at")?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                    .with_timezone(&Utc);

                Some(TopicView {
                    id,
                    group_id,
                    profile_id,
                    profile_name,
                    profile_desc,
                    created_at,
                })
            }
            None => None,
        };

        Ok(topic)
    }

    pub async fn list_for_group(
        group_id: &GroupId,
        limit: i64,
        offset: i64,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<TopicView>, TopicError> {
        let rows = sqlx::query(
            r#"
      SELECT 
        t.id,
        t.group_id,
        t.profile_id,
        p.name as profile_name,
        p.desc as profile_desc,
        t.created_at
      FROM group_topics t
      INNER JOIN profiles p ON t.profile_id = p.id
      WHERE t.group_id = ?
      ORDER BY t.created_at DESC
      LIMIT ? OFFSET ?
      "#,
        )
        .bind(group_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **conn)
        .await?;

        let mut topics = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = TopicId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                GroupId::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id = ProfileId::parse_str(&profile_id_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let profile_name: String = row.try_get("profile_name")?;
            let profile_desc: String = row.try_get("profile_desc")?;

            let created_at_str: String = row.try_get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            topics.push(TopicView {
                id,
                group_id,
                profile_id,
                profile_name,
                profile_desc,
                created_at,
            });
        }

        Ok(topics)
    }

    pub async fn latest_for_group(
        group_id: &GroupId,
        limit: i64,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<TopicView>, TopicError> {
        Self::list_for_group(group_id, limit, 0, conn).await
    }

    pub async fn list_for_profile(
        profile_id: &ProfileId,
        limit: i64,
        offset: i64,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<TopicView>, TopicError> {
        let rows = sqlx::query(
            r#"
      SELECT 
        t.id,
        t.group_id,
        t.profile_id,
        p.name as profile_name,
        p.desc as profile_desc,
        t.created_at
      FROM group_topics t
      INNER JOIN profiles p ON t.profile_id = p.id
      WHERE t.profile_id = ?
      ORDER BY t.created_at DESC
      LIMIT ? OFFSET ?
      "#,
        )
        .bind(profile_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **conn)
        .await?;

        let mut topics = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = TopicId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let group_id_str: String = row.try_get("group_id")?;
            let group_id =
                GroupId::parse_str(&group_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id = ProfileId::parse_str(&profile_id_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let profile_name: String = row.try_get("profile_name")?;
            let profile_desc: String = row.try_get("profile_desc")?;

            let created_at_str: String = row.try_get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            topics.push(TopicView {
                id,
                group_id,
                profile_id,
                profile_name,
                profile_desc,
                created_at,
            });
        }

        Ok(topics)
    }

    pub async fn delete<'a, E>(id: &TopicId, conn: E) -> Result<(), TopicError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      DELETE FROM group_topics
      WHERE id = ?
      "#,
        )
        .bind(id.to_string())
        .execute(conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{profile::Profile, test_utils};

    #[tokio::test]
    async fn creates_and_fetches_topic() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create a profile first
        let profile = Profile::create(
            "Test Topic".to_string(),
            "This is a test topic description".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        // Create topic referencing the profile
        let topic = Topic::create(group_id, profile.id, &mut *conn)
            .await
            .unwrap();

        assert_eq!(topic.group_id, group_id);
        assert_eq!(topic.profile_id, profile.id);

        // Fetch by ID - returns TopicView with profile data
        let fetched = Topic::by_id(&topic.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(fetched.id, topic.id);
        assert_eq!(fetched.profile_id, profile.id);
        assert_eq!(fetched.profile_name, "Test Topic");
        assert_eq!(fetched.profile_desc, "This is a test topic description");
    }

    #[tokio::test]
    async fn lists_topics_for_group() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create profiles
        let profile1 = Profile::create(
            "List Topic 1".to_string(),
            "Desc 1".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();
        let profile2 = Profile::create(
            "List Topic 2".to_string(),
            "Desc 2".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();
        let profile3 = Profile::create(
            "List Topic 3".to_string(),
            "Desc 3".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        // Create topics
        Topic::create(group_id, profile1.id, &mut *conn)
            .await
            .unwrap();
        Topic::create(group_id, profile2.id, &mut *conn)
            .await
            .unwrap();
        Topic::create(group_id, profile3.id, &mut *conn)
            .await
            .unwrap();

        // List all topics for group
        let topics = Topic::list_for_group(&group_id, 10, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(topics.len(), 3);

        // Verify profile data is included
        assert!(topics.iter().any(|t| t.profile_name == "List Topic 1"));
        assert!(topics.iter().any(|t| t.profile_name == "List Topic 2"));
        assert!(topics.iter().any(|t| t.profile_name == "List Topic 3"));

        // Test pagination
        let page1 = Topic::list_for_group(&group_id, 2, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = Topic::list_for_group(&group_id, 2, 2, &mut conn)
            .await
            .unwrap();
        assert_eq!(page2.len(), 1);

        // Topics should be ordered by created_at DESC (newest first)
        assert!(page1[0].created_at >= page1[1].created_at);
    }

    #[tokio::test]
    async fn lists_latest_topics() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create topics with unique profile names
        for i in 1..=5 {
            let profile = Profile::create(
                format!("LatestTopic{}", i), // Unique name
                format!("Description {}", i),
                None,
                &mut *conn,
            )
            .await
            .unwrap();

            Topic::create(group_id, profile.id, &mut *conn)
                .await
                .unwrap();
        }

        // Get latest 3
        let latest = Topic::latest_for_group(&group_id, 3, &mut conn)
            .await
            .unwrap();
        assert_eq!(latest.len(), 3);
        assert!(latest[0].created_at >= latest[1].created_at);
        assert!(latest[1].created_at >= latest[2].created_at);
    }

    #[tokio::test]
    async fn lists_topics_for_profile() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id1 = GroupId::new();
        let group_id2 = GroupId::new();

        let profile = Profile::create(
            "Shared Topic".to_string(),
            "Used in multiple groups".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        // Create topics in different groups using same profile
        Topic::create(group_id1, profile.id, &mut *conn)
            .await
            .unwrap();
        Topic::create(group_id2, profile.id, &mut *conn)
            .await
            .unwrap();

        // List topics by profile
        let topics = Topic::list_for_profile(&profile.id, 10, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(topics.len(), 2);
        assert!(topics.iter().any(|t| t.group_id == group_id1));
        assert!(topics.iter().any(|t| t.group_id == group_id2));

        // All should have the same profile name/desc
        assert!(topics.iter().all(|t| t.profile_name == "Shared Topic"));
    }

    #[tokio::test]
    async fn deletes_topic() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();
        let profile = Profile::create(
            "To Delete".to_string(),
            "Will be deleted".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        let topic = Topic::create(group_id, profile.id, &mut *conn)
            .await
            .unwrap();

        // Verify it exists
        let fetched = Topic::by_id(&topic.id, &mut conn).await.unwrap();
        assert!(fetched.is_some());

        // Delete it
        Topic::delete(&topic.id, &mut *conn).await.unwrap();

        // Verify it's gone
        let fetched = Topic::by_id(&topic.id, &mut conn).await.unwrap();
        assert!(fetched.is_none());
    }
}
