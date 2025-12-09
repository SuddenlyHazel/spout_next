use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any};
use thiserror::Error;

use crate::ids::{PostId, TopicId, UserId};

#[derive(Debug, Error)]
pub enum PostError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Post {
    #[sqlx(try_from = "String")]
    pub id: PostId,
    #[sqlx(try_from = "String")]
    pub user_id: UserId,
    #[sqlx(try_from = "String")]
    pub topic_id: TopicId,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct PostView {
    pub id: PostId,
    pub user_id: UserId,
    pub user_profile_name: String,
    pub topic_id: TopicId,
    pub topic_profile_name: String,
    pub topic_profile_desc: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub async fn create<'a, E>(
        user_id: UserId,
        topic_id: TopicId,
        title: String,
        body: String,
        conn: E,
    ) -> Result<Post, PostError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = PostId::new();
        let created_at = Utc::now();

        sqlx::query(
            r#"
      INSERT INTO group_posts (id, user_id, topic_id, title, body, created_at)
      VALUES (?, ?, ?, ?, ?, ?)
      "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(topic_id.to_string())
        .bind(&title)
        .bind(&body)
        .bind(created_at.to_rfc3339())
        .execute(conn)
        .await?;

        Ok(Post {
            id,
            user_id,
            topic_id,
            title,
            body,
            created_at,
        })
    }

    pub async fn by_id(
        id: &PostId,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Option<PostView>, PostError> {
        let row = sqlx::query(
            r#"
      SELECT
        p.id,
        p.user_id,
        up.name as user_profile_name,
        p.topic_id,
        tp.name as topic_profile_name,
        tp.desc as topic_profile_desc,
        p.title,
        p.body,
        p.created_at
      FROM group_posts p
      INNER JOIN group_users u ON p.user_id = u.id
      INNER JOIN profiles up ON u.profile_id = up.id
      INNER JOIN group_topics t ON p.topic_id = t.id
      INNER JOIN profiles tp ON t.profile_id = tp.id
      WHERE p.id = ?
      "#,
        )
        .bind(id.to_string())
        .fetch_optional(&mut **conn)
        .await?;

        let post = match row {
            Some(row) => {
                let id_str: String = row.try_get("id")?;
                let id =
                    PostId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let user_id_str: String = row.try_get("user_id")?;
                let user_id = UserId::parse_str(&user_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let user_profile_name: String = row.try_get("user_profile_name")?;

                let topic_id_str: String = row.try_get("topic_id")?;
                let topic_id = TopicId::parse_str(&topic_id_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let topic_profile_name: String = row.try_get("topic_profile_name")?;
                let topic_profile_desc: String = row.try_get("topic_profile_desc")?;

                let title: String = row.try_get("title")?;
                let body: String = row.try_get("body")?;

                let created_at_str: String = row.try_get("created_at")?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                    .with_timezone(&Utc);

                Some(PostView {
                    id,
                    user_id,
                    user_profile_name,
                    topic_id,
                    topic_profile_name,
                    topic_profile_desc,
                    title,
                    body,
                    created_at,
                })
            }
            None => None,
        };

        Ok(post)
    }

    pub async fn list_for_topic(
        topic_id: &TopicId,
        limit: i64,
        offset: i64,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<PostView>, PostError> {
        let rows = sqlx::query(
            r#"
      SELECT
        p.id,
        p.user_id,
        up.name as user_profile_name,
        p.topic_id,
        tp.name as topic_profile_name,
        tp.desc as topic_profile_desc,
        p.title,
        p.body,
        p.created_at
      FROM group_posts p
      INNER JOIN group_users u ON p.user_id = u.id
      INNER JOIN profiles up ON u.profile_id = up.id
      INNER JOIN group_topics t ON p.topic_id = t.id
      INNER JOIN profiles tp ON t.profile_id = tp.id
      WHERE p.topic_id = ?
      ORDER BY p.created_at ASC
      LIMIT ? OFFSET ?
      "#,
        )
        .bind(topic_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **conn)
        .await?;

        let mut posts = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = PostId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let user_id_str: String = row.try_get("user_id")?;
            let user_id =
                UserId::parse_str(&user_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let user_profile_name: String = row.try_get("user_profile_name")?;

            let topic_id_str: String = row.try_get("topic_id")?;
            let topic_id =
                TopicId::parse_str(&topic_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let topic_profile_name: String = row.try_get("topic_profile_name")?;
            let topic_profile_desc: String = row.try_get("topic_profile_desc")?;

            let title: String = row.try_get("title")?;
            let body: String = row.try_get("body")?;

            let created_at_str: String = row.try_get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            posts.push(PostView {
                id,
                user_id,
                user_profile_name,
                topic_id,
                topic_profile_name,
                topic_profile_desc,
                title,
                body,
                created_at,
            });
        }

        Ok(posts)
    }

    pub async fn list_for_user(
        user_id: &UserId,
        limit: i64,
        offset: i64,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<PostView>, PostError> {
        let rows = sqlx::query(
            r#"
      SELECT
        p.id,
        p.user_id,
        up.name as user_profile_name,
        p.topic_id,
        tp.name as topic_profile_name,
        tp.desc as topic_profile_desc,
        p.title,
        p.body,
        p.created_at
      FROM group_posts p
      INNER JOIN group_users u ON p.user_id = u.id
      INNER JOIN profiles up ON u.profile_id = up.id
      INNER JOIN group_topics t ON p.topic_id = t.id
      INNER JOIN profiles tp ON t.profile_id = tp.id
      WHERE p.user_id = ?
      ORDER BY p.created_at DESC
      LIMIT ? OFFSET ?
      "#,
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **conn)
        .await?;

        let mut posts = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let id = PostId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let user_id_str: String = row.try_get("user_id")?;
            let user_id =
                UserId::parse_str(&user_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let user_profile_name: String = row.try_get("user_profile_name")?;

            let topic_id_str: String = row.try_get("topic_id")?;
            let topic_id =
                TopicId::parse_str(&topic_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let topic_profile_name: String = row.try_get("topic_profile_name")?;
            let topic_profile_desc: String = row.try_get("topic_profile_desc")?;

            let title: String = row.try_get("title")?;
            let body: String = row.try_get("body")?;

            let created_at_str: String = row.try_get("created_at")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            posts.push(PostView {
                id,
                user_id,
                user_profile_name,
                topic_id,
                topic_profile_name,
                topic_profile_desc,
                title,
                body,
                created_at,
            });
        }

        Ok(posts)
    }

    pub async fn delete<'a, E>(id: &PostId, conn: E) -> Result<(), PostError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      DELETE FROM group_posts
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
    use crate::{
        group::topic::Topic, group::user::User, ids::GroupId, profile::Profile, test_utils,
    };

    #[tokio::test]
    async fn creates_and_fetches_post() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create user profile
        let user_profile = Profile::create(
            "Post Test User".to_string(),
            "User for post test".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        // Create user
        let user = User::add(group_id, user_profile.id, &mut *conn)
            .await
            .unwrap();

        // Create topic profile
        let topic_profile = Profile::create(
            "Post Test Topic".to_string(),
            "Topic description".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        // Create topic
        let topic = Topic::create(group_id, topic_profile.id, &mut *conn)
            .await
            .unwrap();

        // Create post
        let post = Post::create(
            user.id,
            topic.id,
            "Post Title".to_string(),
            "Post body content".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        assert_eq!(post.user_id, user.id);
        assert_eq!(post.topic_id, topic.id);
        assert_eq!(post.title, "Post Title");
        assert_eq!(post.body, "Post body content");

        // Fetch by ID
        let fetched = Post::by_id(&post.id, &mut conn).await.unwrap().unwrap();
        assert_eq!(fetched.id, post.id);
        assert_eq!(fetched.title, "Post Title");
        assert_eq!(fetched.body, "Post body content");
    }

    #[tokio::test]
    async fn lists_posts_for_topic() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create user profile
        let user_profile = Profile::create(
            "List Posts User".to_string(),
            "User for listing posts".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        let user = User::add(group_id, user_profile.id, &mut *conn)
            .await
            .unwrap();

        let topic_profile = Profile::create(
            "List Posts Topic".to_string(),
            "Description".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        let topic = Topic::create(group_id, topic_profile.id, &mut *conn)
            .await
            .unwrap();

        // Create multiple posts
        Post::create(
            user.id,
            topic.id,
            "Post 1".to_string(),
            "Body 1".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        Post::create(
            user.id,
            topic.id,
            "Post 2".to_string(),
            "Body 2".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        Post::create(
            user.id,
            topic.id,
            "Post 3".to_string(),
            "Body 3".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        // List posts for topic
        let posts = Post::list_for_topic(&topic.id, 10, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(posts.len(), 3);

        // Posts should be ordered by created_at ASC (oldest first, like a conversation)
        assert!(posts[0].created_at <= posts[1].created_at);
        assert!(posts[1].created_at <= posts[2].created_at);

        // Test pagination
        let page1 = Post::list_for_topic(&topic.id, 2, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = Post::list_for_topic(&topic.id, 2, 2, &mut conn)
            .await
            .unwrap();
        assert_eq!(page2.len(), 1);
    }

    #[tokio::test]
    async fn lists_posts_for_user() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create user profile
        let user_profile = Profile::create(
            "User Posts Test User".to_string(),
            "User for user posts test".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        let user = User::add(group_id, user_profile.id, &mut *conn)
            .await
            .unwrap();

        // Create two topics
        let topic1_profile = Profile::create(
            "User Posts Topic 1".to_string(),
            "Description 1".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();
        let topic1 = Topic::create(group_id, topic1_profile.id, &mut *conn)
            .await
            .unwrap();

        let topic2_profile = Profile::create(
            "User Posts Topic 2".to_string(),
            "Description 2".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();
        let topic2 = Topic::create(group_id, topic2_profile.id, &mut *conn)
            .await
            .unwrap();

        // Create posts in different topics by same user
        Post::create(
            user.id,
            topic1.id,
            "Post in Topic 1".to_string(),
            "Body 1".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        Post::create(
            user.id,
            topic2.id,
            "Post in Topic 2".to_string(),
            "Body 2".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        // List posts by user
        let posts = Post::list_for_user(&user.id, 10, 0, &mut conn)
            .await
            .unwrap();
        assert_eq!(posts.len(), 2);
        assert!(posts.iter().any(|p| p.topic_id == topic1.id));
        assert!(posts.iter().any(|p| p.topic_id == topic2.id));
    }

    #[tokio::test]
    async fn deletes_post() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let group_id = GroupId::new();

        // Create user profile
        let user_profile = Profile::create(
            "Delete Post User".to_string(),
            "User for delete test".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();

        let user = User::add(group_id, user_profile.id, &mut *conn)
            .await
            .unwrap();

        let topic_profile = Profile::create(
            "Delete Post Topic".to_string(),
            "Description".to_string(),
            None,
            &mut *conn,
        )
        .await
        .unwrap();
        let topic = Topic::create(group_id, topic_profile.id, &mut *conn)
            .await
            .unwrap();

        let post = Post::create(
            user.id,
            topic.id,
            "To Delete".to_string(),
            "Will be deleted".to_string(),
            &mut *conn,
        )
        .await
        .unwrap();

        // Verify it exists
        let fetched = Post::by_id(&post.id, &mut conn).await.unwrap();
        assert!(fetched.is_some());

        // Delete it
        Post::delete(&post.id, &mut *conn).await.unwrap();

        // Verify it's gone
        let fetched = Post::by_id(&post.id, &mut conn).await.unwrap();
        assert!(fetched.is_none());
    }
}
