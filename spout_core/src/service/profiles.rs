use iroh::PublicKey;
use sqlx::{Any, Connection, Pool};
use thiserror::Error;
use zel_core::prelude::*;

use crate::{
    identity::{Identity, IdentityError},
    profile::{self, Profile},
};

#[derive(Debug, Error)]
pub enum ProfilesServiceError {
    #[error("fatal database error")]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    ApplicationProfile(#[from] profile::ProfileError),
    #[error(transparent)]
    ApplicationIdentity(#[from] IdentityError),
}

// TODO : need to actually dig into each error type
// and correctly flag
impl From<ProfilesServiceError> for ResourceError {
    fn from(error: ProfilesServiceError) -> Self {
        match error {
            ProfilesServiceError::DbError(error) => ResourceError::infra(error),
            ProfilesServiceError::ApplicationProfile(profile_error) => {
                ResourceError::app(profile_error)
            }
            ProfilesServiceError::ApplicationIdentity(identity_error) => {
                ResourceError::app(identity_error)
            }
        }
    }
}

#[derive(Clone)]
pub struct ProfilesService {
    pool: Pool<Any>,
}

impl ProfilesService {
    pub async fn _create_profile(
        &self,
        node_id: PublicKey,
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
    ) -> Result<Profile, ProfilesServiceError> {
        let mut conn = self.pool.acquire().await?;

        let mut txn = conn.begin().await?;

        let profile = {
            let profile = Profile::create(name, desc, picture, &mut *txn).await?;
            let _ = Identity::create(node_id, profile.id.clone(), &mut *txn).await?;
            profile
        };

        txn.commit().await?;
        Ok(profile)
    }

    pub async fn _list_profiles(
        &self,
        node_id: PublicKey,
    ) -> Result<Vec<Profile>, ProfilesServiceError> {
        let mut conn = self.pool.acquire().await?;
        let mut profiles = vec![];
        let identites = Identity::list_for_node_id(&node_id, &mut conn).await?;

        for identity in identites {
            if let Some(profile) = Profile::by_id(&identity.profile_id, &mut conn).await? {
                profiles.push(profile);
            }
        }
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
    ) -> Result<Profile, ResourceError>;

    #[doc = "List all profiles associated with the identity of the calling peer"]
    #[method(name = "list_profiles")]
    async fn list_profiles(&self) -> Result<Vec<Profile>, ResourceError>;
}

#[async_trait]
impl ProfilesServer for ProfilesService {
    async fn create_profile(
        &self,
        ctx: RequestContext,
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
    ) -> Result<Profile, ResourceError> {
        Ok(self
            ._create_profile(ctx.remote_id(), name, desc, picture)
            .await?)
    }

    async fn list_profiles(&self, ctx: RequestContext) -> Result<Vec<Profile>, ResourceError> {
        Ok(self._list_profiles(ctx.remote_id()).await?)
    }
}
