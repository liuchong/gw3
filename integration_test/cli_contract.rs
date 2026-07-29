use assert_cmd::Command;
use gw3::config::DEFAULT_USER_AGENT;
use httpmock::Method::GET;
use httpmock::MockServer;
use predicates::prelude::*;
use serde_json::json;

#[test]
fn manifest_declares_only_one_binary_entrypoint() {
    let manifest = include_str!("../Cargo.toml");

    assert_eq!(manifest.matches("[[bin]]").count(), 1);
    assert!(manifest.contains("name = \"gw3\""));
    assert!(!manifest.contains("name = \"gw3-mcp\""));
}

#[test]
fn cli_mcp_help_exposes_stdio_server_mode() {
    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");

    cmd.args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"));
}

#[test]
fn cli_item_get_outputs_public_item_json() {
    let server = MockServer::start();
    let _items = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/items")
            .query_param("ids", "19684");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([{ "id": 19684, "name": "Copper Ore" }]));
    });

    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");
    cmd.env("GW3_API_BASE_URL", server.base_url())
        .args(["item", "get", "--ids", "19684"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Copper Ore"));
}

#[test]
fn cli_account_info_requires_key_without_http_request() {
    let server = MockServer::start();
    let account = server.mock(|when, then| {
        when.method(GET).path("/v2/account");
        then.status(200)
            .json_body(json!({ "name": "ShouldNotBeCalled.0000" }));
    });

    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");
    cmd.env("GW3_API_BASE_URL", server.base_url())
        .env_remove("GW2_API_KEY")
        .args(["account", "info"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("GW2_API_KEY"));

    account.assert_calls(0);
}

#[test]
fn cli_wiki_search_outputs_page_title() {
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

    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");
    cmd.env("GW3_WIKI_BASE_URL", server.url("/"))
        .args(["wiki", "search", "Legendary armor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Legendary armor"));
}

#[test]
fn cli_public_routes_lists_public_registry_without_account_endpoint() {
    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");

    cmd.args(["public", "routes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/v2/skins"))
        .stdout(predicate::str::contains("\"path\": \"/v2/account\"").not());
}

#[test]
fn cli_public_get_hits_registry_backed_endpoint() {
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

    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");
    cmd.env("GW3_API_BASE_URL", server.base_url())
        .args(["public", "get", "skins", "--ids", "4674", "--lang", "zh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Foefire Wraps"));
}

#[test]
fn cli_public_call_supports_query_pairs() {
    let server = MockServer::start();
    let _search = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/recipes/search")
            .query_param("input", "46747");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([12051, 12052]));
    });

    let mut cmd = Command::cargo_bin("gw3").expect("gw3 binary should build");
    cmd.env("GW3_API_BASE_URL", server.base_url())
        .args(["public", "call", "recipes_search", "--query", "input=46747"])
        .assert()
        .success()
        .stdout(predicate::str::contains("12051"));
}
