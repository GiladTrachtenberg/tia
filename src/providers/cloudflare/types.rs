use serde::Deserialize;

#[allow(dead_code)] // NOTE: Used by pagination helpers
pub const DEFAULT_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneInfo {
    pub zone_id: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaginationStrategy {
    PageBased,
    CursorBased,
    SinglePage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total_count: Option<u32>,
    pub has_more: bool,
}

impl<T> PagedResponse<T> {
    pub fn new(items: Vec<T>, total_count: Option<u32>, has_more: bool) -> Self {
        Self {
            items,
            total_count,
            has_more,
        }
    }

    pub fn single_page(items: Vec<T>) -> Self {
        Self {
            items,
            total_count: None,
            has_more: false,
        }
    }
}

#[allow(dead_code)] // NOTE: Used in tests and pagination helpers
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub success: bool,
    #[serde(default)]
    pub errors: Vec<CloudflareApiError>,
    pub result: Option<T>,
    #[serde(default)]
    #[allow(dead_code)] // NOTE: Used by pagination helpers
    pub result_info: Option<ResultInfo>,
}

#[allow(dead_code)] // NOTE: Used in tests
#[derive(Debug, Deserialize)]
pub struct CloudflareApiError {
    #[allow(dead_code)] // NOTE: Used for error logging
    pub code: u32,
    pub message: String,
}

#[allow(dead_code)] // NOTE: Used by pagination helpers
#[derive(Debug, Deserialize, Default)]
pub struct ResultInfo {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total_count: Option<u32>,
    #[serde(default)]
    pub cursors: Option<Cursors>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Cursors {
    #[allow(dead_code)] // NOTE: Used by pagination helpers
    pub after: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
    #[allow(dead_code)] // NOTE: Used for logging
    pub name: String,
    pub account: ZoneAccount,
}

#[derive(Debug, Deserialize)]
pub struct ZoneAccount {
    pub id: String,
    #[allow(dead_code)] // NOTE: Used for logging
    pub name: String,
}

pub fn is_zone_id(input: &str) -> bool {
    input.len() == 32 && input.chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(default)]
    pub zone_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl DnsRecord {
    pub fn into_resource(self, zone_id: &str) -> crate::resource::Resource {
        crate::resource::Resource {
            resource_type: "cloudflare_dns_record".to_string(),
            resource_id: self.id,
            name: self.name,
            zone_id: self.zone_id.unwrap_or_else(|| zone_id.to_string()),
            metadata: serde_json::json!({
                "type": self.type_,
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PageRuleTarget {
    pub target: String,
    pub constraint: PageRuleConstraint,
}

#[derive(Debug, Deserialize)]
pub struct PageRuleConstraint {
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PageRule {
    pub id: String,
    pub targets: Vec<PageRuleTarget>,
}

impl PageRule {
    pub fn into_resource(self, zone_id: &str) -> crate::resource::Resource {
        let name = self
            .targets
            .first()
            .map(|t| t.constraint.value.clone())
            .unwrap_or_else(|| self.id.clone());

        crate::resource::Resource {
            resource_type: "cloudflare_page_rule".to_string(),
            resource_id: self.id,
            name,
            zone_id: zone_id.to_string(),
            metadata: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_record_deserialization_with_serde_rename() {
        let json = r#"{
            "id": "023e105f4ecef8ad9ca31a8372d0c353",
            "zone_id": "abc123def456",
            "zone_name": "example.com",
            "name": "api.example.com",
            "type": "A",
            "content": "198.51.100.4",
            "ttl": 3600,
            "proxiable": true,
            "proxied": true,
            "comment": "API server",
            "created_on": "2014-01-01T05:20:00.12345Z",
            "modified_on": "2014-01-01T05:20:00.12345Z"
        }"#;

        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.id, "023e105f4ecef8ad9ca31a8372d0c353");
        assert_eq!(record.name, "api.example.com");
        assert_eq!(record.type_, "A");
    }

    #[test]
    fn test_dns_record_deserialization_ignores_unknown_fields() {
        let json = r#"{
            "id": "record123",
            "zone_id": "zone456",
            "name": "www.example.com",
            "type": "CNAME",
            "content": "example.com",
            "ttl": 1
        }"#;

        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.id, "record123");
        assert_eq!(record.zone_id, Some("zone456".to_string()));
        assert_eq!(record.name, "www.example.com");
        assert_eq!(record.type_, "CNAME");
    }

    #[test]
    fn test_dns_record_to_resource_zone_id_fallback() {
        let record = DnsRecord {
            id: "rec789".to_string(),
            zone_id: None,
            name: "fallback.example.com".to_string(),
            type_: "AAAA".to_string(),
        };

        let resource = record.into_resource("fallback_zone");

        assert_eq!(resource.zone_id, "fallback_zone");
        assert_eq!(resource.resource_id, "rec789");
        assert_eq!(resource.metadata, serde_json::json!({"type": "AAAA"}));
    }

    #[test]
    fn test_dns_record_to_resource_type_is_cloudflare_dns_record() {
        let record = DnsRecord {
            id: "rec123".to_string(),
            zone_id: Some("zone456".to_string()),
            name: "api.example.com".to_string(),
            type_: "A".to_string(),
        };

        let resource = record.into_resource("zone456");

        assert_eq!(
            resource.resource_type, "cloudflare_dns_record",
            "CRITICAL: resource_type must be 'cloudflare_dns_record' (Terraform v5 naming)"
        );
        assert_eq!(resource.resource_id, "rec123");
        assert_eq!(resource.name, "api.example.com");
        assert_eq!(resource.zone_id, "zone456");
        assert_eq!(resource.metadata, serde_json::json!({"type": "A"}));
    }

    #[test]
    fn test_zone_info_fields() {
        let info = ZoneInfo {
            zone_id: "abc123".to_string(),
            account_id: "def456".to_string(),
        };
        assert_eq!(info.zone_id, "abc123");
        assert_eq!(info.account_id, "def456");
    }

    #[test]
    fn test_zone_info_clone() {
        let info = ZoneInfo {
            zone_id: "abc123".to_string(),
            account_id: "def456".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }

    #[test]
    fn test_is_zone_id_valid_32_hex() {
        assert!(is_zone_id("023e105f4ecef8ad9ca31a8372d0c353"));
        assert!(is_zone_id("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(is_zone_id("0123456789abcdef0123456789abcdef"));
        assert!(is_zone_id("ABCDEF0123456789ABCDEF0123456789"));
    }

    #[test]
    fn test_is_zone_id_invalid_domain_names() {
        assert!(!is_zone_id("example.com"));
        assert!(!is_zone_id("my-zone.io"));
        assert!(!is_zone_id("subdomain.example.org"));
    }

    #[test]
    fn test_is_zone_id_invalid_length() {
        assert!(!is_zone_id("abc123"));
        assert!(!is_zone_id("023e105f4ecef8ad9ca31a8372d0c35"));
        assert!(!is_zone_id("023e105f4ecef8ad9ca31a8372d0c3530"));
    }

    #[test]
    fn test_is_zone_id_invalid_non_hex() {
        assert!(!is_zone_id("023e105f4ecef8ad9ca31a8372d0c35g"));
        assert!(!is_zone_id("023e105f4ecef8ad-ca31a8372d0c353"));
        assert!(!is_zone_id("023e105f4ecef8ad ca31a8372d0c353"));
    }

    #[test]
    fn test_is_zone_id_empty_string() {
        assert!(!is_zone_id(""));
    }

    #[test]
    fn test_pagination_strategy_variants() {
        let _page = PaginationStrategy::PageBased;
        let _cursor = PaginationStrategy::CursorBased;
        let _single = PaginationStrategy::SinglePage;
    }

    #[test]
    fn test_cloudflare_response_deserialization() {
        let json = r#"{
            "success": true,
            "errors": [],
            "result": [{"id": "zone123", "name": "example.com", "account": {"id": "acc456", "name": "Test Account"}}],
            "result_info": {"page": 1, "per_page": 20, "total_count": 1}
        }"#;

        let response: CloudflareResponse<Vec<Zone>> = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert!(response.errors.is_empty());
        let result = response.result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "zone123");
        assert_eq!(result[0].name, "example.com");
        assert_eq!(result[0].account.id, "acc456");
        assert_eq!(result[0].account.name, "Test Account");
    }

    #[test]
    fn test_cloudflare_response_with_errors() {
        let json = r#"{
            "success": false,
            "errors": [{"code": 1000, "message": "Invalid API Token"}],
            "result": null
        }"#;

        let response: CloudflareResponse<Zone> = serde_json::from_str(json).unwrap();
        assert!(!response.success);
        assert!(response.result.is_none());
        assert_eq!(response.errors.len(), 1);
        assert_eq!(response.errors[0].code, 1000);
        assert_eq!(response.errors[0].message, "Invalid API Token");
    }

    #[test]
    fn test_result_info_with_cursors() {
        let json = r#"{
            "success": true,
            "errors": [],
            "result": [],
            "result_info": {"cursors": {"after": "next_cursor_token"}}
        }"#;

        let response: CloudflareResponse<Vec<Zone>> = serde_json::from_str(json).unwrap();
        let cursors = response.result_info.unwrap().cursors.unwrap();
        assert_eq!(cursors.after, Some("next_cursor_token".to_string()));
    }

    #[test]
    fn test_paged_response_new() {
        let response = PagedResponse::new(vec![1, 2, 3], Some(100), true);
        assert_eq!(response.items, vec![1, 2, 3]);
        assert_eq!(response.total_count, Some(100));
        assert!(response.has_more);
    }

    #[test]
    fn test_paged_response_single_page() {
        let response = PagedResponse::single_page(vec!["a", "b"]);
        assert_eq!(response.items, vec!["a", "b"]);
        assert_eq!(response.total_count, None);
        assert!(!response.has_more);
    }

    #[test]
    fn test_paged_response_clone() {
        let response = PagedResponse::new(vec![1, 2], Some(10), false);
        let cloned = response.clone();
        assert_eq!(response, cloned);
    }

    #[test]
    fn test_page_rule_deserialization() {
        let json = r#"{
            "id": "023e105f4ecef8ad9ca31a8372d0c353",
            "status": "active",
            "priority": 1,
            "created_on": "2014-01-01T05:20:00.12345Z",
            "modified_on": "2014-01-01T05:20:00.12345Z",
            "targets": [
                {
                    "target": "url",
                    "constraint": {
                        "operator": "matches",
                        "value": "*example.com/images/*"
                    }
                }
            ],
            "actions": [
                {
                    "id": "browser_check",
                    "value": "on"
                }
            ]
        }"#;

        let rule: PageRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.id, "023e105f4ecef8ad9ca31a8372d0c353");
        assert_eq!(rule.targets.len(), 1);
        assert_eq!(rule.targets[0].target, "url");
        assert_eq!(rule.targets[0].constraint.operator, "matches");
        assert_eq!(rule.targets[0].constraint.value, "*example.com/images/*");
    }

    #[test]
    fn test_page_rule_to_resource() {
        let rule = PageRule {
            id: "rule123".to_string(),
            targets: vec![PageRuleTarget {
                target: "url".to_string(),
                constraint: PageRuleConstraint {
                    operator: "matches".to_string(),
                    value: "*example.com/images/*".to_string(),
                },
            }],
        };

        let resource = rule.into_resource("zone456");

        assert_eq!(resource.resource_type, "cloudflare_page_rule");
        assert_eq!(resource.resource_id, "rule123");
        assert_eq!(resource.name, "*example.com/images/*");
        assert_eq!(resource.zone_id, "zone456");
        assert_eq!(resource.metadata, serde_json::json!({}));
    }

    #[test]
    fn test_page_rule_to_resource_empty_targets_fallback() {
        let rule = PageRule {
            id: "rule_no_targets".to_string(),
            targets: vec![],
        };

        let resource = rule.into_resource("zone789");

        assert_eq!(resource.name, "rule_no_targets");
    }
}
