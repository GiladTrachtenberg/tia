use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use super::CloudflareError;
use super::types::{DEFAULT_PAGE_SIZE, DnsRecord, Zone, ZoneInfo, is_zone_id};

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

#[derive(Clone)]
pub struct CloudflareClient {
    client: reqwest::Client,
    #[allow(dead_code)] // NOTE: TBA in future iterations (needed for token refresh)
    token: String,
    base_url: String,
}

impl CloudflareClient {
    pub fn new(token: String) -> Result<Self, CloudflareError> {
        Self::with_base_url(token, CLOUDFLARE_API_BASE.to_string())
    }

    /// NOTE: Primarily used for testing with mock servers.
    pub fn with_base_url(token: String, base_url: String) -> Result<Self, CloudflareError> {
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

    #[allow(dead_code)] // NOTE: Used by resource discovery
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    #[allow(dead_code)] // NOTE: Used by resource discovery
    pub fn api_base(&self) -> &str {
        &self.base_url
    }

    // NOTE: Auto-detects zone ID (32-char hex) vs zone name
    pub async fn lookup_zone(&self, zone: &str) -> Result<ZoneInfo, CloudflareError> {
        if is_zone_id(zone) {
            self.lookup_zone_by_id(zone).await
        } else {
            self.lookup_zone_by_name(zone).await
        }
    }

    async fn lookup_zone_by_id(&self, zone_id: &str) -> Result<ZoneInfo, CloudflareError> {
        let url = format!("{}/zones/{}", self.base_url, zone_id);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        let body: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| CloudflareError::ZoneLookupFailed {
                    message: format!("Failed to parse response: {}", e),
                })?;

        let success = body
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let error_msg = body
                .get("errors")
                .and_then(|e| e.as_array())
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            if status.as_u16() == 404 || error_msg.to_lowercase().contains("not found") {
                return Err(CloudflareError::ZoneNotFound {
                    zone: zone_id.to_string(),
                });
            }

            return Err(CloudflareError::ZoneLookupFailed { message: error_msg });
        }

        let result: Zone = serde_json::from_value(body["result"].clone()).map_err(|e| {
            CloudflareError::ZoneLookupFailed {
                message: format!("Failed to parse zone: {}", e),
            }
        })?;

        Ok(ZoneInfo {
            zone_id: result.id,
            account_id: result.account.id,
        })
    }

    async fn lookup_zone_by_name(&self, zone_name: &str) -> Result<ZoneInfo, CloudflareError> {
        let encoded_name = urlencoding::encode(zone_name);
        let url = format!("{}/zones?name={}", self.base_url, encoded_name);

        let response = self.client.get(&url).send().await?;

        let body: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| CloudflareError::ZoneLookupFailed {
                    message: format!("Failed to parse response: {}", e),
                })?;

        let success = body
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let error_msg = body
                .get("errors")
                .and_then(|e| e.as_array())
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(CloudflareError::ZoneLookupFailed { message: error_msg });
        }

        let zones: Vec<Zone> = serde_json::from_value(body["result"].clone()).map_err(|e| {
            CloudflareError::ZoneLookupFailed {
                message: format!("Failed to parse zones: {}", e),
            }
        })?;

        let zone = zones
            .into_iter()
            .next()
            .ok_or_else(|| CloudflareError::ZoneNotFound {
                zone: zone_name.to_string(),
            })?;

        Ok(ZoneInfo {
            zone_id: zone.id,
            account_id: zone.account.id,
        })
    }

    pub async fn discover_dns_records(
        &self,
        zone_id: &str,
    ) -> Result<Vec<DnsRecord>, CloudflareError> {
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);

        self.fetch_all_pages(&url, DEFAULT_PAGE_SIZE, |result| async move {
            serde_json::from_value::<Vec<DnsRecord>>(result).map_err(|e| {
                CloudflareError::DiscoveryFailed {
                    resource_type: "cloudflare_dns_record".to_string(),
                    message: format!("Failed to parse DNS records: {}", e),
                }
            })
        })
        .await
    }

    pub async fn fetch_all_pages<T, F, Fut>(
        &self,
        base_url: &str,
        page_size: u32,
        parse_fn: F,
    ) -> Result<Vec<T>, CloudflareError>
    where
        F: Fn(serde_json::Value) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<T>, CloudflareError>>,
    {
        let mut all_results = Vec::new();
        let mut page = 1u32;

        loop {
            let url = format!("{}?page={}&per_page={}", base_url, page, page_size);
            let response = self.client.get(&url).send().await?;

            let body: serde_json::Value =
                response.json().await.map_err(|e| CloudflareError::Api {
                    status: 0,
                    message: format!("Failed to parse response: {}", e),
                })?;

            let success = body
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !success {
                let error_msg = body
                    .get("errors")
                    .and_then(|e| e.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return Err(CloudflareError::Api {
                    status: 0,
                    message: error_msg,
                });
            }

            let page_results = parse_fn(body["result"].clone()).await?;
            let count = page_results.len();
            all_results.extend(page_results);

            let total_count = body
                .get("result_info")
                .and_then(|ri| ri.get("total_count"))
                .and_then(|tc| tc.as_u64())
                .unwrap_or(0) as u32;

            if page * page_size >= total_count || count == 0 {
                break;
            }

            page += 1;
        }

        Ok(all_results)
    }

    #[allow(dead_code)] // NOTE: Used by ruleset discovery
    pub async fn fetch_all_cursors<T, F, Fut>(
        &self,
        base_url: &str,
        page_size: u32,
        parse_fn: F,
    ) -> Result<Vec<T>, CloudflareError>
    where
        F: Fn(serde_json::Value) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<T>, CloudflareError>>,
    {
        let mut all_results = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let url = match &cursor {
                Some(c) => format!("{}?per_page={}&cursor={}", base_url, page_size, c),
                None => format!("{}?per_page={}", base_url, page_size),
            };

            let response = self.client.get(&url).send().await?;

            let body: serde_json::Value =
                response.json().await.map_err(|e| CloudflareError::Api {
                    status: 0,
                    message: format!("Failed to parse response: {}", e),
                })?;

            let success = body
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !success {
                let error_msg = body
                    .get("errors")
                    .and_then(|e| e.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return Err(CloudflareError::Api {
                    status: 0,
                    message: error_msg,
                });
            }

            let page_results = parse_fn(body["result"].clone()).await?;
            all_results.extend(page_results);

            let next_cursor = body
                .get("result_info")
                .and_then(|ri| ri.get("cursors"))
                .and_then(|c| c.get("after"))
                .and_then(|a| a.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            match next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }

        Ok(all_results)
    }
}

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
}
