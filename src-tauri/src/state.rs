use anyhow::Result;
use poe_data::DataEngine;
use std::path::Path;
use std::sync::Arc;

pub struct AppState {
    pub engine: Arc<DataEngine>,
}

impl AppState {
    pub async fn init(data_dir: &Path) -> Result<Self> {
        let db_path = data_dir.join("poescout.db");
        let repoe_dir = data_dir.join("data");

        let engine = DataEngine::new(&db_path, &repoe_dir).await?;

        Ok(Self {
            engine: Arc::new(engine),
        })
    }
}
