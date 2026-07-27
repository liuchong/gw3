use crate::error::Gw3Error;

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

pub(crate) fn validate_lang(lang: Option<&str>) -> Result<(), Gw3Error> {
    match lang {
        Some("en" | "es" | "de" | "fr" | "zh") | None => Ok(()),
        Some(other) => Err(Gw3Error::InvalidLanguage(other.to_string())),
    }
}
