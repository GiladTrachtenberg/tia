use async_trait::async_trait;

use super::{DiscoverConfig, Provider, ProviderError, Resource};

pub struct CloudflareProvider;

impl CloudflareProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CloudflareProvider {
    fn name(&self) -> &str {
        "cloudflare"
    }

    async fn discover(&self, _config: &DiscoverConfig) -> Result<Vec<Resource>, ProviderError> {
        tracing::info!("Cloudflare provider not yet implemented");
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
