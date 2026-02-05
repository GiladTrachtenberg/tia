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
        ProviderCommand::Cloudflare { command } => match command {
            CloudflareCommand::Discover(args) => {
                let provider = providers::get_provider("cloudflare", args.token.clone())?;
                let config = DiscoverConfig {
                    zone: args.zone,
                    token: args.token,
                };
                let resources = provider.discover(&config).await?;
                tracing::info!(count = resources.len(), "discovery complete");
            }
            CloudflareCommand::Generate(_args) => {
                let provider = providers::get_provider("cloudflare", None)?;
                tracing::info!("Cloudflare generate - not yet implemented");
                let _ = provider; // Suppress unused warning
            }
            CloudflareCommand::Diff(_args) => {
                let provider = providers::get_provider("cloudflare", None)?;
                tracing::info!("Cloudflare diff - not yet implemented");
                let _ = provider; // Suppress unused warning
            }
        },
    }

    Ok(())
}
