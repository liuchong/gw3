use schemars::JsonSchema;
use serde::Deserialize;

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
