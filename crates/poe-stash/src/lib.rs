pub mod client;
pub mod matcher;

use client::StashClient;
use matcher::price_item;
use poe_core::types::*;
use poe_pricing::PricingEngine;
use std::sync::Arc;

pub struct StashTracker {
    client: StashClient,
    pricing: Arc<PricingEngine>,
    snapshots: Vec<SnapshotRecord>,
    cached_tabs: Option<Vec<StashTab>>,
}

struct SnapshotRecord {
    timestamp: chrono::DateTime<chrono::Utc>,
    total_chaos: f64,
}

impl StashTracker {
    pub fn new(pricing: Arc<PricingEngine>) -> Self {
        Self {
            client: StashClient::new(),
            pricing,
            snapshots: Vec::new(),
            cached_tabs: None,
        }
    }

    pub fn set_session(&mut self, poesessid: String, account_name: String) {
        self.client.set_credentials(poesessid, account_name);
    }

    pub fn clear_session(&mut self) {
        self.client.clear_credentials();
        self.cached_tabs = None;
    }

    pub fn is_authenticated(&self) -> bool {
        self.client.is_configured()
    }

    pub async fn validate_session(&mut self) -> Result<(), String> {
        self.client.validate_session().await
    }

    pub async fn fetch_tabs(&mut self, league: &str) -> Result<Vec<StashTab>, String> {
        let tabs = self.client.fetch_tabs(league).await?;
        self.cached_tabs = Some(tabs.clone());
        Ok(tabs)
    }

    pub fn get_cached_tabs(&self) -> Option<&Vec<StashTab>> {
        self.cached_tabs.as_ref()
    }

    pub async fn ensure_pricing_fresh(&self, league: &str) -> Result<(), String> {
        self.pricing.ensure_fresh(league).await
    }

    pub async fn scan_single_tab(
        &mut self,
        league: &str,
        tab: &StashTab,
    ) -> Result<(TabSummary, Vec<PricedItem>), String> {
        let items = self.client.fetch_tab_items(league, tab.index).await?;
        let mut tab_chaos = 0.0;
        let mut tab_priced = Vec::new();

        for item in &items {
            let priced = price_item(item, &self.pricing).await;
            if let Some(tp) = priced.total_price {
                tab_chaos += tp;
            }
            tab_priced.push(priced);
        }

        let summary = TabSummary {
            tab_name: tab.id.clone(),
            tab_index: tab.index,
            chaos_value: tab_chaos,
            item_count: tab_priced.len() as u32,
        };

        Ok((summary, tab_priced))
    }

    pub async fn finalize_snapshot(
        &mut self,
        mut tab_summaries: Vec<TabSummary>,
        all_priced: Vec<PricedItem>,
        rate_limited: bool,
    ) -> PortfolioSummary {
        let total_chaos: f64 = tab_summaries.iter().map(|t| t.chaos_value).sum();
        let divine_ratio = self.pricing.divine_ratio().await;
        let total_divine = if divine_ratio > 0.0 {
            total_chaos / divine_ratio
        } else {
            0.0
        };

        self.snapshots.push(SnapshotRecord {
            timestamp: chrono::Utc::now(),
            total_chaos,
        });

        let mut sorted_priced = all_priced;
        sorted_priced.sort_by(|a, b| {
            b.total_price
                .unwrap_or(0.0)
                .partial_cmp(&a.total_price.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        tab_summaries.sort_by(|a, b| {
            b.chaos_value
                .partial_cmp(&a.chaos_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        PortfolioSummary {
            total_chaos,
            total_divine,
            tab_summaries,
            items: sorted_priced,
            chaos_per_hour: self.chaos_per_hour(),
            snapshot_count: self.snapshots.len() as u32,
            rate_limited,
        }
    }

    pub async fn take_snapshot(&mut self, league: &str) -> Result<PortfolioSummary, String> {
        self.ensure_pricing_fresh(league).await?;
        let tabs = self.fetch_tabs(league).await?;
        let mut tab_summaries = Vec::new();
        let mut all_priced: Vec<PricedItem> = Vec::new();

        for tab in &tabs {
            let (summary, priced) = self.scan_single_tab(league, tab).await?;
            tab_summaries.push(summary);
            all_priced.extend(priced);
        }

        Ok(self.finalize_snapshot(tab_summaries, all_priced, false).await)
    }

    fn chaos_per_hour(&self) -> Option<f64> {
        if self.snapshots.len() < 2 {
            return None;
        }
        let first = &self.snapshots[0];
        let last = &self.snapshots[self.snapshots.len() - 1];
        let hours = (last.timestamp - first.timestamp).num_seconds() as f64 / 3600.0;
        if hours < 0.01 {
            return None;
        }
        let diff = last.total_chaos - first.total_chaos;
        Some(diff / hours)
    }
}
