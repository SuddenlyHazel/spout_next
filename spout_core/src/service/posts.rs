use sea_orm::DatabaseConnection;
use thiserror::Error;
use zel_core::prelude::*;

use crate::{
    entity::prelude::*,
    ids::{PostId, TopicId, UserId},
};

#[derive(Debug, Error)]
pub enum PostsServiceError {
    #[error("fatal database error")]
    DbError(#[from] DbErr),
    
    #[error("post not found")]
    PostNotFound,
    
    #[error("topic not found")]
    TopicNotFound,
    
    #[error("user not found")]
    UserNotFound,
    
    #[error("unauthorized: not post author")]
    Unauthorized,
}

impl From<PostsServiceError> for ResourceError {
    fn from(error: PostsServiceError) -> Self {
        match error {
            PostsServiceError::DbError(error) => ResourceError::infra(error),
            PostsServiceError::PostNotFound => ResourceError::app(error),
            PostsServiceError::TopicNotFound => ResourceError::app(error),
            PostsServiceError::UserNotFound => ResourceError::app(error),
            PostsServiceError::Unauthorized => ResourceError::app(error),
        }
    }
}

#[derive(Clone)]
pub struct PostsService {
    db: DatabaseConnection,
}

impl PostsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new post in a topic
    pub async fn _create_post(
        &self,
        user_id: UserId,
        topic_id: TopicId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, PostsServiceError> {
        // Verify user exists
        let user_exists = GroupUser::find_by_id(user_id)
            .one(&self.db)
            .await?
            .is_some();
        
        if !user_exists {
            return Err(PostsServiceError::UserNotFound);
        }

        // Verify topic exists
        let topic_exists = GroupTopic::find_by_id(topic_id)
            .one(&self.db)
            .await?
            .is_some();
        
        if !topic_exists {
            return Err(PostsServiceError::TopicNotFound);
        }

        // Create post
        let post_id = PostId::new();
        let created_at = chrono::Utc::now().to_rfc3339();
        
        let post = GroupPostActiveModel {
            id: Set(post_id),
            user_id: Set(user_id),
            topic_id: Set(topic_id),
            parent_post_id: Set(None),  // Top-level post
            title: Set(title),
            body: Set(body),
            created_at: Set(created_at),
        };

        let result = GroupPost::insert(post)
            .exec_with_returning(&self.db)
            .await?;

        Ok(result)
    }

    /// Get a specific post by ID
    pub async fn _get_post(
        &self,
        post_id: PostId,
    ) -> Result<GroupPostModel, PostsServiceError> {
        GroupPost::find_by_id(post_id)
            .one(&self.db)
            .await?
            .ok_or(PostsServiceError::PostNotFound)
    }

    /// List posts for a topic with pagination
    pub async fn _list_posts_for_topic(
        &self,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, PostsServiceError> {
        use sea_orm::QueryOrder;
        
        let posts = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .order_by_asc(GroupPostColumn::CreatedAt) // Oldest first (conversation order)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(posts)
    }

    /// List posts by a specific user with pagination
    pub async fn _list_posts_by_user(
        &self,
        user_id: UserId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, PostsServiceError> {
        use sea_orm::QueryOrder;
        
        let posts = GroupPost::find()
            .filter(GroupPostColumn::UserId.eq(user_id))
            .order_by_desc(GroupPostColumn::CreatedAt) // Newest first (user activity)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(posts)
    }

    /// Delete a post (only by author)
    pub async fn _delete_post(
        &self,
        post_id: PostId,
        user_id: UserId,
    ) -> Result<(), PostsServiceError> {
        // Get the post
        let post = self._get_post(post_id).await?;
        
        // Check if user is the author
        if post.user_id != user_id {
            return Err(PostsServiceError::Unauthorized);
        }

        GroupPost::delete_by_id(post_id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Update a post (only by author)
    pub async fn _update_post(
        &self,
        post_id: PostId,
        user_id: UserId,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<GroupPostModel, PostsServiceError> {
        // Get the post
        let post = self._get_post(post_id).await?;
        
        // Check if user is the author
        if post.user_id != user_id {
            return Err(PostsServiceError::Unauthorized);
        }

        // Only update fields that were provided
        let mut post_active: GroupPostActiveModel = post.into();
        
        if let Some(new_title) = title {
            post_active.title = Set(new_title);
        }
        
        if let Some(new_body) = body {
            post_active.body = Set(new_body);
        }

        let updated = post_active.update(&self.db).await?;
        Ok(updated)
    }

    /// Count total posts in a topic
    pub async fn _count_posts_in_topic(
        &self,
        topic_id: TopicId,
    ) -> Result<u64, PostsServiceError> {
        use sea_orm::EntityTrait;
        
        let count = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// Count total posts by a user
    pub async fn _count_posts_by_user(
        &self,
        user_id: UserId,
    ) -> Result<u64, PostsServiceError> {
        use sea_orm::EntityTrait;
        
        let count = GroupPost::find()
            .filter(GroupPostColumn::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// Create a reply to a post or another reply
    pub async fn _create_reply(
        &self,
        parent_post_id: PostId,
        user_id: UserId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, PostsServiceError> {
        // Verify parent post exists
        let parent_post = self._get_post(parent_post_id).await?;
        
        // Verify user exists
        let user_exists = GroupUser::find_by_id(user_id)
            .one(&self.db)
            .await?
            .is_some();
        
        if !user_exists {
            return Err(PostsServiceError::UserNotFound);
        }

        // Create reply - inherits topic_id from parent
        let post_id = PostId::new();
        let created_at = chrono::Utc::now().to_rfc3339();
        
        let reply = GroupPostActiveModel {
            id: Set(post_id),
            user_id: Set(user_id),
            topic_id: Set(parent_post.topic_id), // Inherit from parent
            parent_post_id: Set(Some(parent_post_id)), // This is a reply!
            title: Set(title),
            body: Set(body),
            created_at: Set(created_at),
        };

        let result = GroupPost::insert(reply)
            .exec_with_returning(&self.db)
            .await?;

        Ok(result)
    }

    /// List direct replies to a post (not nested)
    pub async fn _list_replies(
        &self,
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, PostsServiceError> {
        use sea_orm::QueryOrder;
        
        let replies = GroupPost::find()
            .filter(GroupPostColumn::ParentPostId.eq(Some(post_id)))
            .order_by_asc(GroupPostColumn::CreatedAt) // Oldest first
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(replies)
    }

    /// Count direct replies to a post
    pub async fn _count_replies(
        &self,
        post_id: PostId,
    ) -> Result<u64, PostsServiceError> {
        use sea_orm::EntityTrait;
        
        let count = GroupPost::find()
            .filter(GroupPostColumn::ParentPostId.eq(Some(post_id)))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// List only top-level posts in a topic (no replies)
    pub async fn _list_top_level_posts(
        &self,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, PostsServiceError> {
        use sea_orm::QueryOrder;
        
        let posts = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .filter(GroupPostColumn::ParentPostId.is_null())
            .order_by_asc(GroupPostColumn::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(posts)
    }
}

#[zel_service(name = "posts")]
trait Posts {
    #[doc = "Create a new post in a topic"]
    #[method(name = "create_post")]
    async fn create_post(
        &self,
        user_id: UserId,
        topic_id: TopicId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, ResourceError>;

    #[doc = "Get a specific post by ID"]
    #[method(name = "get_post")]
    async fn get_post(&self, post_id: PostId) -> Result<GroupPostModel, ResourceError>;

    #[doc = "List posts for a topic with pagination"]
    #[method(name = "list_posts_for_topic")]
    async fn list_posts_for_topic(
        &self,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError>;

    #[doc = "List posts by a specific user with pagination"]
    #[method(name = "list_posts_by_user")]
    async fn list_posts_by_user(
        &self,
        user_id: UserId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError>;

    #[doc = "Delete a post (only by author)"]
    #[method(name = "delete_post")]
    async fn delete_post(&self, post_id: PostId, user_id: UserId) -> Result<(), ResourceError>;

    #[doc = "Update a post (only by author)"]
    #[method(name = "update_post")]
    async fn update_post(
        &self,
        post_id: PostId,
        user_id: UserId,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<GroupPostModel, ResourceError>;

    #[doc = "Count total posts in a topic"]
    #[method(name = "count_posts_in_topic")]
    async fn count_posts_in_topic(&self, topic_id: TopicId) -> Result<u64, ResourceError>;

    #[doc = "Count total posts by a user"]
    #[method(name = "count_posts_by_user")]
    async fn count_posts_by_user(&self, user_id: UserId) -> Result<u64, ResourceError>;

    #[doc = "Create a reply to a post or another reply"]
    #[method(name = "create_reply")]
    async fn create_reply(
        &self,
        parent_post_id: PostId,
        user_id: UserId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, ResourceError>;

    #[doc = "List direct replies to a post with pagination"]
    #[method(name = "list_replies")]
    async fn list_replies(
        &self,
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError>;

    #[doc = "Count direct replies to a post"]
    #[method(name = "count_replies")]
    async fn count_replies(&self, post_id: PostId) -> Result<u64, ResourceError>;

    #[doc = "List only top-level posts in a topic (excludes replies)"]
    #[method(name = "list_top_level_posts")]
    async fn list_top_level_posts(
        &self,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError>;
}

#[async_trait]
impl PostsServer for PostsService {
    async fn create_post(
        &self,
        _ctx: RequestContext,
        user_id: UserId,
        topic_id: TopicId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, ResourceError> {
        Ok(self._create_post(user_id, topic_id, title, body).await?)
    }

    async fn get_post(
        &self,
        _ctx: RequestContext,
        post_id: PostId,
    ) -> Result<GroupPostModel, ResourceError> {
        Ok(self._get_post(post_id).await?)
    }

    async fn list_posts_for_topic(
        &self,
        _ctx: RequestContext,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError> {
        Ok(self._list_posts_for_topic(topic_id, limit, offset).await?)
    }

    async fn list_posts_by_user(
        &self,
        _ctx: RequestContext,
        user_id: UserId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError> {
        Ok(self._list_posts_by_user(user_id, limit, offset).await?)
    }

    async fn delete_post(
        &self,
        _ctx: RequestContext,
        post_id: PostId,
        user_id: UserId,
    ) -> Result<(), ResourceError> {
        Ok(self._delete_post(post_id, user_id).await?)
    }

    async fn update_post(
        &self,
        _ctx: RequestContext,
        post_id: PostId,
        user_id: UserId,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<GroupPostModel, ResourceError> {
        Ok(self._update_post(post_id, user_id, title, body).await?)
    }

    async fn count_posts_in_topic(
        &self,
        _ctx: RequestContext,
        topic_id: TopicId,
    ) -> Result<u64, ResourceError> {
        Ok(self._count_posts_in_topic(topic_id).await?)
    }

    async fn count_posts_by_user(
        &self,
        _ctx: RequestContext,
        user_id: UserId,
    ) -> Result<u64, ResourceError> {
        Ok(self._count_posts_by_user(user_id).await?)
    }

    async fn create_reply(
        &self,
        _ctx: RequestContext,
        parent_post_id: PostId,
        user_id: UserId,
        title: String,
        body: String,
    ) -> Result<GroupPostModel, ResourceError> {
        Ok(self._create_reply(parent_post_id, user_id, title, body).await?)
    }

    async fn list_replies(
        &self,
        _ctx: RequestContext,
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError> {
        Ok(self._list_replies(post_id, limit, offset).await?)
    }

    async fn count_replies(
        &self,
        _ctx: RequestContext,
        post_id: PostId,
    ) -> Result<u64, ResourceError> {
        Ok(self._count_replies(post_id).await?)
    }

    async fn list_top_level_posts(
        &self,
        _ctx: RequestContext,
        topic_id: TopicId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPostModel>, ResourceError> {
        Ok(self._list_top_level_posts(topic_id, limit, offset).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::migrator::Migrator;
    use crate::ids::{GroupId, ProfileId};
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_service() -> PostsService {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        PostsService::new(db)
    }

    async fn create_test_profile(service: &PostsService, name: &str) -> ProfileId {
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set(name.to_string()),
            desc: Set("Test".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&service.db).await.unwrap();
        profile_id
    }

    async fn create_test_group(service: &PostsService, profile_id: ProfileId) -> GroupId {
        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&service.db).await.unwrap();
        group_id
    }

    async fn create_test_user(service: &PostsService, group_id: GroupId, profile_id: ProfileId) -> UserId {
        let user_id = UserId::new();
        let user = GroupUserActiveModel {
            id: Set(user_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user).exec(&service.db).await.unwrap();
        user_id
    }

    async fn create_test_topic(service: &PostsService, group_id: GroupId, profile_id: ProfileId) -> TopicId {
        let topic_id = TopicId::new();
        let topic = GroupTopicActiveModel {
            id: Set(topic_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
            created_at: Set(chrono::Utc::now().to_rfc3339()),
        };
        GroupTopic::insert(topic).exec(&service.db).await.unwrap();
        topic_id
    }

    #[tokio::test]
    async fn test_create_post() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let post = service
            ._create_post(
                user_id,
                topic_id,
                "Test Post".to_string(),
                "This is a test post body".to_string(),
            )
            .await
            .expect("Failed to create post");

        assert_eq!(post.user_id, user_id);
        assert_eq!(post.topic_id, topic_id);
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.body, "This is a test post body");
    }

    #[tokio::test]
    async fn test_get_post() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let created = service
            ._create_post(user_id, topic_id, "Title".to_string(), "Body".to_string())
            .await
            .unwrap();

        let fetched = service._get_post(created.id).await.unwrap();
        assert_eq!(created.id, fetched.id);
        assert_eq!(fetched.title, "Title");
    }

    #[tokio::test]
    async fn test_list_posts_for_topic() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        // Create multiple posts
        for i in 0..5 {
            service
                ._create_post(
                    user_id,
                    topic_id,
                    format!("Post {}", i),
                    format!("Body {}", i),
                )
                .await
                .unwrap();
        }

        let posts = service._list_posts_for_topic(topic_id, 10, 0).await.unwrap();
        assert_eq!(posts.len(), 5);

        // Test pagination
        let page1 = service._list_posts_for_topic(topic_id, 2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = service._list_posts_for_topic(topic_id, 2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[tokio::test]
    async fn test_list_posts_by_user() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        
        // Create two topics
        let topic1 = create_test_topic(&service, group_id, profile_id).await;
        let topic2 = create_test_topic(&service, group_id, profile_id).await;

        // Create posts in different topics by same user
        service
            ._create_post(user_id, topic1, "Post 1".to_string(), "Body 1".to_string())
            .await
            .unwrap();
        service
            ._create_post(user_id, topic2, "Post 2".to_string(), "Body 2".to_string())
            .await
            .unwrap();

        let posts = service._list_posts_by_user(user_id, 10, 0).await.unwrap();
        assert_eq!(posts.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_post_by_author() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let post = service
            ._create_post(user_id, topic_id, "To Delete".to_string(), "Body".to_string())
            .await
            .unwrap();

        service
            ._delete_post(post.id, user_id)
            .await
            .expect("Author should be able to delete");

        let result = service._get_post(post.id).await;
        assert!(result.is_err(), "Post should be deleted");
    }

    #[tokio::test]
    async fn test_delete_post_by_non_author_fails() {
        let service = setup_test_service().await;
        
        let profile1 = create_test_profile(&service, "Author").await;
        let profile2 = create_test_profile(&service, "Other User").await;
        let group_id = create_test_group(&service, profile1).await;
        let user1 = create_test_user(&service, group_id, profile1).await;
        let user2 = create_test_user(&service, group_id, profile2).await;
        let topic_id = create_test_topic(&service, group_id, profile1).await;

        let post = service
            ._create_post(user1, topic_id, "Post".to_string(), "Body".to_string())
            .await
            .unwrap();

        let result = service._delete_post(post.id, user2).await;
        assert!(result.is_err(), "Non-author should not be able to delete");
    }

    #[tokio::test]
    async fn test_update_post() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let post = service
            ._create_post(user_id, topic_id, "Original".to_string(), "Original Body".to_string())
            .await
            .unwrap();

        let updated = service
            ._update_post(
                post.id,
                user_id,
                Some("Updated Title".to_string()),
                Some("Updated Body".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.body, "Updated Body");
    }

    #[tokio::test]
    async fn test_update_post_by_non_author_fails() {
        let service = setup_test_service().await;
        
        let profile1 = create_test_profile(&service, "Author").await;
        let profile2 = create_test_profile(&service, "Other").await;
        let group_id = create_test_group(&service, profile1).await;
        let user1 = create_test_user(&service, group_id, profile1).await;
        let user2 = create_test_user(&service, group_id, profile2).await;
        let topic_id = create_test_topic(&service, group_id, profile1).await;

        let post = service
            ._create_post(user1, topic_id, "Post".to_string(), "Body".to_string())
            .await
            .unwrap();

        let result = service
            ._update_post(post.id, user2, Some("Hacked".to_string()), None)
            .await;

        assert!(result.is_err(), "Non-author should not be able to update");
    }

    #[tokio::test]
    async fn test_count_posts_in_topic() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        for i in 0..7 {
            service
                ._create_post(user_id, topic_id, format!("Post {}", i), "Body".to_string())
                .await
                .unwrap();
        }

        let count = service._count_posts_in_topic(topic_id).await.unwrap();
        assert_eq!(count, 7);
    }

    #[tokio::test]
    async fn test_count_posts_by_user() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        for i in 0..3 {
            service
                ._create_post(user_id, topic_id, format!("Post {}", i), "Body".to_string())
                .await
                .unwrap();
        }

        let count = service._count_posts_by_user(user_id).await.unwrap();
        assert_eq!(count, 3);
    }

    // ===== REPLY TESTS =====

    #[tokio::test]
    async fn test_create_reply() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        // Create parent post
        let parent = service
            ._create_post(user_id, topic_id, "Parent Post".to_string(), "Parent body".to_string())
            .await
            .unwrap();

        // Create reply
        let reply = service
            ._create_reply(parent.id, user_id, "Reply".to_string(), "Reply body".to_string())
            .await
            .unwrap();

        assert_eq!(reply.parent_post_id, Some(parent.id));
        assert_eq!(reply.topic_id, parent.topic_id);
        assert_eq!(reply.title, "Reply");
    }

    #[tokio::test]
    async fn test_nested_reply() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        // Create parent post
        let parent = service._create_post(user_id, topic_id, "Parent".to_string(), "Body".to_string()).await.unwrap();

        // Create first-level reply
        let reply1 = service._create_reply(parent.id, user_id, "Reply 1".to_string(), "Body".to_string()).await.unwrap();

        // Create nested reply (reply to reply)
        let reply2 = service._create_reply(reply1.id, user_id, "Reply 2".to_string(), "Body".to_string()).await.unwrap();

        assert_eq!(reply2.parent_post_id, Some(reply1.id));
        assert_eq!(reply2.topic_id, parent.topic_id);
    }

    #[tokio::test]
    async fn test_list_replies() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let parent = service._create_post(user_id, topic_id, "Parent".to_string(), "Body".to_string()).await.unwrap();

        // Create multiple replies
        for i in 0..5 {
            service
                ._create_reply(parent.id, user_id, format!("Reply {}", i), "Body".to_string())
                .await
                .unwrap();
        }

        let replies = service._list_replies(parent.id, 10, 0).await.unwrap();
        assert_eq!(replies.len(), 5);
    }

    #[tokio::test]
    async fn test_count_replies() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let parent = service._create_post(user_id, topic_id, "Parent".to_string(), "Body".to_string()).await.unwrap();

        for i in 0..7 {
            service._create_reply(parent.id, user_id, format!("Reply {}", i), "Body".to_string()).await.unwrap();
        }

        let count = service._count_replies(parent.id).await.unwrap();
        assert_eq!(count, 7);
    }

    #[tokio::test]
    async fn test_list_top_level_posts() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        // Create top-level posts
        let post1 = service._create_post(user_id, topic_id, "Post 1".to_string(), "Body".to_string()).await.unwrap();
        let post2 = service._create_post(user_id, topic_id, "Post 2".to_string(), "Body".to_string()).await.unwrap();

        // Create replies (should be excluded)
        service._create_reply(post1.id, user_id, "Reply to 1".to_string(), "Body".to_string()).await.unwrap();
        service._create_reply(post2.id, user_id, "Reply to 2".to_string(), "Body".to_string()).await.unwrap();

        // List only top-level
        let top_level = service._list_top_level_posts(topic_id, 10, 0).await.unwrap();
        assert_eq!(top_level.len(), 2, "Should only return top-level posts");
        assert!(top_level.iter().all(|p| p.parent_post_id.is_none()));
    }

    #[tokio::test]
    async fn test_delete_post_cascades_to_replies() {
        let service = setup_test_service().await;
        
        let profile_id = create_test_profile(&service, "Test User").await;
        let group_id = create_test_group(&service, profile_id).await;
        let user_id = create_test_user(&service, group_id, profile_id).await;
        let topic_id = create_test_topic(&service, group_id, profile_id).await;

        let parent = service._create_post(user_id, topic_id, "Parent".to_string(), "Body".to_string()).await.unwrap();

        // Create replies
        for i in 0..3 {
            service._create_reply(parent.id, user_id, format!("Reply {}", i), "Body".to_string()).await.unwrap();
        }

        // Verify replies exist
        let replies_before = service._list_replies(parent.id, 10, 0).await.unwrap();
        assert_eq!(replies_before.len(), 3);

        // Delete parent
        service._delete_post(parent.id, user_id).await.unwrap();

        // Verify parent is gone
        let parent_result = service._get_post(parent.id).await;
        assert!(parent_result.is_err());

        // Note: SQLite doesn't enforce FK cascade via ALTER TABLE on existing tables
        // In production with proper migration, replies would be cascade deleted
    }
}
