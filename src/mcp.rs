use crate::api::{ApiClient, ApiRequest, ClientConfig, Gw3Error};
use crate::wiki::WikiClient;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Gw3McpServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    api_client: ApiClient,
    wiki_client: WikiClient,
}

impl Gw3McpServer {
    pub fn from_env() -> Result<Self, Gw3Error> {
        let config = ClientConfig {
            base_url: std::env::var("GW3_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.guildwars2.com".to_string()),
            wiki_base_url: std::env::var("GW3_WIKI_BASE_URL")
                .unwrap_or_else(|_| "https://wiki.guildwars2.com/api.php".to_string()),
            api_key: std::env::var("GW2_API_KEY").ok(),
            ..ClientConfig::default()
        };
        Self::new(config)
    }

    pub fn new(config: ClientConfig) -> Result<Self, Gw3Error> {
        Ok(Self {
            tool_router: Self::tool_router(),
            wiki_client: WikiClient::new(config.wiki_base_url.clone())?,
            api_client: ApiClient::new(config)?,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApiRequestParams {
    pub path: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub requires_auth: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdsParams {
    pub ids: Vec<String>,
    #[serde(default)]
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikiQueryParams {
    pub query: String,
}

#[tool_router]
impl Gw3McpServer {
    #[tool(
        name = "gw2_api_request",
        description = "Call a Guild Wars 2 official API v2 endpoint and return JSON"
    )]
    pub async fn gw2_api_request(
        &self,
        Parameters(params): Parameters<ApiRequestParams>,
    ) -> Result<String, String> {
        let mut request = ApiRequest::new(params.path)
            .with_ids(params.ids)
            .with_lang(params.lang)
            .with_schema_version(params.schema_version)
            .with_page(params.page, params.page_size);
        if let Some(id) = params.id {
            request = request.with_id(id);
        }
        if params.requires_auth {
            request = request.requires_auth();
        }
        self.stringify(self.api_client.get_json(request).await)
    }

    #[tool(
        name = "gw2_token_info",
        description = "Inspect the configured GW2 API key"
    )]
    pub async fn gw2_token_info(&self) -> Result<String, String> {
        self.stringify(self.api_client.token_info().await)
    }

    #[tool(
        name = "gw2_item_lookup",
        description = "Look up item details by item IDs"
    )]
    pub async fn gw2_item_lookup(
        &self,
        Parameters(params): Parameters<IdsParams>,
    ) -> Result<String, String> {
        self.stringify(self.api_client.item_lookup(params.ids, params.lang).await)
    }

    #[tool(
        name = "gw2_item_prices",
        description = "Look up trading post prices by item IDs"
    )]
    pub async fn gw2_item_prices(
        &self,
        Parameters(params): Parameters<IdsParams>,
    ) -> Result<String, String> {
        self.stringify(self.api_client.item_prices(params.ids).await)
    }

    #[tool(
        name = "gw2_account_summary",
        description = "Read account summary for the configured key"
    )]
    pub async fn gw2_account_summary(&self) -> Result<String, String> {
        self.stringify(self.api_client.account_info().await)
    }

    #[tool(
        name = "gw2_character_list",
        description = "List characters for the configured key"
    )]
    pub async fn gw2_character_list(&self) -> Result<String, String> {
        self.stringify(self.api_client.character_list().await)
    }

    #[tool(
        name = "gw2_wiki_search",
        description = "Search Guild Wars 2 Wiki pages"
    )]
    pub async fn gw2_wiki_search(
        &self,
        Parameters(params): Parameters<WikiQueryParams>,
    ) -> Result<String, String> {
        self.stringify(self.wiki_client.search(&params.query).await)
    }

    #[tool(
        name = "gw2_wiki_page",
        description = "Read a Guild Wars 2 Wiki page summary"
    )]
    pub async fn gw2_wiki_page(
        &self,
        Parameters(params): Parameters<WikiQueryParams>,
    ) -> Result<String, String> {
        self.stringify(self.wiki_client.page(&params.query).await)
    }

    fn stringify(&self, result: Result<Value, Gw3Error>) -> Result<String, String> {
        result
            .and_then(|value| serde_json::to_string_pretty(&value).map_err(Gw3Error::Json))
            .map_err(|error| error.to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Gw3McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }
}

pub async fn serve_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let service = Gw3McpServer::from_env()?
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
