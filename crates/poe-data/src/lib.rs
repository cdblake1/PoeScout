pub mod db;
pub mod index;
pub mod ingest;
pub mod models;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DataEngine {
    pub db: db::Database,
    pub index: Arc<RwLock<index::MemIndex>>,
}

impl DataEngine {
    pub async fn new(db_path: &Path, data_dir: &Path) -> Result<Self> {
        let db = db::Database::open(db_path)?;
        db.migrate()?;

        let index = Arc::new(RwLock::new(index::MemIndex::new()));

        let engine = Self { db, index };

        // Check if data needs ingestion
        if engine.db.is_empty()? {
            tracing::info!("No data found, starting ingestion...");
            engine.ingest_all(data_dir).await?;
        }

        // Build in-memory indexes
        engine.build_indexes().await?;

        Ok(engine)
    }

    async fn ingest_all(&self, data_dir: &Path) -> Result<()> {
        ingest::download_repoe_data(data_dir).await?;
        ingest::ingest_mods(&self.db, data_dir)?;
        ingest::ingest_base_items(&self.db, data_dir)?;
        Ok(())
    }

    async fn build_indexes(&self) -> Result<()> {
        let mods = self.db.load_all_mods()?;
        let mut idx = self.index.write().await;
        idx.build_from_mods(&mods);
        tracing::info!("Built in-memory index: {} mods", mods.len());
        Ok(())
    }
}
