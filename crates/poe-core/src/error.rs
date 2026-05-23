use thiserror::Error;

#[derive(Debug, Error)]
pub enum PoeError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Data ingestion error: {0}")]
    Ingestion(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Rate limited — retry after {0}s")]
    RateLimited(u64),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("API error: {0}")]
    Api(String),
}

impl serde::Serialize for PoeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
