//! Mini Argo CD controller.
//!
//! Phase 0: load Kubernetes configuration, connect to the API server,
//! and keep running until terminated.

use kube::{Client, Config};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("loading Kubernetes configuration");
    let config = Config::infer().await?;

    tracing::info!("connecting to Kubernetes API server");
    let client = Client::try_from(config)?;
    let version = client.apiserver_version().await?;

    tracing::info!(
        git_version = %version.git_version,
        "connected to Kubernetes API server"
    );

    tracing::info!("controller is running");
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");

    Ok(())
}
