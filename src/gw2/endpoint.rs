use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSpec {
    pub path: String,
    pub lang: bool,
    pub auth: bool,
    pub active: bool,
}
