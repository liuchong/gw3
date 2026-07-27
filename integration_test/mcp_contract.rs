use gw3::api::ClientConfig;
use gw3::mcp::{Gw3McpServer, IdsParams};
use httpmock::Method::GET;
use httpmock::MockServer;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

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
