use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

const DEFAULT_API_BASE_URL: &str = "https://api.guildwars2.com";
const DEFAULT_WIKI_BASE_URL: &str = "https://wiki.guildwars2.com/api.php";
const MAX_IDS_PER_REQUEST: usize = 200;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub wiki_base_url: String,
    pub lang: Option<String>,
    pub schema_version: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_API_BASE_URL.to_string(),
            wiki_base_url: DEFAULT_WIKI_BASE_URL.to_string(),
            lang: None,
            schema_version: None,
            api_key: None,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiRequest {
    pub path: String,
    pub id: Option<String>,
    pub ids: Vec<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub lang: Option<String>,
    pub schema_version: Option<String>,
    pub query: Vec<(String, String)>,
    pub requires_auth: bool,
}

impl ApiRequest {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }

    pub fn requires_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_ids<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        self.ids = ids.into_iter().map(|id| id.to_string()).collect();
        self
    }

    pub fn with_lang(mut self, lang: Option<String>) -> Self {
        self.lang = lang;
        self
    }

    pub fn with_schema_version(mut self, schema_version: Option<String>) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn with_page(mut self, page: Option<u32>, page_size: Option<u32>) -> Self {
        self.page = page;
        self.page_size = page_size;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    config: ClientConfig,
    http: Client,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self, Gw3Error> {
        validate_lang(config.lang.as_deref())?;
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(Gw3Error::HttpClient)?;
        Ok(Self { config, http })
    }

    pub fn from_env() -> Result<Self, Gw3Error> {
        Self::new(ClientConfig {
            base_url: std::env::var("GW3_API_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string()),
            wiki_base_url: std::env::var("GW3_WIKI_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_WIKI_BASE_URL.to_string()),
            api_key: std::env::var("GW2_API_KEY").ok(),
            ..ClientConfig::default()
        })
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    pub async fn routes(&self) -> Result<Value, Gw3Error> {
        self.get_json(ApiRequest::new("/v2.json").with_schema_version(Some("latest".to_string())))
            .await
    }

    pub async fn token_info(&self) -> Result<Value, Gw3Error> {
        self.get_json(ApiRequest::new("/v2/tokeninfo").requires_auth())
            .await
    }

    pub async fn item_lookup<I, S>(&self, ids: I, lang: Option<String>) -> Result<Value, Gw3Error>
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        self.get_json(ApiRequest::new("/v2/items").with_ids(ids).with_lang(lang))
            .await
    }

    pub async fn item_prices<I, S>(&self, ids: I) -> Result<Value, Gw3Error>
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        self.get_json(ApiRequest::new("/v2/commerce/prices").with_ids(ids))
            .await
    }

    pub async fn account_info(&self) -> Result<Value, Gw3Error> {
        self.get_json(ApiRequest::new("/v2/account").requires_auth())
            .await
    }

    pub async fn character_list(&self) -> Result<Value, Gw3Error> {
        self.get_json(ApiRequest::new("/v2/characters").requires_auth())
            .await
    }

    pub async fn get_json(&self, request: ApiRequest) -> Result<Value, Gw3Error> {
        if request.requires_auth && self.config.api_key.is_none() {
            return Err(Gw3Error::MissingApiKey);
        }

        if request.ids.len() > MAX_IDS_PER_REQUEST {
            return self.get_chunked_json(request).await;
        }

        self.get_single_json(&request, request.ids.as_slice()).await
    }

    async fn get_chunked_json(&self, request: ApiRequest) -> Result<Value, Gw3Error> {
        let mut merged = Vec::new();
        for chunk in request.ids.chunks(MAX_IDS_PER_REQUEST) {
            let value = self.get_single_json(&request, chunk).await?;
            match value {
                Value::Array(items) => merged.extend(items),
                other => return Err(Gw3Error::UnexpectedResponse(other)),
            }
        }
        Ok(Value::Array(merged))
    }

    async fn get_single_json(
        &self,
        request: &ApiRequest,
        ids: &[String],
    ) -> Result<Value, Gw3Error> {
        validate_lang(request.lang.as_deref())?;
        let url = self.build_url(request, ids)?;
        let mut http_request = self.http.get(url);

        if request.requires_auth
            && let Some(api_key) = &self.config.api_key
        {
            http_request = http_request.bearer_auth(api_key);
        }

        let response = http_request.send().await.map_err(Gw3Error::Transport)?;
        let status = response.status();
        let body = response.text().await.map_err(Gw3Error::Transport)?;

        match status {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                serde_json::from_str(&body).map_err(Gw3Error::Json)
            }
            StatusCode::FORBIDDEN => Err(Gw3Error::Forbidden { body }),
            StatusCode::NOT_FOUND => Err(Gw3Error::NotFound { body }),
            StatusCode::TOO_MANY_REQUESTS => Err(Gw3Error::RateLimited { body }),
            StatusCode::SERVICE_UNAVAILABLE => Err(Gw3Error::DisabledEndpoint { body }),
            status if status.is_server_error() => Err(Gw3Error::Upstream {
                status: status.as_u16(),
                body,
            }),
            status => Err(Gw3Error::UnexpectedStatus {
                status: status.as_u16(),
                body,
            }),
        }
    }

    fn build_url(&self, request: &ApiRequest, ids: &[String]) -> Result<Url, Gw3Error> {
        let path = normalize_api_path(&request.path)?;
        let mut url = Url::parse(&format!(
            "{}{}",
            self.config.base_url.trim_end_matches('/'),
            path
        ))
        .map_err(|source| Gw3Error::InvalidUrl {
            url: format!("{}{}", self.config.base_url.trim_end_matches('/'), path),
            source,
        })?;

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(id) = &request.id {
                pairs.append_pair("id", id);
            }
            if !ids.is_empty() {
                pairs.append_pair("ids", &ids.join(","));
            }
            if let Some(page) = request.page {
                pairs.append_pair("page", &page.to_string());
            }
            if let Some(page_size) = request.page_size {
                pairs.append_pair("page_size", &page_size.to_string());
            }
            let lang = request.lang.as_ref().or(self.config.lang.as_ref());
            if let Some(lang) = lang {
                pairs.append_pair("lang", lang);
            }
            let schema_version = request
                .schema_version
                .as_ref()
                .or(self.config.schema_version.as_ref());
            if let Some(schema_version) = schema_version {
                pairs.append_pair("v", schema_version);
            }
            for (name, value) in &request.query {
                pairs.append_pair(name, value);
            }
        }

        Ok(url)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSpec {
    pub path: String,
    pub lang: bool,
    pub auth: bool,
    pub active: bool,
}

#[derive(Debug, Error)]
pub enum Gw3Error {
    #[error("invalid language `{0}`; expected one of en, es, de, fr, zh")]
    InvalidLanguage(String),
    #[error("GW2_API_KEY or --api-key is required for this command")]
    MissingApiKey,
    #[error("invalid API path `{0}`; expected /v2/...")]
    InvalidPath(String),
    #[error("invalid URL `{url}`: {source}")]
    InvalidUrl {
        url: String,
        source: url::ParseError,
    },
    #[error("failed to build HTTP client: {0}")]
    HttpClient(reqwest::Error),
    #[error("transport error: {0}")]
    Transport(reqwest::Error),
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
    #[error("forbidden by official API: {body}")]
    Forbidden { body: String },
    #[error("official API resource was not found: {body}")]
    NotFound { body: String },
    #[error("official API rate limit exceeded: {body}")]
    RateLimited { body: String },
    #[error("official API endpoint is disabled: {body}")]
    DisabledEndpoint { body: String },
    #[error("official API upstream failure {status}: {body}")]
    Upstream { status: u16, body: String },
    #[error("unexpected HTTP status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(Value),
}

pub fn normalize_api_path(path: &str) -> Result<String, Gw3Error> {
    let trimmed = path.trim();
    if trimmed == "/v2.json" || trimmed == "v2.json" {
        return Ok("/v2.json".to_string());
    }

    let path = trimmed.strip_prefix('/').unwrap_or(trimmed);
    if path == "v2" || path.starts_with("v2/") {
        Ok(format!("/{path}"))
    } else if !path.is_empty() {
        Ok(format!("/v2/{path}"))
    } else {
        Err(Gw3Error::InvalidPath(trimmed.to_string()))
    }
}

fn validate_lang(lang: Option<&str>) -> Result<(), Gw3Error> {
    match lang {
        Some("en" | "es" | "de" | "fr" | "zh") | None => Ok(()),
        Some(other) => Err(Gw3Error::InvalidLanguage(other.to_string())),
    }
}
