pub const DEFAULT_API_BASE_URL: &str = "https://api.guildwars2.com";
pub const DEFAULT_WIKI_BASE_URL: &str = "https://wiki.guildwars2.com/api.php";

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
