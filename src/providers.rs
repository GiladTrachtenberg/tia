pub mod cloudflare;

use async_trait::async_trait;
use thiserror::Error;

use crate::resource::{DiscoverConfig, Resource};

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn discover(&self, config: &DiscoverConfig) -> Result<Vec<Resource>, ProviderError>;
    fn generate_import(&self, resource: &Resource) -> String;
    fn resource_types(&self) -> Vec<&str>;
}

pub fn get_provider(name: &str) -> Result<Box<dyn Provider>, ProviderError> {
    match name {
        "cloudflare" => Ok(Box::new(cloudflare::CloudflareProvider::new())),
        other => Err(ProviderError::UnknownProvider(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider_cloudflare() {
        let provider = get_provider("cloudflare").unwrap();
        assert_eq!(provider.name(), "cloudflare");
    }

    #[test]
    fn test_get_provider_unknown() {
        let result = get_provider("unknown");
        assert!(result.is_err());
        match result {
            Err(ProviderError::UnknownProvider(name)) => assert_eq!(name, "unknown"),
            _ => panic!("expected UnknownProvider error"),
        }
    }

    #[test]
    fn test_cloudflare_resource_types() {
        let provider = cloudflare::CloudflareProvider::new();
        let types = provider.resource_types();
        assert!(types.contains(&"cloudflare_record"));
        assert!(types.contains(&"cloudflare_page_rule"));
        assert!(types.contains(&"cloudflare_firewall_rule"));
        assert!(types.contains(&"cloudflare_worker_script"));
        assert!(types.contains(&"cloudflare_waf_rule"));
    }

    #[tokio::test]
    async fn test_cloudflare_discover_returns_empty() {
        let provider = cloudflare::CloudflareProvider::new();
        let config = DiscoverConfig::default();
        let resources = provider.discover(&config).await.unwrap();
        assert!(resources.is_empty());
    }

    #[test]
    fn test_cloudflare_generate_import_placeholder() {
        let provider = cloudflare::CloudflareProvider::new();
        let resource = Resource {
            resource_type: "cloudflare_record".to_string(),
            resource_id: "abc123".to_string(),
            name: "example".to_string(),
            zone_id: "zone456".to_string(),
            metadata: serde_json::json!({}),
        };
        let import = provider.generate_import(&resource);
        assert!(import.contains("import {"));
        assert!(import.contains("cloudflare_record"));
        assert!(import.contains("abc123"));
    }
}
