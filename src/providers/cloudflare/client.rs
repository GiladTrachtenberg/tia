use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use super::CloudflareError;

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// HTTP client for Cloudflare API operations.
///
/// SECURITY: The token field is private and never exposed in Debug output.
#[derive(Clone)]
pub struct CloudflareClient {
    client: reqwest::Client,
    #[allow(dead_code)] // NOTE: TBA in future iterations (needed for token refresh)
    token: String,
    base_url: String,
}

impl CloudflareClient {
    /// Creates a new CloudflareClient with the given API token.
    ///
    /// The token is stored securely and used for Bearer authentication.
    pub fn new(token: String) -> Result<Self, CloudflareError> {
        Self::with_base_url(token, CLOUDFLARE_API_BASE.to_string())
    }

    /// Creates a new CloudflareClient with a custom base URL (for testing).
    #[cfg(test)]
    pub fn with_base_url(token: String, base_url: String) -> Result<Self, CloudflareError> {
        Self::create_client(token, base_url)
    }

    #[cfg(not(test))]
    fn with_base_url(token: String, base_url: String) -> Result<Self, CloudflareError> {
        Self::create_client(token, base_url)
    }

    fn create_client(token: String, base_url: String) -> Result<Self, CloudflareError> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        let header_value =
            HeaderValue::from_str(&auth_value).map_err(|_| CloudflareError::Auth {
                message: "Invalid token format".to_string(),
            })?;
        headers.insert(AUTHORIZATION, header_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(CloudflareError::Network)?;

        Ok(Self {
            client,
            token,
            base_url,
        })
    }

    /// Verifies the API token is valid by calling Cloudflare's token verify endpoint.
    ///
    /// Returns Ok(()) if the token is valid, or an appropriate error otherwise.
    pub async fn verify_auth(&self) -> Result<(), CloudflareError> {
        let url = format!("{}/user/tokens/verify", self.base_url);

        let response = self.client.get(&url).send().await?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.map_err(|e| CloudflareError::Api {
            status: status.as_u16(),
            message: format!("Failed to parse response: {}", e),
        })?;

        if body.get("success").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(());
        }

        // Extract error message from Cloudflare response
        let error_message = body
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown authentication error");

        Err(CloudflareError::Auth {
            message: error_message.to_string(),
        })
    }

    /// Returns a reference to the underlying reqwest client for making API calls.
    // NOTE: TBA in future iterations (zone/record discovery)
    #[allow(dead_code)]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Returns the base URL for Cloudflare API.
    // NOTE: TBA in future iterations (zone/record discovery)
    #[allow(dead_code)]
    pub fn api_base(&self) -> &str {
        &self.base_url
    }
}

/// Manual Debug implementation that masks the token.
///
/// SECURITY: Never expose the actual token value in debug output.
impl std::fmt::Debug for CloudflareClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudflareClient")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CloudflareClient::new("test_token".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_debug_does_not_expose_token() {
        let client = CloudflareClient::new("super_secret_token_12345".to_string()).unwrap();
        let debug_output = format!("{:?}", client);

        assert!(
            debug_output.contains("[REDACTED]"),
            "Debug output should contain [REDACTED]"
        );
        assert!(
            !debug_output.contains("super_secret_token_12345"),
            "Debug output must NOT contain the actual token"
        );
    }

    #[test]
    fn test_client_is_clone() {
        let client = CloudflareClient::new("test_token".to_string()).unwrap();
        let _cloned = client.clone();
    }

    #[test]
    fn test_api_base_url() {
        let client = CloudflareClient::new("test_token".to_string()).unwrap();
        assert_eq!(client.api_base(), "https://api.cloudflare.com/client/v4");
    }

    #[tokio::test]
    async fn test_verify_auth_valid_token() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user/tokens/verify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "result": { "id": "abc123", "status": "active" }
            })))
            .mount(&mock_server)
            .await;

        let client =
            CloudflareClient::with_base_url("valid_token".to_string(), mock_server.uri()).unwrap();

        let result = client.verify_auth().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_auth_invalid_token() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user/tokens/verify"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "success": false,
                "errors": [{ "code": 1000, "message": "Invalid API Token" }]
            })))
            .mount(&mock_server)
            .await;

        let client =
            CloudflareClient::with_base_url("invalid_token".to_string(), mock_server.uri())
                .unwrap();

        let result = client.verify_auth().await;
        assert!(result.is_err());

        if let Err(CloudflareError::Auth { message }) = result {
            assert_eq!(message, "Invalid API Token");
        } else {
            panic!("Expected CloudflareError::Auth");
        }
    }

    #[tokio::test]
    async fn test_verify_auth_error_does_not_contain_token() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let secret_token = "cf_super_secret_token_xyz789";

        Mock::given(method("GET"))
            .and(path("/user/tokens/verify"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "success": false,
                "errors": [{ "code": 1000, "message": "Invalid API Token" }]
            })))
            .mount(&mock_server)
            .await;

        let client =
            CloudflareClient::with_base_url(secret_token.to_string(), mock_server.uri()).unwrap();

        let result = client.verify_auth().await;
        let error_string = format!("{:?}", result);

        assert!(
            !error_string.contains(secret_token),
            "Error output must not contain the token"
        );
    }
}
