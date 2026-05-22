use poe_maps::state::{MapRun, MapStats, StateEvent, TrackerState};
use poe_maps::MapTracker;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

pub type MapTrackerState = Arc<Mutex<Option<MapTracker>>>;

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
pub async fn get_map_stats(
    tracker_state: State<'_, MapTrackerState>,
) -> Result<MapStats, String> {
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

pub async fn poll_events_loop(app: AppHandle, tracker_state: MapTrackerState) {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        let mut guard = tracker_state.lock().await;
        let tracker = match guard.as_mut() {
            Some(t) if t.is_running() => t,
            _ => return,
        };

        let events = tracker.poll_events();
        drop(guard);

        for event in events {
            match &event {
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
    }
}
