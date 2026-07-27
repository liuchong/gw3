use crate::api::Gw3Error;
use reqwest::{Client, StatusCode, Url};
use serde_json::Value;
use std::time::Duration;

const DEFAULT_WIKI_BASE_URL: &str = "https://wiki.guildwars2.com/api.php";

#[derive(Debug, Clone)]
pub struct WikiClient {
    base_url: String,
    http: Client,
}

impl WikiClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, Gw3Error> {
        let base_url = base_url.into();
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(Gw3Error::HttpClient)?;
        Ok(Self { base_url, http })
    }

    pub fn from_env() -> Result<Self, Gw3Error> {
        Self::new(
            std::env::var("GW3_WIKI_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_WIKI_BASE_URL.to_string()),
        )
    }

    pub async fn search(&self, query: &str) -> Result<Value, Gw3Error> {
        let mut url = self.base_url()?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("action", "query");
            pairs.append_pair("list", "search");
            pairs.append_pair("srsearch", query);
            pairs.append_pair("format", "json");
            pairs.append_pair("srlimit", "10");
        }
        self.get_json(url).await
    }

    pub async fn page(&self, title: &str) -> Result<Value, Gw3Error> {
        let mut url = self.base_url()?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("action", "query");
            pairs.append_pair("prop", "extracts|info");
            pairs.append_pair("exintro", "1");
            pairs.append_pair("explaintext", "1");
            pairs.append_pair("inprop", "url");
            pairs.append_pair("titles", title);
            pairs.append_pair("format", "json");
        }
        self.get_json(url).await
    }

    async fn get_json(&self, url: Url) -> Result<Value, Gw3Error> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(Gw3Error::Transport)?;
        let status = response.status();
        let body = response.text().await.map_err(Gw3Error::Transport)?;

        match status {
            StatusCode::OK => serde_json::from_str(&body).map_err(Gw3Error::Json),
            StatusCode::TOO_MANY_REQUESTS => Err(Gw3Error::RateLimited { body }),
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

    fn base_url(&self) -> Result<Url, Gw3Error> {
        Url::parse(&self.base_url).map_err(|source| Gw3Error::InvalidUrl {
            url: self.base_url.clone(),
            source,
        })
    }
}
