mod cache;
mod cli;
mod error;
mod output;
mod providers;
mod resource;
mod terraform;

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::EnvFilter;

use cli::{Cli, CloudflareCommand, ProviderCommand};
use resource::DiscoverConfig;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        ProviderCommand::Cloudflare { command } => {
            let provider = providers::get_provider("cloudflare")?;
            match command {
                CloudflareCommand::Discover(_args) => {
                    let config = DiscoverConfig::default();
                    let resources = provider.discover(&config).await?;
                    tracing::info!(count = resources.len(), "discovery complete");
                    tracing::warn!("Cloudflare provider not yet implemented");
                }
                CloudflareCommand::Generate(_args) => {
                    tracing::info!("Cloudflare generate - not yet implemented");
                }
                CloudflareCommand::Diff(_args) => {
                    tracing::info!("Cloudflare diff - not yet implemented");
                }
            }
        }
    }

    Ok(())
}
