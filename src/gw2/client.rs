use super::{
    ApiRequest, PublicEndpointKind, excluded_endpoint, normalize_api_path, public_endpoint,
    public_endpoints, validate_lang,
};
use crate::config::{
    ClientConfig, DEFAULT_API_BASE_URL, DEFAULT_USER_AGENT, DEFAULT_WIKI_BASE_URL,
};
use crate::error::Gw3Error;
use reqwest::{Client, StatusCode, Url};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

const MAX_IDS_PER_REQUEST: usize = 200;

#[derive(Debug, Clone)]
pub struct Gw2Client {
    config: ClientConfig,
    http: Client,
}

impl Gw2Client {
    pub fn new(config: ClientConfig) -> Result<Self, Gw3Error> {
        validate_lang(config.lang.as_deref())?;
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(DEFAULT_USER_AGENT)
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

    pub async fn public_routes(&self) -> Result<Value, Gw3Error> {
        serde_json::to_value(public_endpoints()).map_err(Gw3Error::Json)
    }

    pub async fn public_list(
        &self,
        key: &str,
        lang: Option<String>,
        schema_version: Option<String>,
    ) -> Result<Value, Gw3Error> {
        let endpoint = self.resolve_public_endpoint(key)?;
        if endpoint.kind != PublicEndpointKind::Collection {
            return Err(Gw3Error::UnsupportedPublicOperation {
                key: key.to_string(),
                operation: "list".to_string(),
            });
        }
        self.get_json(
            ApiRequest::new(endpoint.path)
                .with_lang(lang)
                .with_schema_version(schema_version),
        )
        .await
    }

    pub async fn public_get(
        &self,
        key: &str,
        id: Option<String>,
        ids: Vec<String>,
        lang: Option<String>,
        schema_version: Option<String>,
    ) -> Result<Value, Gw3Error> {
        let endpoint = self.resolve_public_endpoint(key)?;
        match endpoint.kind {
            PublicEndpointKind::Collection => {
                self.get_json(
                    ApiRequest::new(endpoint.path)
                        .with_id_opt(id)
                        .with_ids(ids)
                        .with_lang(lang)
                        .with_schema_version(schema_version),
                )
                .await
            }
            PublicEndpointKind::Singleton => {
                if id.is_some() || !ids.is_empty() {
                    return Err(Gw3Error::UnsupportedPublicOperation {
                        key: key.to_string(),
                        operation: "get with id or ids".to_string(),
                    });
                }
                self.get_json(
                    ApiRequest::new(endpoint.path)
                        .with_lang(lang)
                        .with_schema_version(schema_version),
                )
                .await
            }
            PublicEndpointKind::CallOnly => Err(Gw3Error::UnsupportedPublicOperation {
                key: key.to_string(),
                operation: "get".to_string(),
            }),
        }
    }

    pub async fn public_all(
        &self,
        key: &str,
        lang: Option<String>,
        schema_version: Option<String>,
    ) -> Result<Value, Gw3Error> {
        let endpoint = self.resolve_public_endpoint(key)?;
        if endpoint.kind != PublicEndpointKind::Collection {
            return Err(Gw3Error::UnsupportedPublicOperation {
                key: key.to_string(),
                operation: "all".to_string(),
            });
        }
        self.get_json(
            ApiRequest::new(endpoint.path)
                .with_query_param("ids", "all")
                .with_lang(lang)
                .with_schema_version(schema_version),
        )
        .await
    }

    pub async fn public_page(
        &self,
        key: &str,
        page: u32,
        page_size: Option<u32>,
        lang: Option<String>,
        schema_version: Option<String>,
    ) -> Result<Value, Gw3Error> {
        let endpoint = self.resolve_public_endpoint(key)?;
        if endpoint.kind != PublicEndpointKind::Collection {
            return Err(Gw3Error::UnsupportedPublicOperation {
                key: key.to_string(),
                operation: "page".to_string(),
            });
        }
        self.get_json(
            ApiRequest::new(endpoint.path)
                .with_lang(lang)
                .with_schema_version(schema_version)
                .with_page(Some(page), page_size),
        )
        .await
    }

    pub async fn public_call<I, K, V, J, Q, R>(
        &self,
        key: &str,
        path_params: I,
        query: J,
        lang: Option<String>,
        schema_version: Option<String>,
    ) -> Result<Value, Gw3Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
        J: IntoIterator<Item = (Q, R)>,
        Q: Into<String>,
        R: Into<String>,
    {
        let endpoint = self.resolve_public_endpoint(key)?;
        let path_params = path_params
            .into_iter()
            .map(|(name, value)| (name.into(), value.into()))
            .collect::<BTreeMap<String, String>>();
        let rendered_path =
            self.render_public_path(key, endpoint.path, endpoint.path_params, &path_params)?;
        let request = ApiRequest::new(rendered_path)
            .with_lang(lang)
            .with_schema_version(schema_version)
            .with_query_params(query);
        self.get_json(request).await
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

    fn resolve_public_endpoint(
        &self,
        key: &str,
    ) -> Result<&'static super::PublicEndpointSpec, Gw3Error> {
        if let Some(endpoint) = public_endpoint(key) {
            return Ok(endpoint);
        }
        if let Some(endpoint) = excluded_endpoint(key) {
            if endpoint.auth {
                return Err(Gw3Error::PublicEndpointRequiresAuth {
                    key: key.to_string(),
                    path: endpoint.path.to_string(),
                });
            }
            if !endpoint.active {
                return Err(Gw3Error::InactivePublicEndpoint {
                    key: key.to_string(),
                    path: endpoint.path.to_string(),
                });
            }
        }
        Err(Gw3Error::UnknownPublicEndpoint(key.to_string()))
    }

    fn render_public_path(
        &self,
        key: &str,
        path_template: &str,
        expected_params: &[&str],
        provided_params: &BTreeMap<String, String>,
    ) -> Result<String, Gw3Error> {
        for expected in expected_params {
            if !provided_params.contains_key(*expected) {
                return Err(Gw3Error::MissingPathParameter {
                    key: key.to_string(),
                    name: (*expected).to_string(),
                });
            }
        }

        for provided in provided_params.keys() {
            if !expected_params
                .iter()
                .any(|expected| expected == &provided.as_str())
            {
                return Err(Gw3Error::UnexpectedPathParameter {
                    key: key.to_string(),
                    name: provided.clone(),
                });
            }
        }

        let mut path = path_template.to_string();
        for expected in expected_params {
            let placeholder = format!(":{expected}");
            if let Some(value) = provided_params.get(*expected) {
                path = path.replace(&placeholder, value);
            }
        }
        Ok(path)
    }
}

trait ApiRequestExt {
    fn with_id_opt(self, id: Option<String>) -> Self;
}

impl ApiRequestExt for ApiRequest {
    fn with_id_opt(mut self, id: Option<String>) -> Self {
        if let Some(id) = id {
            self = self.with_id(id);
        }
        self
    }
}
