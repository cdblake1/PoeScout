use poe_core::types::*;
use poe_pricing::PricingEngine;
use poe_stash::StashTracker;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
struct ScanProgress {
    current: u32,
    total: u32,
    tab_name: String,
    tab_type: String,
}

#[derive(Serialize, Deserialize)]
struct SavedPortfolio {
    portfolio: PortfolioSummary,
    last_updated: String,
}

pub type PricingState = Arc<PricingEngine>;
pub type StashTrackerState = Arc<Mutex<StashTracker>>;

#[tauri::command]
pub async fn set_session_id(
    poesessid: String,
    account_name: String,
    stash_state: State<'_, StashTrackerState>,
) -> Result<(), String> {
    let mut tracker = stash_state.lock().await;
    tracker.set_session(poesessid, account_name);
    Ok(())
}

#[tauri::command]
pub async fn get_stash_tabs(
    league: String,
    stash_state: State<'_, StashTrackerState>,
) -> Result<Vec<StashTab>, String> {
    let mut tracker = stash_state.lock().await;
    tracker.fetch_tabs(&league).await
}

#[tauri::command]
pub async fn take_stash_snapshot(
    league: String,
    stash_state: State<'_, StashTrackerState>,
) -> Result<PortfolioSummary, String> {
    let mut tracker = stash_state.lock().await;
    tracker.take_snapshot(&league).await
}

#[tauri::command]
pub async fn refresh_prices(
    league: String,
    pricing_state: State<'_, PricingState>,
) -> Result<(), String> {
    pricing_state.refresh(&league).await
}

#[tauri::command]
pub async fn get_price(
    item_name: String,
    league: String,
    pricing_state: State<'_, PricingState>,
) -> Result<Option<PriceRecord>, String> {
    pricing_state.ensure_fresh(&league).await?;
    Ok(pricing_state.get_price(&item_name).await)
}

#[tauri::command]
pub async fn save_credentials(
    poesessid: String,
    account_name: String,
    app: AppHandle,
    stash_state: State<'_, StashTrackerState>,
) -> Result<(), String> {
    // Validate the session before saving
    {
        let mut tracker = stash_state.lock().await;
        tracker.set_session(poesessid.clone(), account_name.clone());
        tracker.validate_session().await.map_err(|e| {
            tracker.clear_session();
            e
        })?;
    }

    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("credentials.json");

    let data = serde_json::json!({
        "poesessid": poesessid,
        "account_name": account_name
    });

    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("Failed to save credentials: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_credentials(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("credentials.json");

    if !path.exists() {
        return Ok(None);
    }

    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read credentials: {}", e))?;

    let data: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid credentials file: {}", e))?;

    Ok(Some(data))
}

#[tauri::command]
pub async fn get_current_league(
    pricing_state: State<'_, PricingState>,
) -> Result<String, String> {
    pricing_state.get_current_league().await
}

#[tauri::command]
pub async fn get_all_leagues(
    pricing_state: State<'_, PricingState>,
) -> Result<Vec<String>, String> {
    pricing_state.get_all_leagues().await
}

/// Shallow-merge `incoming` over `base` so a partial save from one panel (e.g.
/// `league`) doesn't clobber keys written by another (e.g. `selected_tabs`,
/// `character`) — both are needed by the automatic session lifecycle. If `base`
/// isn't a JSON object, `incoming` wins.
fn merge_settings(base: serde_json::Value, incoming: serde_json::Value) -> serde_json::Value {
    match (base, incoming) {
        (serde_json::Value::Object(mut b), serde_json::Value::Object(inc)) => {
            for (k, v) in inc {
                b.insert(k, v);
            }
            serde_json::Value::Object(b)
        }
        (_, incoming) => incoming,
    }
}

#[tauri::command]
pub async fn save_settings(
    settings: serde_json::Value,
    app: AppHandle,
) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json");

    let base = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let merged = merge_settings(base, settings);

    std::fs::write(&path, serde_json::to_string_pretty(&merged).unwrap())
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json");

    if !path.exists() {
        return Ok(None);
    }

    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;

    let data: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid settings file: {}", e))?;

    Ok(Some(data))
}

#[tauri::command]
pub async fn take_selective_snapshot(
    league: String,
    tab_indices: Vec<u32>,
    app: AppHandle,
    stash_state: State<'_, StashTrackerState>,
) -> Result<PortfolioSummary, String> {
    let mut tracker = stash_state.lock().await;

    tracker.set_min_stack_chaos(crate::commands::maps::settings_min_stack_chaos(&app));
    tracker.ensure_pricing_fresh(&league).await?;

    let tabs = match tracker.get_cached_tabs() {
        Some(cached) => cached.clone(),
        None => tracker.fetch_tabs(&league).await?,
    };

    let selected: Vec<_> = tabs
        .into_iter()
        .filter(|t| tab_indices.contains(&t.index))
        .collect();

    let total = selected.len() as u32;
    let mut tab_summaries = Vec::new();
    let mut all_priced: Vec<PricedItem> = Vec::new();
    let mut rate_limited = false;

    for (i, tab) in selected.iter().enumerate() {
        let _ = app.emit(
            "stash:scan-progress",
            ScanProgress {
                current: (i + 1) as u32,
                total,
                tab_name: tab.id.clone(),
                tab_type: tab.tab_type.clone(),
            },
        );

        match tracker.scan_single_tab(&league, tab).await {
            Ok((summary, priced)) => {
                tab_summaries.push(summary);
                all_priced.extend(priced);
            }
            Err(e) if e.contains("Rate limited") => {
                rate_limited = true;
                break;
            }
            Err(e) => return Err(e),
        }
    }

    let summary = tracker
        .finalize_snapshot(tab_summaries, all_priced, rate_limited)
        .await;
    drop(tracker);

    // Record a net-worth time-series point (6.5).
    let map_state = app.state::<crate::commands::maps::MapTrackerState>();
    let guard = map_state.lock().await;
    if let Some(t) = guard.as_ref() {
        let _ = t.record_portfolio_snapshot(summary.total_chaos, summary.total_divine);
    }

    Ok(summary)
}

#[tauri::command]
pub async fn validate_credentials(
    stash_state: State<'_, StashTrackerState>,
) -> Result<bool, String> {
    let mut tracker = stash_state.lock().await;
    if !tracker.is_authenticated() {
        return Ok(false);
    }
    match tracker.validate_session().await {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn delete_credentials(
    app: AppHandle,
    stash_state: State<'_, StashTrackerState>,
) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("credentials.json");

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete credentials: {}", e))?;
    }

    let mut tracker = stash_state.lock().await;
    tracker.clear_session();

    Ok(())
}

#[tauri::command]
pub async fn save_portfolio(
    portfolio: PortfolioSummary,
    app: AppHandle,
) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("portfolio.json");

    let saved = SavedPortfolio {
        portfolio,
        last_updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string(),
    };

    std::fs::write(&path, serde_json::to_string(&saved).unwrap())
        .map_err(|e| format!("Failed to save portfolio: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_portfolio(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("portfolio.json");

    if !path.exists() {
        return Ok(None);
    }

    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read portfolio: {}", e))?;

    let data: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid portfolio file: {}", e))?;

    Ok(Some(data))
}

pub fn load_credentials_sync(
    data_dir: &std::path::Path,
) -> Option<(String, String)> {
    let path = data_dir.join("credentials.json");
    if !path.exists() {
        return None;
    }
    let contents = std::fs::read_to_string(&path).ok()?;
    let data: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let sessid = data["poesessid"].as_str()?.to_string();
    let account = data["account_name"].as_str()?.to_string();
    if sessid.is_empty() || account.is_empty() {
        return None;
    }
    Some((sessid, account))
}

#[cfg(test)]
mod tests {
    use super::merge_settings;
    use serde_json::json;

    #[test]
    fn merge_overwrites_and_preserves_other_keys() {
        let base = json!({"league": "Standard", "selected_tabs": [1, 2], "character": "Me"});
        let merged = merge_settings(base, json!({"league": "Mirage"}));
        assert_eq!(merged["league"], "Mirage");
        assert_eq!(merged["selected_tabs"], json!([1, 2]));
        assert_eq!(merged["character"], "Me");
    }

    #[test]
    fn merge_adds_new_keys() {
        let merged = merge_settings(
            json!({"league": "Mirage"}),
            json!({"character": "Me", "selected_tabs": [3]}),
        );
        assert_eq!(merged["league"], "Mirage");
        assert_eq!(merged["character"], "Me");
        assert_eq!(merged["selected_tabs"], json!([3]));
    }

    #[test]
    fn merge_non_object_base_falls_back_to_incoming() {
        let merged = merge_settings(json!("corrupt"), json!({"league": "Mirage"}));
        assert_eq!(merged, json!({"league": "Mirage"}));
    }
}
