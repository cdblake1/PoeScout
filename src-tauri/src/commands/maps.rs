use crate::commands::stash::StashTrackerState;
use poe_maps::session::{next_session_action, SessionAction};
use poe_maps::state::{
    ItemRate, ItemRateScope, MapRun, MapSession, MapStats, MapTypeStat, PortfolioSnapshot,
    StateEvent, TrackerState,
};
use poe_maps::MapTracker;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

pub type MapTrackerState = Arc<Mutex<Option<MapTracker>>>;

const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 900; // 15 min in town/hideout auto-ends a session

#[tauri::command]
pub async fn get_tracker_state(
    tracker_state: State<'_, MapTrackerState>,
) -> Result<TrackerState, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => Ok(tracker.state()),
        None => Ok(TrackerState::Stopped),
    }
}

#[tauri::command]
pub async fn get_map_history(
    limit: u32,
    offset: u32,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<Vec<MapRun>, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.get_history(limit, offset).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn get_map_stats(tracker_state: State<'_, MapTrackerState>) -> Result<MapStats, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.get_stats().map_err(|e| e.to_string()),
        None => Ok(MapStats {
            total_runs: 0,
            avg_duration_secs: 0.0,
            maps_per_hour: 0.0,
            total_deaths: 0,
        }),
    }
}

#[tauri::command]
pub async fn get_map_sessions(
    limit: u32,
    offset: u32,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<Vec<MapSession>, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.get_sessions(limit, offset).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn get_map_type_stats(
    tracker_state: State<'_, MapTrackerState>,
) -> Result<Vec<MapTypeStat>, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.get_map_type_stats().map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn get_items_per_hour(
    scope: ItemRateScope,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<Vec<ItemRate>, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.get_items_per_hour(&scope).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn get_net_worth_history(
    limit: u32,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<Vec<PortfolioSnapshot>, String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker
            .get_portfolio_snapshots(limit)
            .map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[derive(Serialize)]
pub struct SessionDetail {
    session: MapSession,
    runs: Vec<MapRun>,
}

#[tauri::command]
pub async fn get_session_detail(
    session_id: i64,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<SessionDetail, String> {
    let guard = tracker_state.lock().await;
    let tracker = guard.as_ref().ok_or("Map tracker not running")?;
    let session = tracker
        .get_sessions(1000, 0)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|s| s.id == Some(session_id))
        .ok_or("Session not found")?;
    let runs = tracker
        .get_session_runs(session_id)
        .map_err(|e| e.to_string())?;
    Ok(SessionDetail { session, runs })
}

/// Set the player's character so deaths/level-ups are attributed only to them.
#[tauri::command]
pub async fn set_tracked_character(
    character: Option<String>,
    tracker_state: State<'_, MapTrackerState>,
) -> Result<(), String> {
    let mut guard = tracker_state.lock().await;
    if let Some(tracker) = guard.as_mut() {
        tracker.set_character(character);
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_map_history(
    tracker_state: State<'_, MapTrackerState>,
) -> Result<(), String> {
    let guard = tracker_state.lock().await;
    match &*guard {
        Some(tracker) => tracker.clear_history().map_err(|e| e.to_string()),
        None => Ok(()),
    }
}

// --- Auto session lifecycle helpers ---

fn read_settings(app: &AppHandle) -> Option<serde_json::Value> {
    let path = app.path().app_data_dir().ok()?.join("settings.json");
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn settings_league(app: &AppHandle) -> Option<String> {
    read_settings(app)?
        .get("league")?
        .as_str()
        .map(String::from)
}

fn settings_character(app: &AppHandle) -> Option<String> {
    let c = read_settings(app)?.get("character")?.as_str()?.to_string();
    if c.is_empty() {
        None
    } else {
        Some(c)
    }
}

/// Per-stack chaos threshold for stash snapshot totals (6.5b). 0 = no filter.
pub(crate) fn settings_min_stack_chaos(app: &AppHandle) -> f64 {
    read_settings(app)
        .and_then(|v| v.get("min_stack_chaos").and_then(|x| x.as_f64()))
        .unwrap_or(0.0)
}

/// poe.ninja listing-count threshold for snapshot totals (6.5c). 0 = no filter.
pub(crate) fn settings_min_listing_count(app: &AppHandle) -> u32 {
    read_settings(app)
        .and_then(|v| v.get("min_listing_count").and_then(|x| x.as_u64()))
        .map(|n| n as u32)
        .unwrap_or(0)
}

/// Optional price-league override; falls back to the game league. Lets the user
/// price a dead/private league against Standard, for example. (6.5c)
pub(crate) fn settings_price_league(app: &AppHandle) -> Option<String> {
    read_settings(app)?
        .get("price_league")?
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn settings_idle_timeout(app: &AppHandle) -> u64 {
    read_settings(app)
        .and_then(|v| v.get("session_idle_timeout_secs").and_then(|t| t.as_u64()))
        .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS)
}

/// Best-effort total chaos of the user's selected stash tabs. Returns `None` if
/// not authenticated, no tabs selected, or the scan fails (e.g. rate limited) —
/// in which case a session is recorded without a profit figure.
async fn snapshot_total_chaos(app: &AppHandle) -> Option<f64> {
    let settings = read_settings(app)?;
    let league = settings.get("league")?.as_str()?.to_string();
    let tabs: Vec<u32> = settings
        .get("selected_tabs")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u32))
        .collect();
    if tabs.is_empty() {
        return None;
    }

    let stash_state = app.state::<StashTrackerState>();
    let mut tracker = stash_state.lock().await;
    if !tracker.is_authenticated() {
        return None;
    }
    tracker.set_min_stack_chaos(settings_min_stack_chaos(app));
    tracker.set_min_listing_count(settings_min_listing_count(app));
    // Pricing can use a different league than the game (6.5c).
    let price_league = settings_price_league(app).unwrap_or_else(|| league.clone());
    tracker.ensure_pricing_fresh(&price_league).await.ok()?;

    let all_tabs = match tracker.get_cached_tabs() {
        Some(cached) => cached.clone(),
        None => tracker.fetch_tabs(&league).await.ok()?,
    };
    let selected: Vec<_> = all_tabs
        .into_iter()
        .filter(|t| tabs.contains(&t.index))
        .collect();

    let mut total = 0.0;
    for tab in &selected {
        match tracker.scan_single_tab(&league, tab).await {
            Ok((summary, _priced)) => total += summary.chaos_value,
            Err(_) => return None, // rate limited / error → no reliable figure
        }
    }
    Some(total)
}

pub async fn poll_events_loop(app: AppHandle, tracker_state: MapTrackerState) {
    // Wall-clock timer for auto-ending a session after sustained town/hideout idle.
    let mut idle_since: Option<Instant> = None;
    // started_at of the run we last baselined, so we snapshot inventory once per run.
    let mut current_run_key: Option<String> = None;
    // Previous tick's state, used to detect InMap→Idle transitions and freeze
    // the run's "end" inventory at suspend time before town activity leaks in.
    let mut prev_state: Option<TrackerState> = None;

    loop {
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Drain + persist events, then read state without holding the lock further.
        let (events, state, active_session) = {
            let mut guard = tracker_state.lock().await;
            let tracker = match guard.as_mut() {
                Some(t) if t.is_running() => t,
                _ => return,
            };
            let events = tracker.poll_events();
            (events, tracker.state(), tracker.active_session_id())
        };

        for event in &events {
            match event {
                StateEvent::StateChanged(state) => {
                    let _ = app.emit("map-tracker:state-change", state);
                }
                StateEvent::MapCompleted(run) => {
                    let _ = app.emit("map-tracker:map-complete", run);
                }
                StateEvent::Death {
                    map_name,
                    total_deaths,
                } => {
                    #[derive(serde::Serialize, Clone)]
                    struct DeathPayload {
                        map_name: String,
                        total_deaths: u32,
                    }
                    let _ = app.emit(
                        "map-tracker:death",
                        DeathPayload {
                            map_name: map_name.clone(),
                            total_deaths: *total_deaths,
                        },
                    );
                }
            }
        }

        // --- Suspend-time inventory snapshot (6.3 town-leak fix) ---
        // Detect the InMap → Idle transition (entering town/hideout) and freeze
        // the prior run's end inventory NOW, before any town activity leaks in.
        // capture_loot (below, fired on the eventual MapCompleted) will prefer
        // this pending snapshot over a fresh fetch.
        let was_in_map = matches!(prev_state, Some(TrackerState::InMap { .. }));
        let now_idle = matches!(state, TrackerState::Idle { .. });
        if was_in_map && now_idle {
            if let Some(character) = settings_character(&app) {
                let stash = app.state::<StashTrackerState>();
                let mut t = stash.lock().await;
                if t.is_authenticated() {
                    if let Err(e) = t.snapshot_character_at_suspend(&character).await {
                        tracing::warn!("Loot suspend snapshot failed: {}", e);
                    }
                }
            }
        }
        prev_state = Some(state.clone());

        // --- Per-map loot capture (6.3; needs a configured character + creds) ---
        if let Some(character) = settings_character(&app) {
            let mut captured_any = false;

            for event in &events {
                if let StateEvent::MapCompleted(run) = event {
                    if let Some(run_id) = run.id {
                        let league = settings_league(&app).unwrap_or_default();
                        let captured = {
                            let stash = app.state::<StashTrackerState>();
                            let mut t = stash.lock().await;
                            if t.is_authenticated() && !league.is_empty() {
                                t.capture_loot(&character, &league).await.ok()
                            } else {
                                None
                            }
                        };
                        if let Some((total, priced)) = captured {
                            captured_any = true;
                            let items: Vec<poe_maps::state::LootItem> = priced
                                .into_iter()
                                .map(|p| poe_maps::state::LootItem {
                                    name: p.name,
                                    type_line: p.type_line,
                                    stack_size: p.stack_size,
                                    unit_chaos: p.unit_chaos,
                                    total_chaos: p.total_chaos,
                                    frame_type: p.frame_type,
                                })
                                .collect();
                            let mut guard = tracker_state.lock().await;
                            if let Some(tracker) = guard.as_mut() {
                                if let Err(e) = tracker.set_run_loot(run_id, total, &items) {
                                    tracing::error!("Failed to save run loot: {}", e);
                                }
                            }
                            drop(guard);
                            let _ = app.emit("map-tracker:loot", run_id);
                        }
                    }
                }
            }

            // Baseline the inventory at the start of a new run, unless a capture
            // this tick already reset the baseline to the current inventory.
            if let TrackerState::InMap { started_at, .. } = &state {
                if current_run_key.as_deref() != Some(started_at.as_str()) {
                    current_run_key = Some(started_at.clone());
                    if !captured_any {
                        let stash = app.state::<StashTrackerState>();
                        let mut t = stash.lock().await;
                        if t.is_authenticated() {
                            if let Err(e) = t.snapshot_character_baseline(&character).await {
                                tracing::warn!("Loot baseline snapshot failed: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // --- Automatic session lifecycle ---
        // Maintain the wall-clock idle timer here; delegate the decision to the
        // pure `next_session_action` (unit-tested in poe-maps).
        let timeout = settings_idle_timeout(&app);
        if matches!(state, TrackerState::InMap { .. }) {
            idle_since = None;
        } else if active_session.is_some() {
            if idle_since.is_none() {
                idle_since = Some(Instant::now());
            }
        } else {
            idle_since = None;
        }
        let idle_elapsed = idle_since.map(|t| t.elapsed().as_secs()).unwrap_or(0);

        match next_session_action(&state, active_session.is_some(), idle_elapsed, timeout) {
            SessionAction::Start => {
                // First map out of town → open a session (snapshot stash for start value).
                let start_chaos = snapshot_total_chaos(&app).await;
                let league = settings_league(&app);
                let mut guard = tracker_state.lock().await;
                if let Some(tracker) = guard.as_mut() {
                    if tracker.active_session_id().is_none() {
                        if let Some(c) = start_chaos {
                            let _ = tracker.record_portfolio_snapshot(c, 0.0);
                        }
                        match tracker.start_session(league.as_deref(), None, start_chaos) {
                            Ok(id) => {
                                let _ = app.emit("map-tracker:session-start", id);
                            }
                            Err(e) => tracing::error!("Failed to start session: {}", e),
                        }
                    }
                }
            }
            SessionAction::End => {
                let end_chaos = snapshot_total_chaos(&app).await;
                let mut guard = tracker_state.lock().await;
                if let Some(tracker) = guard.as_mut() {
                    if let Some(sid) = tracker.active_session_id() {
                        if let Some(c) = end_chaos {
                            let _ = tracker.record_portfolio_snapshot(c, 0.0);
                        }
                        if let Err(e) = tracker.end_session(end_chaos) {
                            tracing::error!("Failed to end session: {}", e);
                        } else {
                            let _ = app.emit("map-tracker:session-end", sid);
                        }
                    }
                }
                idle_since = None;
            }
            SessionAction::None => {}
        }
    }
}
