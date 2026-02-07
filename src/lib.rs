//! TIA - Terraform Import Accelerator
//!
//! A library for discovering cloud provider resources and generating Terraform import blocks.

pub mod providers;
pub mod resource;

mod cache;
mod error;
mod output;
mod terraform;

pub use providers::cloudflare::{CloudflareClient, CloudflareError, ZoneInfo};
pub use resource::{DiscoverConfig, Resource};
