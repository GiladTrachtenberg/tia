pub mod cloudflare;

use async_trait::async_trait;
use thiserror::Error;

use crate::resource::{DiscoverConfig, Resource};

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
    #[allow(dead_code)] // NOTE: TBA in future iterations
    #[error("not implemented: {0}")]
    NotImplemented(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("cloudflare error: {0}")]
    Cloudflare(String),
}

#[async_trait]
#[allow(dead_code)] // NOTE: TBA in future iterations (full CLI integration)
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn discover(&self, config: &DiscoverConfig) -> Result<Vec<Resource>, ProviderError>;
    fn generate_import(&self, resource: &Resource) -> String;
    fn resource_types(&self) -> Vec<&str>;
}

pub fn get_provider(name: &str, token: Option<String>) -> Result<Box<dyn Provider>, ProviderError> {
    match name {
        "cloudflare" => Ok(Box::new(cloudflare::CloudflareProvider::new(token))),
        other => Err(ProviderError::UnknownProvider(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider_cloudflare() {
        let provider = get_provider("cloudflare", None).unwrap();
        assert_eq!(provider.name(), "cloudflare");
    }

    #[test]
    fn test_get_provider_unknown() {
        let result = get_provider("unknown", None);
        assert!(result.is_err());
        match result {
            Err(ProviderError::UnknownProvider(name)) => assert_eq!(name, "unknown"),
            _ => panic!("expected UnknownProvider error"),
        }
    }

    #[test]
    fn test_cloudflare_resource_types() {
        let provider = cloudflare::CloudflareProvider::new(None);
        let types = provider.resource_types();
        assert!(types.contains(&"cloudflare_dns_record"));
        assert!(types.contains(&"cloudflare_page_rule"));
        assert!(types.contains(&"cloudflare_ruleset"));
        assert!(!types.contains(&"cloudflare_firewall_rule"));
        assert!(!types.contains(&"cloudflare_waf_rule"));
    }

    #[tokio::test]
    async fn test_cloudflare_discover_no_token_error() {
        let provider = cloudflare::CloudflareProvider::new(None);
        let config = DiscoverConfig::default();
        let result = provider.discover(&config).await;

        assert!(result.is_err());
        if let Err(ProviderError::Auth(msg)) = result {
            assert!(msg.contains("No API token provided"));
        } else {
            panic!("Expected ProviderError::Auth");
        }
    }

    #[test]
    fn test_cloudflare_generate_import_placeholder() {
        let provider = cloudflare::CloudflareProvider::new(None);
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
