use iroh::PublicKey;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any, AnyPool};
use thiserror::Error;
use uuid::Uuid;

use crate::{error::MigrationError, identity::migrations::create_identities_table};

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid public key")]
    InvalidPublicKey(#[from] iroh::KeyParsingError),
    #[error("invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Identity {
    pub node_id: PublicKey,
    pub profile_id: Uuid,
}

impl Identity {
    pub async fn create<'a, E>(
        node_id: PublicKey,
        profile_id: Uuid,
        conn: E,
    ) -> Result<Identity, IdentityError>
    where
        E: Executor<'a, Database = Any>,
    {
        sqlx::query(
            r#"
      INSERT INTO identities (node_id, profile_id)
      VALUES (?, ?)
      "#,
        )
        .bind(node_id.as_bytes().to_vec())
        .bind(profile_id.to_string())
        .execute(conn)
        .await?;

        Ok(Identity {
            node_id,
            profile_id,
        })
    }

    pub async fn list_for_node_id(
        node_id: &PublicKey,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Vec<Identity>, IdentityError> {
        let rows = sqlx::query(
            r#"
      SELECT node_id, profile_id
      FROM identities
      WHERE node_id = ?
      "#,
        )
        .bind(node_id.as_bytes().to_vec())
        .fetch_all(&mut **conn)
        .await?;

        let mut identities = Vec::new();
        for row in rows {
            let node_id_bytes: Vec<u8> = row.try_get("node_id")?;
            if node_id_bytes.len() != 32 {
                return Err(sqlx::Error::Decode(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid node_id length",
                )))
                .into());
            }
            let node_id_arr: [u8; 32] = node_id_bytes.try_into().map_err(|_| {
                sqlx::Error::Decode(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid node_id",
                )))
            })?;
            let node_id = PublicKey::from_bytes(&node_id_arr)?;

            let profile_id_str: String = row.try_get("profile_id")?;
            let profile_id = Uuid::parse_str(&profile_id_str)?;

            identities.push(Identity {
                node_id,
                profile_id,
            });
        }

        Ok(identities)
    }
}

pub async fn migrate_up(conn: AnyPool) -> Result<(), MigrationError> {
    let mut conn = conn.acquire().await?;
    create_identities_table(&mut conn).await?;

    Ok(())
}

mod migrations {
    use sqlx::{pool::PoolConnection, Any};

    use crate::error::MigrationError;

    pub async fn create_identities_table(
        conn: &mut PoolConnection<Any>,
    ) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS identities (
        node_id BLOB NOT NULL,
        profile_id TEXT NOT NULL,
        PRIMARY KEY (node_id, profile_id)
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use iroh::SecretKey;

    use super::*;
    use crate::test_utils;

    #[tokio::test]
    async fn creates_and_lists_identities() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        let node_id = SecretKey::generate(&mut rand::rng());
        let node_id = node_id.public();
        let profile_id1 = Uuid::now_v7();
        let profile_id2 = Uuid::now_v7();

        // Create first identity
        let first_identity = Identity::create(node_id, profile_id1, &mut *conn)
            .await
            .unwrap();

        assert_eq!(first_identity.node_id, node_id);
        assert_eq!(first_identity.profile_id, profile_id1);

        // Create second identity for same node
        let second_identity = Identity::create(node_id, profile_id2, &mut *conn)
            .await
            .unwrap();

        assert_eq!(second_identity.node_id, node_id);
        assert_eq!(second_identity.profile_id, profile_id2);

        // List identities for the node
        let identities = Identity::list_for_node_id(&node_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(identities.len(), 2);
        assert!(identities
            .iter()
            .any(|i| i.profile_id == first_identity.profile_id));
        assert!(identities
            .iter()
            .any(|i| i.profile_id == second_identity.profile_id));
        assert!(identities.iter().all(|i| i.node_id == node_id));

        // Try to create duplicate (same node_id + profile_id)
        let result = Identity::create(node_id, profile_id1, &mut *conn).await;

        assert!(result.is_err());
        match result {
            Err(IdentityError::DatabaseError(_)) => {
                // Expected constraint violation
            }
            _ => panic!("Expected DatabaseError for duplicate identity"),
        }

        // Should still have only 2 identities
        let identities_after_duplicate_attempt = Identity::list_for_node_id(&node_id, &mut conn)
            .await
            .unwrap();
        assert_eq!(identities_after_duplicate_attempt.len(), 2);

        // Test list for different node returns empty
        let different_node = SecretKey::generate(&mut rand::rng());
        let different_node = different_node.public();
        let different_identities = Identity::list_for_node_id(&different_node, &mut conn)
            .await
            .unwrap();
        assert!(different_identities.is_empty());
    }
}
