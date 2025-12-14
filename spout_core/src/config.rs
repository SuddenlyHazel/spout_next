use std::path::PathBuf;

use iroh::SecretKey;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static DATA_DIR_NAME: &str = "spout_next";
static SPOUT_DB_NAME: &str = "spout_db.sqlite";
static CONFIG_FILE_NAME: &str = "config.json";

// For now this directory structure should be like
// data_dir_path
// |- spout_next
//    |- spout_db.sqlite
//    |- config.json

fn default_secret_key() -> SecretKey {
    // TODO (critical) - Gotta check with Iroh team about best way to actually
    // generate the secret key
    SecretKey::generate(&mut rand::rng())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpoutConfig {
    /// Secret key for the local node/instance.
    #[serde(default = "default_secret_key")]
    pub(crate) secret_key: SecretKey,

    /// Secret key used for client-side identity/auth (separate from node secret).
    ///
    /// `serde(default)` keeps backward compatibility with old config.json files.
    #[serde(default = "default_secret_key")]
    pub(crate) client_secret_key: SecretKey,

    pub(crate) database_path: PathBuf,
}

impl SpoutConfig {
    /// Creates a new SpoutConfig with generated secret keys and the specified data directory
    fn new(data_dir: PathBuf) -> Self {
        let secret_key = default_secret_key();
        let client_secret_key = default_secret_key();
        let database_path = data_dir.join(SPOUT_DB_NAME);

        SpoutConfig {
            secret_key,
            client_secret_key,
            database_path,
        }
    }
}

/// Gets the existing config or initializes a new one if it doesn't exist
pub async fn get_or_init() -> Result<SpoutConfig, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().expect("failed to find a data directory on this platform");

    let spout_dir = data_dir.join(DATA_DIR_NAME);
    let config_path = spout_dir.join(CONFIG_FILE_NAME);

    // Create the spout directory if it doesn't exist
    fs::create_dir_all(&spout_dir).await?;

    // Check if config file exists
    if config_path.exists() {
        // Read and deserialize existing config
        let mut file = fs::File::open(&config_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config: SpoutConfig = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        // Create new config
        let config = SpoutConfig::new(spout_dir.clone());

        // Serialize and write to file
        let json = serde_json::to_string_pretty(&config)?;
        let mut file = fs::File::create(&config_path).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(config)
    }
}
