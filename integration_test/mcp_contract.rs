use gw3::api::ClientConfig;
use gw3::mcp::{Gw3McpServer, IdsParams, PublicCallParams, PublicGetParams};
use httpmock::Method::GET;
use httpmock::MockServer;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use std::collections::BTreeMap;

#[tokio::test]
async fn mcp_item_prices_uses_shared_api_client() {
    let server = MockServer::start();
    let _prices = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/commerce/prices")
            .query_param("ids", "19684");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 19684, "buys": { "unit_price": 10 } }]));
    });

    let mcp = Gw3McpServer::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("mcp server should be constructed");

    let output = mcp
        .gw2_item_prices(Parameters(IdsParams {
            ids: vec!["19684".to_string()],
            lang: None,
        }))
        .await
        .expect("mcp tool should return JSON string");

    assert!(output.contains("19684"));
    assert!(output.contains("unit_price"));
}

#[tokio::test]
async fn mcp_public_get_uses_shared_api_client() {
    let server = MockServer::start();
    let _skins = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/skins")
            .query_param("ids", "4674")
            .query_param("lang", "zh");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 4674, "name": "Foefire Wraps" }]));
    });

    let mcp = Gw3McpServer::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("mcp server should be constructed");

    let output = mcp
        .gw2_public_get(Parameters(PublicGetParams {
            key: "skins".to_string(),
            id: None,
            ids: vec!["4674".to_string()],
            lang: Some("zh".to_string()),
            schema_version: None,
        }))
        .await
        .expect("mcp public tool should return JSON string");

    assert!(output.contains("Foefire Wraps"));
}

#[tokio::test]
async fn mcp_public_call_supports_path_and_query_parameters() {
    let server = MockServer::start();
    let _leaderboard = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/pvp/seasons/season-1/leaderboards/ladder/eu")
            .query_param("foo", "bar");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({ "top": [] }));
    });

    let mcp = Gw3McpServer::new(ClientConfig {
        base_url: server.base_url(),
        ..ClientConfig::default()
    })
    .expect("mcp server should be constructed");

    let output = mcp
        .gw2_public_call(Parameters(PublicCallParams {
            key: "pvp_season_leaderboard_entries".to_string(),
            path_params: BTreeMap::from([
                ("id".to_string(), "season-1".to_string()),
                ("board".to_string(), "ladder".to_string()),
                ("region".to_string(), "eu".to_string()),
            ]),
            query: BTreeMap::from([("foo".to_string(), "bar".to_string())]),
            lang: None,
            schema_version: None,
        }))
        .await
        .expect("mcp public call should succeed");

    assert!(output.contains("\"top\""));
}
