use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // NOTE: TBA in future iterations (unified error handling)
pub enum TiaError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error(transparent)]
    Provider(#[from] crate::providers::ProviderError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("configuration error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_auth_error_display() {
        let err = TiaError::Auth("invalid token".to_string());
        assert_eq!(err.to_string(), "authentication failed: invalid token");
    }

    #[test]
    fn test_cache_error_display() {
        let err = TiaError::Cache("disk full".to_string());
        assert_eq!(err.to_string(), "cache error: disk full");
    }

    #[test]
    fn test_config_error_display() {
        let err = TiaError::Config("missing field".to_string());
        assert_eq!(err.to_string(), "configuration error: missing field");
    }

    #[test]
    fn test_io_error_from_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let tia_err: TiaError = io_err.into();
        assert!(matches!(tia_err, TiaError::Io(_)));
        assert!(tia_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_provider_error_from_conversion() {
        let provider_err = crate::providers::ProviderError::UnknownProvider("aws".to_string());
        let tia_err: TiaError = provider_err.into();
        assert!(matches!(tia_err, TiaError::Provider(_)));
        assert!(tia_err.to_string().contains("unknown provider: aws"));
    }
}
