use iroh::PublicKey;
use sea_orm::{DatabaseConnection, TransactionTrait};
use thiserror::Error;
use zel_core::prelude::*;

use crate::{entity::prelude::*, ids::ProfileId};

#[derive(Debug, Error)]
pub enum ProfilesServiceError {
    #[error("fatal database error")]
    DbError(#[from] DbErr),
}

// TODO : need to actually dig into each error type
// and correctly flag
impl From<ProfilesServiceError> for ResourceError {
    fn from(error: ProfilesServiceError) -> Self {
        match error {
            ProfilesServiceError::DbError(error) => ResourceError::infra(error),
        }
    }
}

#[derive(Clone)]
pub struct ProfilesService {
    db: DatabaseConnection,
}

impl ProfilesService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn _create_profile(
        &self,
        node_id: PublicKey,
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
    ) -> Result<ProfileModel, ProfilesServiceError> {
        let txn = self.db.begin().await?;

        // Create profile
        let profile_id = ProfileId::new();
        let profile = ProfileActiveModel {
            id: Set(profile_id),
            name: Set(name),
            desc: Set(desc),
            picture: Set(picture),
        };

        let profile_result = Profile::insert(profile).exec_with_returning(&txn).await?;

        // Create identity linking node_id to profile
        let node_id_bytes = node_id.as_bytes().to_vec();
        let identity = IdentityActiveModel {
            node_id: Set(node_id_bytes),
            profile_id: Set(profile_id),
        };

        Identity::insert(identity).exec(&txn).await?;

        txn.commit().await?;
        Ok(profile_result)
    }

    pub async fn _list_profiles(
        &self,
        node_id: PublicKey,
    ) -> Result<Vec<ProfileModel>, ProfilesServiceError> {
        let node_id_bytes = node_id.as_bytes().to_vec();

        // Use find_with_related to eager-load profiles with their identities
        // This generates an optimized JOIN query instead of multiple queries
        let identities_with_profiles = Identity::find()
            .filter(IdentityColumn::NodeId.eq(node_id_bytes))
            .find_with_related(Profile)
            .all(&self.db)
            .await?;

        // Extract profiles from the relationship results
        // Each tuple is (IdentityModel, Vec<ProfileModel>)
        let profiles: Vec<ProfileModel> = identities_with_profiles
            .into_iter()
            .flat_map(|(_, profiles)| profiles)
            .collect();

        Ok(profiles)
    }
}

#[zel_service(name = "profile")]
trait Profiles {
    #[doc = "Create a profile given current identity of the calling peer"]
    #[method(name = "create_profile")]
    async fn create_profile(
        &self,
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
    ) -> Result<ProfileModel, ResourceError>;

    #[doc = "List all profiles associated with the identity of the calling peer"]
    #[method(name = "list_profiles")]
    async fn list_profiles(&self) -> Result<Vec<ProfileModel>, ResourceError>;
}

#[async_trait]
impl ProfilesServer for ProfilesService {
    async fn create_profile(
        &self,
        ctx: RequestContext,
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
    ) -> Result<ProfileModel, ResourceError> {
        Ok(self
            ._create_profile(ctx.remote_id(), name, desc, picture)
            .await?)
    }

    async fn list_profiles(&self, ctx: RequestContext) -> Result<Vec<ProfileModel>, ResourceError> {
        Ok(self._list_profiles(ctx.remote_id()).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::migrator::Migrator;
    use iroh::SecretKey;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_service() -> ProfilesService {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        ProfilesService::new(db)
    }

    fn test_node_id() -> PublicKey {
        let secret_key = SecretKey::generate(&mut rand::thread_rng());
        secret_key.public()
    }

    #[tokio::test]
    async fn test_create_profile() {
        let service = setup_test_service().await;
        let node_id = test_node_id();

        let profile = service
            ._create_profile(
                node_id,
                "Test User".to_string(),
                "Test Description".to_string(),
                None,
            )
            .await
            .expect("Failed to create profile");

        assert_eq!(profile.name, "Test User");
        assert_eq!(profile.desc, "Test Description");
        assert_eq!(profile.picture, None);
    }

    #[tokio::test]
    async fn test_create_profile_with_picture() {
        let service = setup_test_service().await;
        let node_id = test_node_id();
        let picture_data = vec![1, 2, 3, 4, 5];

        let profile = service
            ._create_profile(
                node_id,
                "User with Picture".to_string(),
                "Has avatar".to_string(),
                Some(picture_data.clone()),
            )
            .await
            .expect("Failed to create profile");

        assert_eq!(profile.name, "User with Picture");
        assert_eq!(profile.picture, Some(picture_data));
    }

    #[tokio::test]
    async fn test_list_profiles_empty() {
        let service = setup_test_service().await;
        let node_id = test_node_id();

        let profiles = service
            ._list_profiles(node_id)
            .await
            .expect("Failed to list profiles");

        assert_eq!(profiles.len(), 0, "New identity should have no profiles");
    }

    #[tokio::test]
    async fn test_list_profiles_single() {
        let service = setup_test_service().await;
        let node_id = test_node_id();

        // Create a profile
        let created = service
            ._create_profile(
                node_id,
                "Profile 1".to_string(),
                "First profile".to_string(),
                None,
            )
            .await
            .expect("Failed to create profile");

        // List profiles
        let profiles = service
            ._list_profiles(node_id)
            .await
            .expect("Failed to list profiles");

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, created.id);
        assert_eq!(profiles[0].name, "Profile 1");
    }

    #[tokio::test]
    async fn test_list_profiles_multiple() {
        let service = setup_test_service().await;
        let node_id = test_node_id();

        // Create multiple profiles for the same identity
        for i in 0..3 {
            service
                ._create_profile(
                    node_id,
                    format!("Profile {}", i),
                    format!("Description {}", i),
                    None,
                )
                .await
                .expect("Failed to create profile");
        }

        // List all profiles
        let profiles = service
            ._list_profiles(node_id)
            .await
            .expect("Failed to list profiles");

        assert_eq!(
            profiles.len(),
            3,
            "Should have 3 profiles for this identity"
        );

        // Verify all profiles are present
        let names: Vec<String> = profiles.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"Profile 0".to_string()));
        assert!(names.contains(&"Profile 1".to_string()));
        assert!(names.contains(&"Profile 2".to_string()));
    }

    #[tokio::test]
    async fn test_list_profiles_isolated_by_identity() {
        let service = setup_test_service().await;
        let node_id_1 = test_node_id();
        let node_id_2 = test_node_id();

        // Create profiles for first identity
        service
            ._create_profile(
                node_id_1,
                "Identity 1 Profile".to_string(),
                "Desc".to_string(),
                None,
            )
            .await
            .expect("Failed to create profile");

        // Create profiles for second identity
        service
            ._create_profile(
                node_id_2,
                "Identity 2 Profile".to_string(),
                "Desc".to_string(),
                None,
            )
            .await
            .expect("Failed to create profile");

        // List profiles for first identity
        let profiles_1 = service
            ._list_profiles(node_id_1)
            .await
            .expect("Failed to list profiles");

        // List profiles for second identity
        let profiles_2 = service
            ._list_profiles(node_id_2)
            .await
            .expect("Failed to list profiles");

        assert_eq!(profiles_1.len(), 1, "Identity 1 should have 1 profile");
        assert_eq!(profiles_2.len(), 1, "Identity 2 should have 1 profile");
        assert_eq!(profiles_1[0].name, "Identity 1 Profile");
        assert_eq!(profiles_2[0].name, "Identity 2 Profile");
    }

    #[tokio::test]
    async fn test_create_profile_creates_identity_link() {
        let service = setup_test_service().await;
        let node_id = test_node_id();

        let profile = service
            ._create_profile(node_id, "Test".to_string(), "Test".to_string(), None)
            .await
            .expect("Failed to create profile");

        // Verify the identity link was created by checking we can list the profile
        let profiles = service
            ._list_profiles(node_id)
            .await
            .expect("Failed to list profiles");

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, profile.id);
    }

    #[tokio::test]
    async fn test_profile_unique_constraint_enforced() {
        let service = setup_test_service().await;
        let node_id_1 = test_node_id();
        let node_id_2 = test_node_id();

        // Create profile with first identity
        let profile = service
            ._create_profile(
                node_id_1,
                "Shared Profile Attempt".to_string(),
                "Desc".to_string(),
                None,
            )
            .await
            .expect("Failed to create profile");

        // Try to manually create an identity link for same profile with different node_id
        // This should fail due to UNIQUE constraint on profile_id
        let node_id_2_bytes = node_id_2.as_bytes().to_vec();
        let identity = IdentityActiveModel {
            node_id: Set(node_id_2_bytes),
            profile_id: Set(profile.id),
        };

        let result = Identity::insert(identity).exec(&service.db).await;

        assert!(
            result.is_err(),
            "Should fail: profile cannot belong to multiple identities"
        );
    }
}
