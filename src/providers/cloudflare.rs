mod client;
mod error;
mod types;

pub use client::CloudflareClient;
pub use error::CloudflareError;
pub use types::{PagedResponse, PaginationStrategy, ZoneInfo, is_zone_id};

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

        let client =
            CloudflareClient::new(token).map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        client
            .verify_auth()
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        tracing::info!("Cloudflare authentication verified");

        let zone = config.zone.as_ref().ok_or_else(|| {
            ProviderError::Cloudflare(
                "No zone provided. Set CLOUDFLARE_ZONE_ID or use --zone flag".to_string(),
            )
        })?;

        let zone_info = client
            .lookup_zone(zone)
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        tracing::info!(
            zone_id = %zone_info.zone_id,
            account_id = %zone_info.account_id,
            "Zone lookup successful"
        );

        let dns_records = client
            .discover_dns_records(&zone_info.zone_id)
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        let mut resources: Vec<Resource> = dns_records
            .into_iter()
            .map(|record| record.into_resource(&zone_info.zone_id))
            .collect();

        tracing::info!(count = resources.len(), "DNS records discovered");

        let page_rules = client
            .discover_page_rules(&zone_info.zone_id)
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        tracing::info!(count = page_rules.len(), "page rules discovered");

        let page_rule_resources: Vec<Resource> = page_rules
            .into_iter()
            .map(|rule| rule.into_resource(&zone_info.zone_id))
            .collect();

        resources.extend(page_rule_resources);

        let rulesets = client
            .discover_rulesets(&zone_info.zone_id, types::DISCOVERABLE_PHASES)
            .await
            .map_err(|e| ProviderError::Cloudflare(e.to_string()))?;

        tracing::info!(count = rulesets.len(), "rulesets discovered");

        let ruleset_resources: Vec<Resource> = rulesets
            .into_iter()
            .map(|ruleset| ruleset.into_resource(&zone_info.zone_id))
            .collect();

        resources.extend(ruleset_resources);

        Ok(resources)
    }

    fn generate_import(&self, resource: &Resource) -> String {
        format!(
            "import {{\n  to = {}.{}\n  id = \"{}\"\n}}",
            resource.resource_type, resource.name, resource.resource_id
        )
    }

    fn resource_types(&self) -> Vec<&str> {
        vec![
            "cloudflare_dns_record",
            "cloudflare_page_rule",
            "cloudflare_ruleset",
        ]
    }
}
