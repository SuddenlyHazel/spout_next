pub mod entity;
pub mod ids;
pub mod models;
use tokio::sync::OnceCell;

use std::{sync::Arc, time::Duration};

use iroh::Endpoint;
use zel_core::{prelude::RpcServerBuilder, protocol::RpcClient, IrohBundle};

use crate::service::profiles::{ProfilesClient, ProfilesServer, ProfilesService};

pub mod service;

pub mod error;

pub mod config;

static SPOUT_CORE: OnceCell<Arc<SpoutCore>> = OnceCell::const_new();
static ALPN: &[u8] = b"spout::0.1.0";

pub async fn core() -> Arc<SpoutCore> {
    SPOUT_CORE
        .get_or_init(|| async move { Arc::new(SpoutCore::start().await.expect("failed to init")) })
        .await
        .clone()
}

/// Main runtime handle for Spout.
pub struct SpoutCore {
    pub config: config::SpoutConfig,

    /// Server bundle that accepts inbound RPC traffic.
    pub server: IrohBundle,

    /// Client-side endpoint used by the UI to connect to the local server.
    pub client_endpoint: Endpoint,

    /// Typed clients for the local server.
    pub profiles: ProfilesClient,
}

impl SpoutCore {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let config = config::get_or_init().await?;
        println!("{config:?}");
        // ----------------
        // Server endpoint
        // ----------------
        let mut server_builder = IrohBundle::builder(Some(config.secret_key.clone())).await?;
        let server_endpoint = server_builder.endpoint().clone();

        // DB + migrations
        let db = models::open_or_create_db(&config).await;
        models::migrate_up(db.clone()).await;

        let profiles_service = ProfilesService::new(db.clone());

        // Register RPC servers
        let rpc_server_builder = RpcServerBuilder::new(ALPN, server_endpoint.clone());

        let rpc_server_builder = profiles_service.register_service(rpc_server_builder);

        let rpc_server = rpc_server_builder.build();

        let server = server_builder.accept(ALPN, rpc_server).finish().await;

        server.wait_online().await;

        // ----------------
        // Client endpoint (for UI)
        // ----------------
        let client_endpoint = Endpoint::builder()
            .secret_key(config.client_secret_key.clone())
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;

        client_endpoint.online().await;

        // Connect client endpoint -> server endpoint
        let conn = client_endpoint
            .connect(server.endpoint.addr(), ALPN)
            .await?;

        let rpc = RpcClient::new(conn).await?;
        let profiles = ProfilesClient::new(rpc);

        if profiles.list_profiles().await?.is_empty() {
            profiles
                .create_profile("Default".to_string(), "Default profile".to_string(), None)
                .await?;
        }

        Ok(Self {
            config,
            server,
            client_endpoint,
            profiles,
        })
    }

    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        // Close client endpoint
        self.client_endpoint.close().await;

        // Shutdown server bundle
        self.server.shutdown(Duration::from_secs(5)).await?;
        Ok(())
    }
}

pub mod prelude {
    pub use super::ids;
    pub use super::entity;
    pub use super::models;

    pub use super::service;

    pub use super::error;

    pub use super::config;

    pub use zel_core;
}
