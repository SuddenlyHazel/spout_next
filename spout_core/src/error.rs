use thiserror::Error;


#[derive(Debug, Error)]
pub enum MigrationError {
  #[error("data store disconnected")]
  AquireError(#[from] sqlx::Error)
}