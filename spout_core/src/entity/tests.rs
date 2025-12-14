#[cfg(test)]
mod entity_tests {
    use crate::entity::prelude::*;
    use crate::ids::*;
    use crate::models::migrator::Migrator;
    use sea_orm_migration::MigratorTrait;

    /// Test helper to create and migrate an in-memory database
    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run all migrations
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    #[tokio::test]
    async fn test_create_and_find_profile() {
        let db = setup_test_db().await;

        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Test User".to_string()),
            desc: Set("Test Description".to_string()),
            picture: Set(None),
        };

        // Insert profile
        let insert_result = Profile::insert(profile)
            .exec(&db)
            .await
            .expect("Failed to insert profile");

        // Find by ID
        let found = Profile::find_by_id(profile_id)
            .one(&db)
            .await
            .expect("Failed to query profile");

        assert!(found.is_some());
        let found_profile = found.unwrap();
        assert_eq!(found_profile.id, profile_id);
        assert_eq!(found_profile.name, "Test User");
        assert_eq!(found_profile.desc, "Test Description");
    }

    #[tokio::test]
    async fn test_create_profile_with_picture() {
        let db = setup_test_db().await;

        let profile_id = ProfileId::new();
        let picture_data = vec![1, 2, 3, 4, 5];

        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("User with Picture".to_string()),
            desc: Set("Has a picture".to_string()),
            picture: Set(Some(picture_data.clone())),
        };

        Profile::insert(profile).exec(&db).await.unwrap();

        let found = Profile::find_by_id(profile_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.picture, Some(picture_data));
    }

    #[tokio::test]
    async fn test_filter_profiles_by_name() {
        let db = setup_test_db().await;

        // Create multiple profiles
        for i in 0..3 {
            let profile = ProfileActiveModel {
                id: Set(ProfileId::new()),
                name: Set(format!("User {}", i)),
                desc: Set(format!("Description {}", i)),
                picture: Set(None),
            };
            Profile::insert(profile).exec(&db).await.unwrap();
        }

        // Find specific profile by name
        let found = Profile::find()
            .filter(ProfileColumn::Name.eq("User 1"))
            .one(&db)
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "User 1");
    }

    #[tokio::test]
    async fn test_create_group_with_profile() {
        let db = setup_test_db().await;

        // Create profile first
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Group Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        // Create group
        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Verify group was created
        let found = Group::find_by_id(group_id).one(&db).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().profile_id, profile_id);
    }

    #[tokio::test]
    async fn test_group_admin_relationship() {
        let db = setup_test_db().await;

        // Create profile and group
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Admin User".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Add admin
        let admin = GroupAdminActiveModel {
            group_id: Set(group_id),
            identity_id: Set(profile_id),
        };
        GroupAdmin::insert(admin).exec(&db).await.unwrap();

        // Query admins for this group
        let admins = GroupAdmin::find()
            .filter(GroupAdminColumn::GroupId.eq(group_id))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(admins.len(), 1);
        assert_eq!(admins[0].identity_id, profile_id);
    }

    #[tokio::test]
    async fn test_group_user_unique_constraint() {
        let db = setup_test_db().await;

        // Setup profile and group
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("User".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Add user to group
        let user1 = GroupUserActiveModel {
            id: Set(UserId::new()),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user1).exec(&db).await.unwrap();

        // Try to add same profile to same group again (should succeed with different user id)
        let user2 = GroupUserActiveModel {
            id: Set(UserId::new()),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };

        // This should fail due to unique constraint on (group_id, profile_id)
        let result = GroupUser::insert(user2).exec(&db).await;
        assert!(result.is_err(), "Should fail due to unique constraint");
    }

    #[tokio::test]
    async fn test_cascade_delete_group() {
        let db = setup_test_db().await;

        // Setup profile and group
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Add admin, user, and banned user
        let admin = GroupAdminActiveModel {
            group_id: Set(group_id),
            identity_id: Set(profile_id),
        };
        GroupAdmin::insert(admin).exec(&db).await.unwrap();

        let user = GroupUserActiveModel {
            id: Set(UserId::new()),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user).exec(&db).await.unwrap();

        // Delete the group
        Group::delete_by_id(group_id).exec(&db).await.unwrap();

        // Verify cascade delete worked
        let admins = GroupAdmin::find()
            .filter(GroupAdminColumn::GroupId.eq(group_id))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(admins.len(), 0, "Admins should be cascade deleted");

        let users = GroupUser::find()
            .filter(GroupUserColumn::GroupId.eq(group_id))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(users.len(), 0, "Users should be cascade deleted");
    }

    #[tokio::test]
    async fn test_topic_and_posts() {
        let db = setup_test_db().await;

        // Setup profile, group, and user
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Poster".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        let user_id = UserId::new();
        let user = GroupUserActiveModel {
            id: Set(user_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user).exec(&db).await.unwrap();

        // Create topic
        let topic_id = TopicId::new();
        let topic = GroupTopicActiveModel {
            id: Set(topic_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
            created_at: Set("2024-01-01T00:00:00Z".to_string()),
        };
        GroupTopic::insert(topic).exec(&db).await.unwrap();

        // Create post
        let post_id = PostId::new();
        let post = GroupPostActiveModel {
            id: Set(post_id),
            user_id: Set(user_id),
            topic_id: Set(topic_id),
            parent_post_id: Set(None),
            title: Set("First Post".to_string()),
            body: Set("Hello, World!".to_string()),
            created_at: Set("2024-01-01T00:01:00Z".to_string()),
        };
        GroupPost::insert(post).exec(&db).await.unwrap();

        // Query posts for topic
        let posts = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "First Post");
        assert_eq!(posts[0].body, "Hello, World!");
    }

    #[tokio::test]
    async fn test_cascade_delete_topic_deletes_posts() {
        let db = setup_test_db().await;

        // Setup complete chain: profile -> group -> user -> topic -> posts
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("User".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        let user_id = UserId::new();
        let user = GroupUserActiveModel {
            id: Set(user_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user).exec(&db).await.unwrap();

        let topic_id = TopicId::new();
        let topic = GroupTopicActiveModel {
            id: Set(topic_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
            created_at: Set("2024-01-01".to_string()),
        };
        GroupTopic::insert(topic).exec(&db).await.unwrap();

        // Create multiple posts
        for i in 0..3 {
            let post = GroupPostActiveModel {
                id: Set(PostId::new()),
                user_id: Set(user_id),
                topic_id: Set(topic_id),
                parent_post_id: Set(None),
                title: Set(format!("Post {}", i)),
                body: Set(format!("Body {}", i)),
                created_at: Set("2024-01-01".to_string()),
            };
            GroupPost::insert(post).exec(&db).await.unwrap();
        }

        // Verify posts exist
        let posts_before = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(posts_before.len(), 3);

        // Delete topic
        GroupTopic::delete_by_id(topic_id).exec(&db).await.unwrap();

        // Verify posts were cascade deleted
        let posts_after = GroupPost::find()
            .filter(GroupPostColumn::TopicId.eq(topic_id))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(
            posts_after.len(),
            0,
            "Posts should be cascade deleted with topic"
        );
    }

    #[tokio::test]
    async fn test_identity_composite_key() {
        let db = setup_test_db().await;

        // Create profile first
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("User".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        // Create identity with composite key (node_id + profile_id)
        let node_id = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let identity = IdentityActiveModel {
            node_id: Set(node_id.clone()),
            profile_id: Set(profile_id),
        };
        Identity::insert(identity).exec(&db).await.unwrap();

        // Query by profile_id
        let found = Identity::find()
            .filter(IdentityColumn::ProfileId.eq(profile_id))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].node_id, node_id);
        assert_eq!(found[0].profile_id, profile_id);
    }

    #[tokio::test]
    async fn test_identity_with_multiple_profiles() {
        let db = setup_test_db().await;

        // Create ONE identity (node_id)
        let node_id = vec![1, 2, 3, 4, 5, 6, 7, 8];

        // Create MULTIPLE profiles for this ONE identity
        for i in 0..3 {
            let profile_id = ProfileId::new();
            let profile = ProfileActiveModel {
                id: Set(profile_id),
                name: Set(format!("Profile {}", i)),
                desc: Set("Persona".to_string()),
                picture: Set(None),
            };
            Profile::insert(profile).exec(&db).await.unwrap();

            // Link SAME node_id to DIFFERENT profiles
            let identity = IdentityActiveModel {
                node_id: Set(node_id.clone()),
                profile_id: Set(profile_id),
            };
            Identity::insert(identity).exec(&db).await.unwrap();
        }

        // Query: should find 3 profiles for this one identity
        let identities = Identity::find()
            .filter(IdentityColumn::NodeId.eq(node_id))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(identities.len(), 3, "One identity should have 3 profiles");
    }

    #[tokio::test]
    async fn test_profile_cannot_belong_to_multiple_identities() {
        let db = setup_test_db().await;

        // Create ONE profile
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Exclusive Profile".to_string()),
            desc: Set("Belongs to one identity only".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        // Create first identity with this profile
        let node_id_1 = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let identity1 = IdentityActiveModel {
            node_id: Set(node_id_1),
            profile_id: Set(profile_id),
        };
        Identity::insert(identity1).exec(&db).await.unwrap();

        // Try to create SECOND identity with SAME profile (should fail)
        let node_id_2 = vec![9, 10, 11, 12, 13, 14, 15, 16];
        let identity2 = IdentityActiveModel {
            node_id: Set(node_id_2),
            profile_id: Set(profile_id), // Same profile!
        };

        let result = Identity::insert(identity2).exec(&db).await;
        assert!(
            result.is_err(),
            "Should fail: profile cannot belong to multiple identities (UNIQUE constraint)"
        );
    }

    #[tokio::test]
    async fn test_find_group_with_related_admins() {
        let db = setup_test_db().await;

        // Setup: Create profile, group, and multiple admins
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Add 3 admins to the group
        for i in 0..3 {
            let admin_profile_id = ProfileId::new();
            let admin_profile = ProfileActiveModel {
                id: Set(admin_profile_id),
                name: Set(format!("Admin {}", i)),
                desc: Set("Admin".to_string()),
                picture: Set(None),
            };
            Profile::insert(admin_profile).exec(&db).await.unwrap();

            let admin = GroupAdminActiveModel {
                group_id: Set(group_id),
                identity_id: Set(admin_profile_id),
            };
            GroupAdmin::insert(admin).exec(&db).await.unwrap();
        }

        // Test: Load group with related admins using find_with_related
        let groups_with_admins = Group::find()
            .filter(GroupColumn::Id.eq(group_id))
            .find_with_related(GroupAdmin)
            .all(&db)
            .await
            .unwrap();

        // Verify the relationship loaded correctly
        assert_eq!(groups_with_admins.len(), 1);
        let (group, admins) = &groups_with_admins[0];
        assert_eq!(group.id, group_id);
        assert_eq!(admins.len(), 3);
    }

    #[tokio::test]
    async fn test_find_group_with_related_users() {
        let db = setup_test_db().await;

        // Setup
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Add 5 users to the group
        for i in 0..5 {
            let user_profile_id = ProfileId::new();
            let user_profile = ProfileActiveModel {
                id: Set(user_profile_id),
                name: Set(format!("User {}", i)),
                desc: Set("User".to_string()),
                picture: Set(None),
            };
            Profile::insert(user_profile).exec(&db).await.unwrap();

            let user = GroupUserActiveModel {
                id: Set(UserId::new()),
                group_id: Set(group_id),
                profile_id: Set(user_profile_id),
            };
            GroupUser::insert(user).exec(&db).await.unwrap();
        }

        // Test: Load group with related users
        let groups_with_users = Group::find()
            .filter(GroupColumn::Id.eq(group_id))
            .find_with_related(GroupUser)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(groups_with_users.len(), 1);
        let (group, users) = &groups_with_users[0];
        assert_eq!(group.id, group_id);
        assert_eq!(users.len(), 5);
    }

    #[tokio::test]
    async fn test_find_group_with_related_topics() {
        let db = setup_test_db().await;

        // Setup
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Create multiple topics in the group
        for i in 0..4 {
            let topic = GroupTopicActiveModel {
                id: Set(TopicId::new()),
                group_id: Set(group_id),
                profile_id: Set(profile_id),
                created_at: Set(format!("2024-01-{:02}", i + 1)),
            };
            GroupTopic::insert(topic).exec(&db).await.unwrap();
        }

        // Test: Load group with related topics
        let groups_with_topics = Group::find()
            .filter(GroupColumn::Id.eq(group_id))
            .find_with_related(GroupTopic)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(groups_with_topics.len(), 1);
        let (group, topics) = &groups_with_topics[0];
        assert_eq!(group.id, group_id);
        assert_eq!(topics.len(), 4);
    }

    #[tokio::test]
    async fn test_find_topic_with_related_posts() {
        let db = setup_test_db().await;

        // Setup: profile, group, user, and topic
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("User".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        let user_id = UserId::new();
        let user = GroupUserActiveModel {
            id: Set(user_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };
        GroupUser::insert(user).exec(&db).await.unwrap();

        let topic_id = TopicId::new();
        let topic = GroupTopicActiveModel {
            id: Set(topic_id),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
            created_at: Set("2024-01-01".to_string()),
        };
        GroupTopic::insert(topic).exec(&db).await.unwrap();

        // Create multiple posts in the topic
        for i in 0..10 {
            let post = GroupPostActiveModel {
                id: Set(PostId::new()),
                user_id: Set(user_id),
                topic_id: Set(topic_id),
                parent_post_id: Set(None),
                title: Set(format!("Post {}", i)),
                body: Set(format!("Body {}", i)),
                created_at: Set("2024-01-01".to_string()),
            };
            GroupPost::insert(post).exec(&db).await.unwrap();
        }

        // Test: Load topic with related posts
        let topics_with_posts = GroupTopic::find()
            .filter(GroupTopicColumn::Id.eq(topic_id))
            .find_with_related(GroupPost)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(topics_with_posts.len(), 1);
        let (topic, posts) = &topics_with_posts[0];
        assert_eq!(topic.id, topic_id);
        assert_eq!(posts.len(), 10);
    }

    #[tokio::test]
    async fn test_relationship_empty_related_collection() {
        let db = setup_test_db().await;

        // Create a group with NO admins
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set("Owner".to_string()),
            desc: Set("Desc".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&db).await.unwrap();

        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };
        Group::insert(group).exec(&db).await.unwrap();

        // Test: Load group with related admins (should be empty)
        let groups_with_admins = Group::find()
            .filter(GroupColumn::Id.eq(group_id))
            .find_with_related(GroupAdmin)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(groups_with_admins.len(), 1);
        let (group, admins) = &groups_with_admins[0];
        assert_eq!(group.id, group_id);
        assert_eq!(admins.len(), 0, "Group should have no admins");
    }
}
