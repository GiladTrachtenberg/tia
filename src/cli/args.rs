use clap::{Parser, Subcommand};

/// TIA - Terraform Import Accelerator
///
/// Discovers cloud provider resources and generates Terraform import blocks.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: ProviderCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommand {
    /// Cloudflare provider commands
    Cloudflare {
        #[command(subcommand)]
        command: CloudflareCommand,
    },
    // Future: Aws, Gcp, etc.
}

#[derive(Subcommand, Debug)]
pub enum CloudflareCommand {
    /// Discover all resources in a Cloudflare zone
    Discover(DiscoverArgs),
    /// Generate Terraform import blocks for discovered resources
    Generate(GenerateArgs),
    /// Compare cloud resources against Terraform state
    Diff(DiffArgs),
}

#[derive(clap::Args, Debug)]
pub struct DiscoverArgs {
    // Placeholder - options added in Epic 2
}

#[derive(clap::Args, Debug)]
pub struct GenerateArgs {
    // Placeholder - options added in Epic 3
}

#[derive(clap::Args, Debug)]
pub struct DiffArgs {
    // Placeholder - options added in Epic 5
}
