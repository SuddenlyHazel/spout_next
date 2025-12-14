use sea_orm::{DatabaseConnection, TransactionTrait};
use thiserror::Error;
use zel_core::prelude::*;

use crate::{
    entity::prelude::*,
    ids::{GroupId, ProfileId},
};

#[derive(Debug, Error)]
pub enum GroupsServiceError {
    #[error("fatal database error")]
    DbError(#[from] DbErr),

    #[error("group not found")]
    GroupNotFound,

    #[error("profile not found")]
    ProfileNotFound,

    #[error("unauthorized: not a group admin")]
    Unauthorized,
}

impl From<GroupsServiceError> for ResourceError {
    fn from(error: GroupsServiceError) -> Self {
        match error {
            GroupsServiceError::DbError(error) => ResourceError::infra(error),
            GroupsServiceError::GroupNotFound => ResourceError::app(error),
            GroupsServiceError::ProfileNotFound => ResourceError::app(error),
            GroupsServiceError::Unauthorized => ResourceError::app(error),
        }
    }
}

#[derive(Clone)]
pub struct GroupsService {
    db: DatabaseConnection,
}

impl GroupsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new group owned by the specified profile
    pub async fn _create_group(
        &self,
        profile_id: ProfileId,
    ) -> Result<GroupModel, GroupsServiceError> {
        // Verify profile exists
        let profile_exists = Profile::find_by_id(profile_id)
            .one(&self.db)
            .await?
            .is_some();

        if !profile_exists {
            return Err(GroupsServiceError::ProfileNotFound);
        }

        let txn = self.db.begin().await?;

        // Create group
        let group_id = GroupId::new();
        let group = GroupActiveModel {
            id: Set(group_id),
            profile_id: Set(profile_id),
        };

        let group_result = Group::insert(group).exec_with_returning(&txn).await?;

        // Make the creator an admin
        let admin = GroupAdminActiveModel {
            group_id: Set(group_id),
            identity_id: Set(profile_id),
        };
        GroupAdmin::insert(admin).exec(&txn).await?;

        txn.commit().await?;
        Ok(group_result)
    }

    /// List all groups owned by a profile
    pub async fn _list_groups(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<GroupModel>, GroupsServiceError> {
        let groups = Group::find()
            .filter(GroupColumn::ProfileId.eq(profile_id))
            .all(&self.db)
            .await?;

        Ok(groups)
    }

    /// Get a specific group by ID
    pub async fn _get_group(&self, group_id: GroupId) -> Result<GroupModel, GroupsServiceError> {
        Group::find_by_id(group_id)
            .one(&self.db)
            .await?
            .ok_or(GroupsServiceError::GroupNotFound)
    }

    /// Delete a group (only by owner or admin)
    pub async fn _delete_group(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<(), GroupsServiceError> {
        // Check if user is an admin
        let is_admin = self._is_admin(group_id, profile_id).await?;
        if !is_admin {
            return Err(GroupsServiceError::Unauthorized);
        }

        // Delete will cascade to all related records due to FK constraints
        Group::delete_by_id(group_id).exec(&self.db).await?;

        Ok(())
    }

    /// Check if a profile is an admin of a group
    pub async fn _is_admin(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<bool, GroupsServiceError> {
        let admin = GroupAdmin::find()
            .filter(GroupAdminColumn::GroupId.eq(group_id))
            .filter(GroupAdminColumn::IdentityId.eq(profile_id))
            .one(&self.db)
            .await?;

        Ok(admin.is_some())
    }

    /// List all admins for a group
    pub async fn _list_admins(
        &self,
        group_id: GroupId,
    ) -> Result<Vec<GroupAdminModel>, GroupsServiceError> {
        let admins = GroupAdmin::find()
            .filter(GroupAdminColumn::GroupId.eq(group_id))
            .all(&self.db)
            .await?;

        Ok(admins)
    }

    /// Add a user to a group
    pub async fn _add_user(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<GroupUserModel, GroupsServiceError> {
        // Verify group exists
        self._get_group(group_id).await?;

        // Verify profile exists
        let profile_exists = Profile::find_by_id(profile_id)
            .one(&self.db)
            .await?
            .is_some();

        if !profile_exists {
            return Err(GroupsServiceError::ProfileNotFound);
        }

        // Add user
        let user = GroupUserActiveModel {
            id: Set(crate::ids::UserId::new()),
            group_id: Set(group_id),
            profile_id: Set(profile_id),
        };

        let result = GroupUser::insert(user)
            .exec_with_returning(&self.db)
            .await?;

        Ok(result)
    }

    /// List all users in a group
    pub async fn _list_users(
        &self,
        group_id: GroupId,
    ) -> Result<Vec<GroupUserModel>, GroupsServiceError> {
        let users = GroupUser::find()
            .filter(GroupUserColumn::GroupId.eq(group_id))
            .all(&self.db)
            .await?;

        Ok(users)
    }
}

#[zel_service(name = "groups")]
trait Groups {
    #[doc = "Create a new group owned by the calling profile"]
    #[method(name = "create_group")]
    async fn create_group(&self, profile_id: ProfileId) -> Result<GroupModel, ResourceError>;

    #[doc = "List all groups owned by a profile"]
    #[method(name = "list_groups")]
    async fn list_groups(&self, profile_id: ProfileId) -> Result<Vec<GroupModel>, ResourceError>;

    #[doc = "Get a specific group by ID"]
    #[method(name = "get_group")]
    async fn get_group(&self, group_id: GroupId) -> Result<GroupModel, ResourceError>;

    #[doc = "Delete a group"]
    #[method(name = "delete_group")]
    async fn delete_group(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<(), ResourceError>;

    #[doc = "Check if a profile is an admin of a group"]
    #[method(name = "is_admin")]
    async fn is_admin(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<bool, ResourceError>;

    #[doc = "List all admins for a group"]
    #[method(name = "list_admins")]
    async fn list_admins(&self, group_id: GroupId) -> Result<Vec<GroupAdminModel>, ResourceError>;

    #[doc = "Add a user to a group"]
    #[method(name = "add_user")]
    async fn add_user(
        &self,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<GroupUserModel, ResourceError>;

    #[doc = "List all users in a group"]
    #[method(name = "list_users")]
    async fn list_users(&self, group_id: GroupId) -> Result<Vec<GroupUserModel>, ResourceError>;
}

#[async_trait]
impl GroupsServer for GroupsService {
    async fn create_group(
        &self,
        _ctx: RequestContext,
        profile_id: ProfileId,
    ) -> Result<GroupModel, ResourceError> {
        Ok(self._create_group(profile_id).await?)
    }

    async fn list_groups(
        &self,
        _ctx: RequestContext,
        profile_id: ProfileId,
    ) -> Result<Vec<GroupModel>, ResourceError> {
        Ok(self._list_groups(profile_id).await?)
    }

    async fn get_group(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
    ) -> Result<GroupModel, ResourceError> {
        Ok(self._get_group(group_id).await?)
    }

    async fn delete_group(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<(), ResourceError> {
        Ok(self._delete_group(group_id, profile_id).await?)
    }

    async fn is_admin(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<bool, ResourceError> {
        Ok(self._is_admin(group_id, profile_id).await?)
    }

    async fn list_admins(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
    ) -> Result<Vec<GroupAdminModel>, ResourceError> {
        Ok(self._list_admins(group_id).await?)
    }

    async fn add_user(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
        profile_id: ProfileId,
    ) -> Result<GroupUserModel, ResourceError> {
        Ok(self._add_user(group_id, profile_id).await?)
    }

    async fn list_users(
        &self,
        _ctx: RequestContext,
        group_id: GroupId,
    ) -> Result<Vec<GroupUserModel>, ResourceError> {
        Ok(self._list_users(group_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::migrator::Migrator;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_service() -> GroupsService {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        GroupsService::new(db)
    }

    async fn create_test_profile(service: &GroupsService) -> ProfileId {
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set(format!("Test User {}", profile_id)), // Unique name
            desc: Set("Test".to_string()),
            picture: Set(None),
        };
        Profile::insert(profile).exec(&service.db).await.unwrap();
        profile_id
    }

    #[tokio::test]
    async fn test_create_group() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        let group = service
            ._create_group(profile_id)
            .await
            .expect("Failed to create group");

        assert_eq!(group.profile_id, profile_id);
    }

    #[tokio::test]
    async fn test_create_group_makes_creator_admin() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        let group = service._create_group(profile_id).await.unwrap();

        let is_admin = service._is_admin(group.id, profile_id).await.unwrap();
        assert!(is_admin, "Creator should be an admin");
    }

    #[tokio::test]
    async fn test_list_groups() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        // Create multiple groups
        for _ in 0..3 {
            service._create_group(profile_id).await.unwrap();
        }

        let groups = service._list_groups(profile_id).await.unwrap();
        assert_eq!(groups.len(), 3, "Should have 3 groups");
    }

    #[tokio::test]
    async fn test_get_group() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        let created = service._create_group(profile_id).await.unwrap();
        let fetched = service._get_group(created.id).await.unwrap();

        assert_eq!(created.id, fetched.id);
    }

    #[tokio::test]
    async fn test_delete_group_by_admin() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        let group = service._create_group(profile_id).await.unwrap();

        service
            ._delete_group(group.id, profile_id)
            .await
            .expect("Admin should be able to delete group");

        let result = service._get_group(group.id).await;
        assert!(result.is_err(), "Group should be deleted");
    }

    #[tokio::test]
    async fn test_delete_group_by_non_admin_fails() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;
        let other_profile_id = create_test_profile(&service).await;

        let group = service._create_group(profile_id).await.unwrap();

        let result = service._delete_group(group.id, other_profile_id).await;
        assert!(result.is_err(), "Non-admin should not be able to delete");
    }

    #[tokio::test]
    async fn test_add_user_to_group() {
        let service = setup_test_service().await;
        let admin_profile = create_test_profile(&service).await;
        let user_profile = create_test_profile(&service).await;

        let group = service._create_group(admin_profile).await.unwrap();

        let user = service
            ._add_user(group.id, user_profile)
            .await
            .expect("Should add user to group");

        assert_eq!(user.group_id, group.id);
        assert_eq!(user.profile_id, user_profile);
    }

    #[tokio::test]
    async fn test_list_users() {
        let service = setup_test_service().await;
        let admin_profile = create_test_profile(&service).await;

        let group = service._create_group(admin_profile).await.unwrap();

        // Add multiple users
        for _ in 0..3 {
            let user_profile = create_test_profile(&service).await;
            service._add_user(group.id, user_profile).await.unwrap();
        }

        let users = service._list_users(group.id).await.unwrap();
        assert_eq!(users.len(), 3, "Should have 3 users");
    }

    #[tokio::test]
    async fn test_list_admins() {
        let service = setup_test_service().await;
        let profile_id = create_test_profile(&service).await;

        let group = service._create_group(profile_id).await.unwrap();

        let admins = service._list_admins(group.id).await.unwrap();
        assert_eq!(admins.len(), 1, "Should have 1 admin (creator)");
        assert_eq!(admins[0].identity_id, profile_id);
    }

    #[tokio::test]
    async fn test_cascade_delete_removes_users() {
        let service = setup_test_service().await;
        let admin_profile = create_test_profile(&service).await;
        let user_profile = create_test_profile(&service).await;

        let group = service._create_group(admin_profile).await.unwrap();
        service._add_user(group.id, user_profile).await.unwrap();

        // Delete group
        service
            ._delete_group(group.id, admin_profile)
            .await
            .unwrap();

        // Verify users were cascade deleted
        let users = service._list_users(group.id).await.unwrap();
        assert_eq!(users.len(), 0, "Users should be cascade deleted");
    }
}
