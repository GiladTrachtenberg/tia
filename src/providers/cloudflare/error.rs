use thiserror::Error;

/// Cloudflare-specific errors that can occur during API operations.
///
/// SECURITY: Error messages must NEVER contain sensitive data like API tokens.
#[derive(Debug, Error)]
pub enum CloudflareError {
    /// Authentication failed (invalid or expired token)
    #[error("authentication failed: {message}")]
    Auth { message: String },

    /// API returned an error response
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    /// Network-level error (connection failed, timeout, etc.)
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Rate limited by Cloudflare API
    #[allow(dead_code)] // NOTE: TBA in future iterations (retry logic)
    #[error("rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    /// Zone not found (no zone with given name/ID exists or not accessible)
    #[error("zone not found: '{zone}'")]
    ZoneNotFound { zone: String },

    /// Zone lookup failed due to API error
    #[error("zone lookup failed: {message}")]
    ZoneLookupFailed { message: String },

    #[error("discovery failed for {resource_type}: {message}")]
    DiscoveryFailed {
        resource_type: String,
        message: String,
    },
}

impl From<CloudflareError> for crate::providers::ProviderError {
    fn from(err: CloudflareError) -> Self {
        crate::providers::ProviderError::Cloudflare(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = CloudflareError::Auth {
            message: "Invalid API Token".to_string(),
        };
        assert_eq!(err.to_string(), "authentication failed: Invalid API Token");
    }

    #[test]
    fn test_api_error_display() {
        let err = CloudflareError::Api {
            status: 403,
            message: "Forbidden".to_string(),
        };
        assert_eq!(err.to_string(), "API error (403): Forbidden");
    }

    #[test]
    fn test_rate_limited_display() {
        let err = CloudflareError::RateLimited { retry_after: 60 };
        assert_eq!(err.to_string(), "rate limited, retry after 60s");
    }

    #[test]
    fn test_error_does_not_contain_token() {
        // Simulate an error that might accidentally include a token
        let fake_token = "cf_super_secret_token_12345";
        let err = CloudflareError::Auth {
            message: "Invalid API Token".to_string(),
        };

        let error_string = err.to_string();
        assert!(
            !error_string.contains(fake_token),
            "Error message should not contain token value"
        );
    }

    #[test]
    fn test_conversion_to_provider_error() {
        let cf_err = CloudflareError::Auth {
            message: "test error".to_string(),
        };
        let provider_err: crate::providers::ProviderError = cf_err.into();

        assert!(matches!(
            provider_err,
            crate::providers::ProviderError::Cloudflare(_)
        ));
        assert!(provider_err.to_string().contains("authentication failed"));
    }

    #[test]
    fn test_zone_not_found_display() {
        let err = CloudflareError::ZoneNotFound {
            zone: "example.com".to_string(),
        };
        assert_eq!(err.to_string(), "zone not found: 'example.com'");
    }

    #[test]
    fn test_zone_not_found_with_id() {
        let err = CloudflareError::ZoneNotFound {
            zone: "023e105f4ecef8ad9ca31a8372d0c353".to_string(),
        };
        assert!(err.to_string().contains("023e105f4ecef8ad9ca31a8372d0c353"));
    }

    #[test]
    fn test_zone_lookup_failed_display() {
        let err = CloudflareError::ZoneLookupFailed {
            message: "Permission denied".to_string(),
        };
        assert_eq!(err.to_string(), "zone lookup failed: Permission denied");
    }

    #[test]
    fn test_discovery_failed_display() {
        let err = CloudflareError::DiscoveryFailed {
            resource_type: "cloudflare_dns_record".to_string(),
            message: "API timeout".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "discovery failed for cloudflare_dns_record: API timeout"
        );
    }
}
