use gw3::api::{ApiClient, ApiRequest, ClientConfig, Gw3Error};
use gw3::config::DEFAULT_USER_AGENT;
use gw3::wiki::WikiClient;
use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

#[tokio::test]
async fn public_item_lookup_returns_json() {
    let server = MockServer::start();
    let _items = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/items")
            .query_param("ids", "19684")
            .query_param("lang", "zh");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 19684, "name": "Copper Ore" }]));
    });

    let client = ApiClient::new(ClientConfig {
        base_url: server.base_url(),
        lang: Some("zh".to_string()),
        ..ClientConfig::default()
    })
    .expect("client config should be valid");

    let value = client
        .get_json(ApiRequest::new("/v2/items").with_ids(["19684"]))
        .await
        .expect("public request should succeed");

    assert_eq!(value[0]["id"], 19684);
}

#[tokio::test]
async fn authenticated_request_requires_key_before_http() {
    let server = MockServer::start();
    let account = server.mock(|when, then| {
        when.method(GET).path("/v2/account");
        then.status(200)
            .json_body(json!({ "name": "ShouldNotBeCalled.0000" }));
    });
    let client = ApiClient::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("client config should be valid");

    let error = client
        .get_json(ApiRequest::new("/v2/account").requires_auth())
        .await
        .expect_err("missing key should fail locally");

    assert!(matches!(error, Gw3Error::MissingApiKey));
    account.assert_calls(0);
}

#[tokio::test]
async fn authenticated_request_uses_authorization_header() {
    let server = MockServer::start();
    let _account = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/account")
            .header("authorization", "Bearer test-key");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({ "name": "Example.1234" }));
    });

    let client = ApiClient::new(ClientConfig {
        base_url: server.base_url(),
        api_key: Some("test-key".to_string()),
        ..ClientConfig::default()
    })
    .expect("client config should be valid");

    let value = client
        .get_json(ApiRequest::new("/v2/account").requires_auth())
        .await
        .expect("authenticated request should succeed");

    assert_eq!(value["name"], "Example.1234");
}

#[tokio::test]
async fn id_lists_are_chunked_to_two_hundred() {
    let server = MockServer::start();
    let ids: Vec<String> = (1..=201).map(|id| id.to_string()).collect();
    let first_chunk = ids[..200].join(",");
    let second_chunk = ids[200..].join(",");

    let _first = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/items")
            .query_param("ids", first_chunk.as_str());
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 1 }]));
    });
    let _second = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/items")
            .query_param("ids", second_chunk.as_str());
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 201 }]));
    });

    let client = ApiClient::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("client config should be valid");

    let value = client
        .get_json(ApiRequest::new("/v2/items").with_ids(ids))
        .await
        .expect("chunked request should succeed");

    assert_eq!(value.as_array().unwrap().len(), 2);
    assert_eq!(value[1]["id"], 201);
}

#[tokio::test]
async fn rate_limit_is_reported_as_stable_error() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/v2/items");
        then.status(429)
            .header("content-type", "application/json")
            .json_body(json!({ "text": "too many requests" }));
    });

    let client = ApiClient::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("client config should be valid");

    let error = client
        .get_json(ApiRequest::new("/v2/items"))
        .await
        .expect_err("429 should be normalized");

    assert!(matches!(error, Gw3Error::RateLimited { .. }));
}

#[tokio::test]
async fn wiki_search_returns_titles() {
    let server = MockServer::start();
    let _search = server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .header("user-agent", DEFAULT_USER_AGENT)
            .query_param("action", "query")
            .query_param("list", "search")
            .query_param("srsearch", "Legendary armor");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "query": {
                    "search": [
                        { "title": "Legendary armor", "pageid": 167995 }
                    ]
                }
            }));
    });

    let client = WikiClient::new(server.url("/")).expect("wiki client should be valid");
    let value = client
        .search("Legendary armor")
        .await
        .expect("wiki search should succeed");

    assert_eq!(value["query"]["search"][0]["title"], "Legendary armor");
}
