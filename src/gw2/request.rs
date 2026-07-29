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

    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((name.into(), value.into()));
        self
    }

    pub fn with_query_params<I, K, V>(mut self, query: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.query = query
            .into_iter()
            .map(|(name, value)| (name.into(), value.into()))
            .collect();
        self
    }
}
