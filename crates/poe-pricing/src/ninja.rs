use poe_core::types::PriceRecord;
use serde::Deserialize;
use std::collections::HashMap;

const BASE_URL: &str = "https://poe.ninja/poe1/api/economy/exchange/current";
const LEAGUES_URL: &str = "https://api.pathofexile.com/leagues?type=main&compact=1";
const SKIP_LEAGUES: &[&str] = &["Standard", "Hardcore", "SSF Standard", "SSF Hardcore"];

const CATEGORIES: &[&str] = &[
    "Currency",
    "Fragment",
    "DivinationCard",
    "SkillGem",
    "BaseType",
    "UniqueMap",
    "Map",
    "UniqueJewel",
    "UniqueFlask",
    "UniqueWeapon",
    "UniqueArmour",
    "UniqueAccessory",
    "Fossil",
    "Resonator",
    "Incubator",
    "Scarab",
    "Oil",
    "Essence",
];

#[derive(Deserialize)]
struct ExchangeOverview {
    core: CoreInfo,
    lines: Vec<ExchangeLine>,
    items: Vec<ExchangeItem>,
}

#[derive(Deserialize)]
struct CoreInfo {
    rates: HashMap<String, f64>,
    primary: Option<String>,
    items: Option<Vec<CoreItem>>,
}

#[derive(Deserialize)]
struct CoreItem {
    id: String,
    name: String,
    category: Option<String>,
    image: Option<String>,
}

#[derive(Deserialize)]
struct ExchangeLine {
    id: String,
    #[serde(rename = "primaryValue")]
    primary_value: f64,
}

#[derive(Deserialize)]
struct ExchangeItem {
    id: String,
    name: String,
    image: Option<String>,
    category: Option<String>,
}

pub struct NinjaClient {
    http: reqwest::Client,
}

impl NinjaClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("PoeScout/0.1")
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    async fn fetch_overview(
        &self,
        league: &str,
        category: &str,
    ) -> Result<Vec<PriceRecord>, String> {
        let url = format!("{}/overview", BASE_URL);
        tracing::debug!("Fetching poe.ninja: league={}, category={}", league, category);
        let resp = self
            .http
            .get(&url)
            .query(&[("league", league), ("type", category)])
            .send()
            .await
            .map_err(|e| format!("poe.ninja request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("poe.ninja returned HTTP {} for {}", resp.status(), category));
        }

        let overview: ExchangeOverview = resp
            .json()
            .await
            .map_err(|e| format!("poe.ninja parse error for {}: {}", category, e))?;

        let divine_rate = overview.core.rates.get("divine").copied();

        let items_by_id: HashMap<&str, &ExchangeItem> = overview
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();

        let mut records: Vec<PriceRecord> = overview
            .lines
            .into_iter()
            .filter_map(|line| {
                let item = items_by_id.get(line.id.as_str())?;
                Some(PriceRecord {
                    name: item.name.clone(),
                    category: item.category.clone().unwrap_or_else(|| category.to_string()),
                    chaos_value: line.primary_value,
                    divine_value: divine_rate.map(|r| line.primary_value * r),
                    icon: item.image.clone(),
                })
            })
            .collect();

        // Core items (e.g. Chaos Orb, Divine Orb) live in core.items, not in lines
        if let Some(core_items) = &overview.core.items {
            let primary = overview.core.primary.as_deref().unwrap_or("chaos");
            for ci in core_items {
                let chaos_value = if ci.id == primary {
                    1.0
                } else if let Some(&rate) = overview.core.rates.get(ci.id.as_str()) {
                    if rate > 0.0 { 1.0 / rate } else { continue }
                } else {
                    continue;
                };
                records.push(PriceRecord {
                    name: ci.name.clone(),
                    category: ci.category.clone().unwrap_or_else(|| category.to_string()),
                    chaos_value,
                    divine_value: divine_rate.map(|r| chaos_value * r),
                    icon: ci.image.clone(),
                });
            }
        }

        tracing::debug!("poe.ninja {}: {} records", category, records.len());
        Ok(records)
    }

    pub async fn fetch_all_prices(&self, league: &str) -> Result<Vec<PriceRecord>, String> {
        let mut futures = Vec::new();
        for cat in CATEGORIES {
            futures.push(self.fetch_overview(league, cat));
        }

        let results = futures::future::join_all(futures).await;

        let mut all = Vec::new();
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(records) => all.extend(records),
                Err(e) => tracing::warn!("Failed to fetch poe.ninja {}: {}", CATEGORIES[i], e),
            }
        }

        if all.is_empty() {
            return Err("No price data fetched from poe.ninja".to_string());
        }

        Ok(all)
    }

    async fn fetch_league_list(&self) -> Result<Vec<String>, String> {
        #[derive(Deserialize)]
        struct LeagueEntry {
            id: String,
        }

        let resp = self
            .http
            .get(LEAGUES_URL)
            .send()
            .await
            .map_err(|e| format!("Leagues API request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Leagues API returned HTTP {}", resp.status()));
        }

        let leagues: Vec<LeagueEntry> = resp
            .json()
            .await
            .map_err(|e| format!("Leagues API parse error: {}", e))?;

        Ok(leagues.into_iter().map(|l| l.id).collect())
    }

    pub async fn fetch_all_leagues(&self) -> Result<Vec<String>, String> {
        self.fetch_league_list().await
    }

    pub async fn fetch_current_league(&self) -> Result<String, String> {
        let leagues = self.fetch_league_list().await?;

        for id in &leagues {
            if SKIP_LEAGUES.iter().any(|s| id == s) {
                continue;
            }
            let lower = id.to_lowercase();
            if lower.contains("ssf")
                || lower.contains("solo self-found")
                || lower.contains("ruthless")
                || lower.contains("hardcore")
            {
                continue;
            }
            tracing::info!("Detected current challenge league: {}", id);
            return Ok(id.clone());
        }

        Err("Could not determine current challenge league".to_string())
    }
}
