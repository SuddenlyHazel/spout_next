use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, prelude::*, Any, AnyPool};
use thiserror::Error;

use crate::{error::MigrationError, ids::ProfileId, profile::migrations::create_profiles_table};

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Profile {
    #[sqlx(try_from = "String")]
    pub id: ProfileId,
    pub name: String,
    pub desc: String,
    pub picture: Option<Vec<u8>>,
}

impl Profile {
    pub async fn create<'a, E>(
        name: String,
        desc: String,
        picture: Option<Vec<u8>>,
        conn: E,
    ) -> Result<Profile, ProfileError>
    where
        E: Executor<'a, Database = Any>,
    {
        let id = ProfileId::new();

        sqlx::query(
            r#"
      INSERT INTO profiles (id, name, desc, picture)
      VALUES (?, ?, ?, ?)
      "#,
        )
        .bind(id.to_string())
        .bind(&name)
        .bind(&desc)
        .bind(&picture)
        .execute(conn)
        .await?;

        Ok(Profile {
            id,
            name,
            desc,
            picture,
        })
    }

    pub async fn by_id(
        id: &ProfileId,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Option<Profile>, ProfileError> {
        let row = sqlx::query(
            r#"
      SELECT id, name, desc, picture
      FROM profiles
      WHERE id = ?
      "#,
        )
        .bind(id.to_string())
        .fetch_optional(&mut **conn)
        .await?;

        let profile = match row {
            Some(row) => {
                let id_str: String = row.try_get("id")?;
                let id =
                    ProfileId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                let name: String = row.try_get("name")?;
                let desc: String = row.try_get("desc")?;
                let picture: Option<Vec<u8>> = row.try_get("picture")?;

                Some(Profile {
                    id,
                    name,
                    desc,
                    picture,
                })
            }
            None => None,
        };

        Ok(profile)
    }

    pub async fn by_name(
        name: &str,
        conn: &mut PoolConnection<Any>,
    ) -> Result<Option<Profile>, ProfileError> {
        let row = sqlx::query(
            r#"
      SELECT id, name, desc, picture
      FROM profiles
      WHERE name = ?
      "#,
        )
        .bind(name)
        .fetch_optional(&mut **conn)
        .await?;

        let profile = match row {
            Some(row) => {
                let id_str: String = row.try_get("id")?;
                let id =
                    ProfileId::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                let name: String = row.try_get("name")?;
                let desc: String = row.try_get("desc")?;
                let picture: Option<Vec<u8>> = row.try_get("picture")?;

                Some(Profile {
                    id,
                    name,
                    desc,
                    picture,
                })
            }
            None => None,
        };

        Ok(profile)
    }
}

pub async fn migrate_up(conn: AnyPool) -> Result<(), MigrationError> {
    let mut conn = conn.acquire().await?;
    create_profiles_table(&mut conn).await?;

    Ok(())
}

mod migrations {
    use sqlx::{pool::PoolConnection, Any};

    use crate::error::MigrationError;

    pub async fn create_profiles_table(
        conn: &mut PoolConnection<Any>,
    ) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS profiles (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        desc TEXT NOT NULL,
        picture BLOB
      )
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE UNIQUE INDEX IF NOT EXISTS idx_profiles_id ON profiles(id)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        sqlx::query(
            r#"
      CREATE UNIQUE INDEX IF NOT EXISTS idx_profiles_name ON profiles(name)
      "#,
        )
        .execute(&mut **conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::*;

    #[tokio::test]
    async fn migrates_creates_and_gets_profile() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;

        // Create a test profile
        let mut conn = pool.acquire().await.unwrap();
        let test_name = "Test User".to_string();
        let test_desc = "A test user description".to_string();
        let test_picture = Some(vec![1, 2, 3, 4, 5]);

        let created_profile = Profile::create(
            test_name.clone(),
            test_desc.clone(),
            test_picture.clone(),
            &mut *conn,
        )
        .await
        .unwrap();

        // Verify created profile has correct data
        assert_eq!(created_profile.name, test_name);
        assert_eq!(created_profile.desc, test_desc);
        assert_eq!(created_profile.picture, test_picture);

        // Retrieve the profile by ID
        let retrieved_profile = Profile::by_id(&created_profile.id, &mut conn)
            .await
            .unwrap();

        // Verify we got the profile back
        assert!(retrieved_profile.is_some());
        let retrieved_profile = retrieved_profile.unwrap();

        // Verify all fields match
        assert_eq!(retrieved_profile.id, created_profile.id);
        assert_eq!(retrieved_profile.name, test_name);
        assert_eq!(retrieved_profile.desc, test_desc);
        assert_eq!(retrieved_profile.picture, test_picture);

        // Test retrieving a non-existent profile
        let non_existent_id = ProfileId::new();
        let non_existent_profile = Profile::by_id(&non_existent_id, &mut conn).await.unwrap();
        assert!(non_existent_profile.is_none());
    }

    #[tokio::test]
    async fn test_unique_name_constraint() {
        test_utils::init_test_drivers();
        let pool = test_utils::create_test_db_with_migrations().await;
        let mut conn = pool.acquire().await.unwrap();

        // Create a profile with a specific name
        let name = "UniqueUser".to_string();
        let desc = "First profile".to_string();

        let first_profile = Profile::create(name.clone(), desc.clone(), None, &mut *conn)
            .await
            .unwrap();

        assert_eq!(first_profile.name, name);

        // Verify we can retrieve the profile by name
        let retrieved_by_name = Profile::by_name(&name, &mut conn).await.unwrap();

        assert!(retrieved_by_name.is_some());
        let retrieved_by_name = retrieved_by_name.unwrap();
        assert_eq!(retrieved_by_name.id, first_profile.id);
        assert_eq!(retrieved_by_name.name, name);
        assert_eq!(retrieved_by_name.desc, desc);

        // Try to create another profile with the same name
        let result =
            Profile::create(name.clone(), "Second profile".to_string(), None, &mut *conn).await;

        // This should fail due to unique constraint on name
        assert!(result.is_err());

        // Verify it's a database error
        match result {
            Err(ProfileError::DatabaseError(_)) => {
                // Expected error type
            }
            _ => panic!("Expected DatabaseError for duplicate name"),
        }

        // Verify we still only have one profile with that name
        let still_first = Profile::by_name(&name, &mut conn).await.unwrap().unwrap();
        assert_eq!(still_first.id, first_profile.id);
        assert_eq!(still_first.desc, desc); // Should still be the first profile's description

        // Verify by_name returns None for non-existent names
        let non_existent = Profile::by_name("NonExistentUser", &mut conn)
            .await
            .unwrap();
        assert!(non_existent.is_none());
    }
}
