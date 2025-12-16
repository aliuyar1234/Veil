//! Veil API Server
//!
//! A REST API server for the Veil privacy platform.

use std::env;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use veil_api::{run_server, ServerConfig};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "veil_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = if let Ok(config_path) = env::var("VEIL_CONFIG") {
        match ServerConfig::from_file(&config_path) {
            Ok(config) => {
                tracing::info!("Loaded configuration from {}", config_path);
                config
            }
            Err(e) => {
                tracing::warn!("Failed to load config from {}: {}", config_path, e);
                tracing::info!("Using environment variables and defaults");
                ServerConfig::from_env()
            }
        }
    } else {
        tracing::info!("No VEIL_CONFIG set, using environment variables and defaults");
        ServerConfig::from_env()
    };

    tracing::info!(
        "Server configuration: host={}, port={}, max_body_size={}",
        config.host,
        config.port,
        config.max_body_size
    );

    // Run the server
    if let Err(e) = run_server(config).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
