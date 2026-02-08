use tia::{CloudflareClient, CloudflareError};
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_discover_rulesets_phase_filtering() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/rulesets"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [
                {
                    "id": "rs_redirect",
                    "name": "My Redirect Rules",
                    "phase": "http_request_dynamic_redirect",
                    "kind": "zone",
                    "description": "test",
                    "version": "1"
                },
                {
                    "id": "rs_managed",
                    "name": "Cloudflare Managed",
                    "phase": "http_request_firewall_managed",
                    "kind": "managed",
                    "description": "",
                    "version": "34"
                },
                {
                    "id": "rs_rewrite",
                    "name": "URL Rewrite Rules",
                    "phase": "http_request_transform",
                    "kind": "zone",
                    "description": "",
                    "version": "2"
                }
            ],
            "result_info": { "cursors": {} }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let phases = &[
        "http_request_dynamic_redirect",
        "http_request_transform",
        "http_request_firewall_custom",
    ];
    let result = client.discover_rulesets("zone123", phases).await.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "rs_redirect");
    assert_eq!(result[0].phase, "http_request_dynamic_redirect");
    assert_eq!(result[1].id, "rs_rewrite");
    assert_eq!(result[1].phase, "http_request_transform");
}

#[tokio::test]
async fn test_discover_rulesets_cursor_pagination() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/rulesets"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [
                {
                    "id": "rs_page1",
                    "name": "Page 1 Ruleset",
                    "phase": "http_request_dynamic_redirect"
                }
            ],
            "result_info": { "cursors": { "after": "cursor_page2" } }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/rulesets"))
        .and(query_param("cursor", "cursor_page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [
                {
                    "id": "rs_page2",
                    "name": "Page 2 Ruleset",
                    "phase": "http_request_transform"
                }
            ],
            "result_info": { "cursors": {} }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let phases = &["http_request_dynamic_redirect", "http_request_transform"];
    let result = client.discover_rulesets("zone123", phases).await.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "rs_page1");
    assert_eq!(result[1].id, "rs_page2");
}

#[tokio::test]
async fn test_discover_rulesets_excludes_non_discoverable_phases() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/rulesets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [
                {
                    "id": "rs_managed_1",
                    "name": "Managed WAF",
                    "phase": "http_request_firewall_managed"
                },
                {
                    "id": "rs_managed_2",
                    "name": "Managed DDoS",
                    "phase": "ddos_l7"
                }
            ],
            "result_info": { "cursors": {} }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let phases = &[
        "http_request_dynamic_redirect",
        "http_request_transform",
        "http_request_firewall_custom",
    ];
    let result = client.discover_rulesets("zone123", phases).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_discover_rulesets_empty_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/rulesets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [],
            "result_info": { "cursors": {} }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let phases = &["http_request_dynamic_redirect"];
    let result = client.discover_rulesets("zone123", phases).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_verify_auth_valid_token() {
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
        CloudflareClient::with_base_url("invalid_token".to_string(), mock_server.uri()).unwrap();

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

#[tokio::test]
async fn test_lookup_zone_by_name_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .and(query_param("name", "example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{
                "id": "023e105f4ecef8ad9ca31a8372d0c353",
                "name": "example.com",
                "account": {
                    "id": "01a7362d577a6c3019a474fd6f485823",
                    "name": "Test Account"
                }
            }],
            "result_info": { "page": 1, "per_page": 20, "total_count": 1 }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.lookup_zone("example.com").await;
    assert!(result.is_ok());

    let zone_info = result.unwrap();
    assert_eq!(zone_info.zone_id, "023e105f4ecef8ad9ca31a8372d0c353");
    assert_eq!(zone_info.account_id, "01a7362d577a6c3019a474fd6f485823");
}

#[tokio::test]
async fn test_lookup_zone_by_id_success() {
    let mock_server = MockServer::start().await;
    let zone_id = "023e105f4ecef8ad9ca31a8372d0c353";

    Mock::given(method("GET"))
        .and(path(format!("/zones/{}", zone_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": {
                "id": zone_id,
                "name": "example.com",
                "account": {
                    "id": "01a7362d577a6c3019a474fd6f485823",
                    "name": "Test Account"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.lookup_zone(zone_id).await;
    assert!(result.is_ok());

    let zone_info = result.unwrap();
    assert_eq!(zone_info.zone_id, zone_id);
    assert_eq!(zone_info.account_id, "01a7362d577a6c3019a474fd6f485823");
}

#[tokio::test]
async fn test_lookup_zone_not_found_by_name() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .and(query_param("name", "nonexistent.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [],
            "result_info": { "page": 1, "per_page": 20, "total_count": 0 }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.lookup_zone("nonexistent.com").await;
    assert!(result.is_err());

    if let Err(CloudflareError::ZoneNotFound { zone }) = result {
        assert_eq!(zone, "nonexistent.com");
    } else {
        panic!("Expected CloudflareError::ZoneNotFound");
    }
}

#[tokio::test]
async fn test_lookup_zone_not_found_by_id() {
    let mock_server = MockServer::start().await;
    let zone_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    Mock::given(method("GET"))
        .and(path(format!("/zones/{}", zone_id)))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "success": false,
            "errors": [{ "code": 7003, "message": "Could not route to /zones/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, perhaps your object identifier is invalid?" }]
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.lookup_zone(zone_id).await;
    assert!(result.is_err());

    if let Err(CloudflareError::ZoneNotFound { zone }) = result {
        assert_eq!(zone, zone_id);
    } else {
        panic!("Expected CloudflareError::ZoneNotFound, got {:?}", result);
    }
}

#[tokio::test]
async fn test_lookup_zone_permission_denied() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .and(query_param("name", "restricted.com"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "success": false,
            "errors": [{ "code": 9109, "message": "You do not have permission to access this zone" }]
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.lookup_zone("restricted.com").await;
    assert!(result.is_err());

    if let Err(CloudflareError::ZoneLookupFailed { message }) = result {
        assert!(message.contains("permission"));
    } else {
        panic!("Expected CloudflareError::ZoneLookupFailed");
    }
}

#[tokio::test]
async fn test_fetch_all_pages_multiple_pages() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dns_records"))
        .and(query_param("page", "1"))
        .and(query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{"id": "r1"}, {"id": "r2"}],
            "result_info": { "page": 1, "per_page": 2, "total_count": 5 }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dns_records"))
        .and(query_param("page", "2"))
        .and(query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{"id": "r3"}, {"id": "r4"}],
            "result_info": { "page": 2, "per_page": 2, "total_count": 5 }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dns_records"))
        .and(query_param("page", "3"))
        .and(query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{"id": "r5"}],
            "result_info": { "page": 3, "per_page": 2, "total_count": 5 }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let results: Vec<String> = client
        .fetch_all_pages(
            &format!("{}/dns_records", mock_server.uri()),
            2,
            |json| async move {
                let items: Vec<serde_json::Value> =
                    serde_json::from_value(json).unwrap_or_default();
                Ok(items
                    .into_iter()
                    .map(|v| v["id"].as_str().unwrap().to_string())
                    .collect())
            },
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 5);
    assert_eq!(results, vec!["r1", "r2", "r3", "r4", "r5"]);
}

#[tokio::test]
async fn test_fetch_all_pages_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dns_records"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "success": false,
            "errors": [{ "code": 9109, "message": "Access denied" }]
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result: Result<Vec<String>, _> = client
        .fetch_all_pages(
            &format!("{}/dns_records", mock_server.uri()),
            10,
            |json| async move {
                let items: Vec<serde_json::Value> =
                    serde_json::from_value(json).unwrap_or_default();
                Ok(items
                    .into_iter()
                    .map(|v| v["id"].as_str().unwrap().to_string())
                    .collect())
            },
        )
        .await;

    assert!(result.is_err());
    if let Err(CloudflareError::Api { message, .. }) = result {
        assert!(message.contains("Access denied"));
    } else {
        panic!("Expected CloudflareError::Api");
    }
}

#[tokio::test]
async fn test_fetch_all_cursors_multiple_pages() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/rulesets"))
        .and(query_param("per_page", "2"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{"id": "rs1"}, {"id": "rs2"}],
            "result_info": { "cursors": { "after": "cursor_abc" } }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rulesets"))
        .and(query_param("per_page", "2"))
        .and(query_param("cursor", "cursor_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "result": [{"id": "rs3"}],
            "result_info": { "cursors": {} }
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let results: Vec<String> = client
        .fetch_all_cursors(
            &format!("{}/rulesets", mock_server.uri()),
            2,
            |json| async move {
                let items: Vec<serde_json::Value> =
                    serde_json::from_value(json).unwrap_or_default();
                Ok(items
                    .into_iter()
                    .map(|v| v["id"].as_str().unwrap().to_string())
                    .collect())
            },
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results, vec!["rs1", "rs2", "rs3"]);
}

#[tokio::test]
async fn test_fetch_all_cursors_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/rulesets"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "success": false,
            "errors": [{ "code": 5000, "message": "Internal server error" }]
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result: Result<Vec<String>, _> = client
        .fetch_all_cursors(
            &format!("{}/rulesets", mock_server.uri()),
            10,
            |json| async move {
                let items: Vec<serde_json::Value> =
                    serde_json::from_value(json).unwrap_or_default();
                Ok(items
                    .into_iter()
                    .map(|v| v["id"].as_str().unwrap().to_string())
                    .collect())
            },
        )
        .await;

    assert!(result.is_err());
    if let Err(CloudflareError::Api { message, .. }) = result {
        assert!(message.contains("Internal server error"));
    } else {
        panic!("Expected CloudflareError::Api");
    }
}

#[tokio::test]
async fn test_discover_page_rules_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/pagerules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "messages": [],
            "result": [
                {
                    "id": "rule_abc",
                    "status": "active",
                    "priority": 1,
                    "targets": [{
                        "target": "url",
                        "constraint": { "operator": "matches", "value": "*example.com/images/*" }
                    }],
                    "actions": [{ "id": "browser_check", "value": "on" }]
                },
                {
                    "id": "rule_def",
                    "status": "disabled",
                    "priority": 2,
                    "targets": [{
                        "target": "url",
                        "constraint": { "operator": "matches", "value": "*example.com/api/*" }
                    }],
                    "actions": [{ "id": "forwarding_url", "value": {"url": "https://api.example.com", "status_code": 301} }]
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.discover_page_rules("zone123").await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "rule_abc");
    assert_eq!(
        result[0].targets[0].constraint.value,
        "*example.com/images/*"
    );
    assert_eq!(result[1].id, "rule_def");
}

#[tokio::test]
async fn test_discover_page_rules_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/pagerules"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "success": false,
            "errors": [{ "code": 9109, "message": "Insufficient permissions" }],
            "messages": [],
            "result": null
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.discover_page_rules("zone123").await;
    assert!(result.is_err());

    if let Err(CloudflareError::Api { status, message }) = result {
        assert_eq!(status, 403);
        assert!(message.contains("Insufficient permissions"));
    } else {
        panic!("Expected CloudflareError::Api");
    }
}

#[tokio::test]
async fn test_discover_page_rules_empty_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/pagerules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errors": [],
            "messages": [],
            "result": []
        })))
        .mount(&mock_server)
        .await;

    let client =
        CloudflareClient::with_base_url("test_token".to_string(), mock_server.uri()).unwrap();

    let result = client.discover_page_rules("zone123").await.unwrap();
    assert!(result.is_empty());
}
