use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: String,
    pub name: String,
    pub zone_id: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoverConfig {
    pub zone: Option<String>,
    pub token: Option<String>,
    #[allow(dead_code)] // NOTE: Populated after zone lookup
    pub zone_id: Option<String>,
    #[allow(dead_code)] // NOTE: Populated after zone lookup, needed for Workers Scripts
    pub account_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_serialization_snake_case() {
        let resource = Resource {
            resource_type: "cloudflare_record".to_string(),
            resource_id: "abc123".to_string(),
            name: "api.example.com".to_string(),
            zone_id: "zone456".to_string(),
            metadata: serde_json::json!({"record_type": "A"}),
        };
        let json = serde_json::to_string(&resource).unwrap();
        assert!(json.contains("resource_type"));
        assert!(json.contains("resource_id"));
        assert!(json.contains("zone_id"));
        assert!(!json.contains("resourceType"));
        assert!(!json.contains("resourceId"));
        assert!(!json.contains("zoneId"));
    }

    #[test]
    fn test_resource_deserialization() {
        let json = r#"{
            "resource_type": "cloudflare_record",
            "resource_id": "abc123",
            "name": "api.example.com",
            "zone_id": "zone456",
            "metadata": {"record_type": "A"}
        }"#;
        let resource: Resource = serde_json::from_str(json).unwrap();
        assert_eq!(resource.resource_type, "cloudflare_record");
        assert_eq!(resource.resource_id, "abc123");
        assert_eq!(resource.name, "api.example.com");
        assert_eq!(resource.zone_id, "zone456");
        assert_eq!(resource.metadata["record_type"], "A");
    }

    #[test]
    fn test_resource_roundtrip() {
        let resource = Resource {
            resource_type: "cloudflare_record".to_string(),
            resource_id: "abc123".to_string(),
            name: "test".to_string(),
            zone_id: "zone789".to_string(),
            metadata: serde_json::json!(null),
        };
        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(resource, deserialized);
    }
}
