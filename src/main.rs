use anyhow::Result;
use tokio::signal;
use tracing::{error, info};

mod mcp;
mod server;
mod settings;
mod signal_cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("starting signal-mcp-server");

    let config = settings::Settings::load()?;
    let server = server::Server::new(config).await?;

    tokio::select! {
        result = server.run() => {
            if let Err(err) = result {
                error!(?err, "server terminated with error");
            }
        }
        _ = signal::ctrl_c() => {
            info!("shutdown signal received");
        }
    }

    Ok(())
}
