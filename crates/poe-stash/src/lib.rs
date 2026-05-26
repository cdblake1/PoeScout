pub mod client;
pub mod inventory;
pub mod matcher;

use client::StashClient;
use inventory::{diff_inventory, InventoryItem, LootDelta, PricedLoot};
use matcher::price_item;
use poe_core::types::*;
use poe_pricing::PricingEngine;
use std::sync::Arc;

pub struct StashTracker {
    client: StashClient,
    pricing: Arc<PricingEngine>,
    snapshots: Vec<SnapshotRecord>,
    cached_tabs: Option<Vec<StashTab>>,
    /// Character inventory captured at map start, for per-map loot diffing (6.3).
    char_baseline: Option<Vec<InventoryItem>>,
    /// Drop stacks below this chaos value from the snapshot total. The items
    /// list still shows everything; only the sum/snapshot total excludes noise.
    /// 0 (default) = no filter. (6.5b)
    min_stack_chaos: f64,
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
            char_baseline: None,
            min_stack_chaos: 0.0,
        }
    }

    /// Set the per-stack chaos threshold for snapshot-total filtering (6.5b).
    pub fn set_min_stack_chaos(&mut self, v: f64) {
        self.min_stack_chaos = v.max(0.0);
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
            // Noise filter (6.5b): only stacks ≥ min_stack_chaos contribute to
            // the snapshot total. Items below threshold still appear in the items
            // list — we just hide them from the chart/snapshot total.
            if let Some(tp) = priced.total_price {
                if tp >= self.min_stack_chaos {
                    tab_chaos += tp;
                }
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

    // --- Per-map loot (6.3) ---

    /// Snapshot the character's inventory as the baseline for a new map.
    pub async fn snapshot_character_baseline(&mut self, character: &str) -> Result<(), String> {
        let items = self.client.fetch_character_inventory(character).await?;
        self.char_baseline = Some(items);
        Ok(())
    }

    /// Diff current character inventory vs the baseline, price the gained loot,
    /// then reset the baseline to the current snapshot. Returns (total_chaos, lines).
    pub async fn capture_loot(
        &mut self,
        character: &str,
        league: &str,
    ) -> Result<(f64, Vec<PricedLoot>), String> {
        self.pricing.ensure_fresh(league).await?;
        let curr = self.client.fetch_character_inventory(character).await?;
        let baseline = self.char_baseline.take().unwrap_or_default();
        let deltas: Vec<LootDelta> = diff_inventory(&baseline, &curr);
        let priced = self.price_loot(&deltas).await;
        let total: f64 = priced.iter().filter_map(|p| p.total_chaos).sum();
        self.char_baseline = Some(curr);
        Ok((total, priced))
    }

    async fn price_loot(&self, deltas: &[LootDelta]) -> Vec<PricedLoot> {
        let mut out = Vec::new();
        for d in deltas {
            let item = StashItem {
                name: d.name.clone(),
                type_line: d.type_line.clone(),
                base_type: None,
                stack_size: Some(d.stack_size),
                max_stack_size: None,
                icon: String::new(),
                ilvl: None,
                identified: None,
                frame_type: d.frame_type,
            };
            let priced = price_item(&item, &self.pricing).await;
            out.push(PricedLoot {
                name: d.name.clone(),
                type_line: d.type_line.clone(),
                stack_size: d.stack_size,
                unit_chaos: priced.unit_price,
                total_chaos: priced.total_price,
                frame_type: d.frame_type,
            });
        }
        out
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
