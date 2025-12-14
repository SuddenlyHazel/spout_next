use thiserror::Error;
use zel_core::prelude::*;

#[derive(Debug, Error)]
pub enum IdentitiesServiceError {
    #[error("fatal database error")]
    DbError(#[from] sqlx::Error),
    // #[error(transparent)]
    // Application(#[from] IdentityError),
}

// TODO : need to actually dig into each error type
// and correctly flag
impl From<IdentitiesServiceError> for ResourceError {
    fn from(error: IdentitiesServiceError) -> Self {
        match error {
            IdentitiesServiceError::DbError(error) => ResourceError::infra(error),
            // IdentitiesServiceError::Application(identity_error) => {
            //     ResourceError::app(identity_error)
            // }
        }
    }
}

// Everything here a the moment can live in ProfilesService

// #[derive(Clone, Debug)]
// pub struct IdentitiesService {
//     pool: Pool<Any>,
// }

// impl IdentitiesService {
//     pub async fn create(
//         &self,
//         node_id: PublicKey,
//         profile: &Profile,
//     ) -> Result<Identity, IdentitiesServiceError> {
//         let mut conn = self.pool.acquire().await?;
//         let identity = Identity::create(node_id, profile.id.to_owned(), &mut *conn).await?;
//         Ok(identity)
//     }

//     pub async fn _list_profiles(
//         &self,
//         node_id: PublicKey,
//     ) -> Result<Vec<Profile>, IdentitiesServiceError> {
//         let mut conn = self.pool.acquire().await?;

//         let identities = Identity::list_for_node_id(&node_id, &mut conn).await?;

//         let mut profiles = vec![];

//         // really inefficent but whatever for now
//         for identity in identities {
//             if let Ok(Some(profile)) = Profile::by_id(&identity.profile_id, &mut conn).await {
//                 profiles.push(profile);
//             }
//         }

//         Ok(profiles)
//     }
// }

// #[zel_service(name = "identity")]
// trait Identities {
//     #[method(name = "list_profiles")]
//     async fn list_profiles(&self) -> Result<Vec<Profile>, ResourceError>;
// }

// #[async_trait]
// impl IdentitiesServer for IdentitiesService {
//     async fn list_profiles(&self, ctx: RequestContext) -> Result<Vec<Profile>, ResourceError> {
//         let remote_id = ctx.connection().remote_id();
//         Ok(self._list_profiles(remote_id).await?)
//     }
// }
