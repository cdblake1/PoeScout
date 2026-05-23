pub mod cache;
pub mod ninja;

use cache::PriceCache;
use ninja::NinjaClient;
use poe_core::types::PriceRecord;
use tokio::sync::RwLock;

pub struct PricingEngine {
    client: NinjaClient,
    cache: RwLock<PriceCache>,
}

impl PricingEngine {
    pub fn new() -> Self {
        Self {
            client: NinjaClient::new(),
            cache: RwLock::new(PriceCache::new(300)),
        }
    }

    pub async fn ensure_fresh(&self, league: &str) -> Result<(), String> {
        if !self.cache.read().await.is_stale() {
            return Ok(());
        }
        self.refresh(league).await
    }

    pub async fn refresh(&self, league: &str) -> Result<(), String> {
        tracing::info!("Refreshing poe.ninja prices for league {}", league);
        let records = self.client.fetch_all_prices(league).await?;
        let count = records.len();
        self.cache.write().await.update(records);
        tracing::info!("Cached {} price records", count);
        Ok(())
    }

    pub async fn get_price(&self, item_name: &str) -> Option<PriceRecord> {
        self.cache.read().await.get_price(item_name).cloned()
    }

    pub async fn divine_ratio(&self) -> f64 {
        self.cache.read().await.divine_ratio()
    }

    pub async fn is_stale(&self) -> bool {
        self.cache.read().await.is_stale()
    }

    pub async fn get_current_league(&self) -> Result<String, String> {
        self.client.fetch_current_league().await
    }

    pub async fn get_all_leagues(&self) -> Result<Vec<String>, String> {
        self.client.fetch_all_leagues().await
    }
}
