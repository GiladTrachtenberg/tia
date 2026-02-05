mod client;
mod error;

pub use client::CloudflareClient;
pub use error::CloudflareError;

use async_trait::async_trait;

use super::{DiscoverConfig, Provider, ProviderError, Resource};

pub struct CloudflareProvider {
    token: Option<String>,
}

impl CloudflareProvider {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl Provider for CloudflareProvider {
    fn name(&self) -> &str {
        "cloudflare"
    }

    async fn discover(&self, config: &DiscoverConfig) -> Result<Vec<Resource>, ProviderError> {
        // Token from constructor takes precedence, then config, then error
        let token = self
            .token
            .clone()
            .or_else(|| config.token.clone())
            .ok_or_else(|| {
                ProviderError::Auth(
                    "No API token provided. Set CLOUDFLARE_API_TOKEN or use --token flag"
                        .to_string(),
                )
            })?;

        // Create client and verify authentication (fail fast)
        let client =
            CloudflareClient::new(token).map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        client
            .verify_auth()
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        tracing::info!("Cloudflare authentication verified successfully");

        // TODO: Implement actual resource discovery in Epic 2 stories
        Ok(vec![])
    }

    fn generate_import(&self, resource: &Resource) -> String {
        format!(
            "import {{\n  to = {}.{}\n  id = \"{}\"\n}}",
            resource.resource_type, resource.name, resource.resource_id
        )
    }

    fn resource_types(&self) -> Vec<&str> {
        vec![
            "cloudflare_record",
            "cloudflare_page_rule",
            "cloudflare_firewall_rule",
            "cloudflare_worker_script",
            "cloudflare_waf_rule",
        ]
    }
}
