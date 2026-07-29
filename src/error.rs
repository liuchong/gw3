use serde_json::Value;
use thiserror::Error;

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
    #[error("unknown public endpoint key `{0}`")]
    UnknownPublicEndpoint(String),
    #[error("public endpoint `{key}` requires authentication and is not exposed in `gw3 public`")]
    PublicEndpointRequiresAuth { key: String, path: String },
    #[error("public endpoint `{key}` is currently inactive: {path}")]
    InactivePublicEndpoint { key: String, path: String },
    #[error("public endpoint `{key}` does not support operation `{operation}`")]
    UnsupportedPublicOperation { key: String, operation: String },
    #[error("public endpoint `{key}` is missing required path parameter `{name}`")]
    MissingPathParameter { key: String, name: String },
    #[error("public endpoint `{key}` received unexpected path parameter `{name}`")]
    UnexpectedPathParameter { key: String, name: String },
    #[error("invalid name=value pair `{0}`")]
    InvalidNameValuePair(String),
    #[error("official API upstream failure {status}: {body}")]
    Upstream { status: u16, body: String },
    #[error("unexpected HTTP status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(Value),
    #[error("runtime error: {0}")]
    Runtime(String),
}
