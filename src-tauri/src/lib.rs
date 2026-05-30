mod commands;
mod state;

use commands::maps::MapTrackerState;
use commands::stash::{PricingState, StashTrackerState};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("poe_scout=debug,poe_data=debug,poe_pricing=debug,info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let rt = tokio::runtime::Runtime::new()?;
            let app_handle = app.handle().clone();
            let state = rt.block_on(async {
                let state = AppState::init(&data_dir).await?;

                // Auto-start map tracker
                let db_path = data_dir.join("poescout.db");
                let tracker_state: MapTrackerState = Arc::new(Mutex::new(None));
                if let Some(client_txt_path) = poe_core::config::detect_client_txt() {
                    match poe_maps::MapTracker::new(&db_path, client_txt_path) {
                        Ok(mut tracker) => {
                            if let Err(e) = tracker.start() {
                                tracing::error!("Failed to start map tracker: {}", e);
                            }
                            // Apply persisted character (for death/level-up attribution) + league.
                            if let Ok(raw) =
                                std::fs::read_to_string(data_dir.join("settings.json"))
                            {
                                if let Ok(v) =
                                    serde_json::from_str::<serde_json::Value>(&raw)
                                {
                                    tracker.set_character(
                                        v.get("character")
                                            .and_then(|c| c.as_str())
                                            .map(String::from),
                                    );
                                    tracker.set_league(
                                        v.get("league")
                                            .and_then(|c| c.as_str())
                                            .map(String::from),
                                    );
                                }
                            }
                            *tracker_state.lock().await = Some(tracker);
                            let ts = tracker_state.clone();
                            let ah = app_handle.clone();
                            tokio::spawn(async move {
                                commands::maps::poll_events_loop(ah, ts).await;
                            });
                        }
                        Err(e) => tracing::error!("Failed to init map tracker: {}", e),
                    }
                } else {
                    tracing::warn!("Client.txt not found — map tracker disabled");
                }
                app_handle.manage(tracker_state);

                // Initialize pricing engine + stash tracker
                let pricing_engine: PricingState = Arc::new(poe_pricing::PricingEngine::new());
                app_handle.manage(pricing_engine.clone());

                let mut stash_tracker = poe_stash::StashTracker::new(pricing_engine);
                if let Some((sessid, account)) =
                    commands::stash::load_credentials_sync(&data_dir)
                {
                    tracing::info!("Loaded saved credentials for account: {}", account);
                    stash_tracker.set_session(sessid, account);
                }
                let stash_state: StashTrackerState = Arc::new(Mutex::new(stash_tracker));
                app_handle.manage(stash_state);

                Ok::<_, anyhow::Error>(state)
            })?;

            app.manage(state);
            app.manage(rt);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::lookup::search_mods,
            commands::lookup::search_bases,
            commands::lookup::get_mod_by_id,
            commands::lookup::list_item_classes,
            commands::lookup::list_bases_by_class,
            commands::lookup::get_affixes_for_base,
            commands::pob::decode_pob_code,
            commands::pob::detect_pob,
            commands::pob::launch_pob_app,
            commands::capture::capture_item_text,
            commands::capture::get_poe_window_rect,
            commands::capture::focus_poe_window,
            commands::capture::is_poe_foreground,
            commands::capture::capture_poe_test,
            commands::maps::get_tracker_state,
            commands::maps::get_map_history,
            commands::maps::get_map_stats,
            commands::maps::get_map_sessions,
            commands::maps::get_map_type_stats,
            commands::maps::get_items_per_hour,
            commands::maps::get_net_worth_history,
            commands::maps::get_session_detail,
            commands::maps::set_tracked_character,
            commands::maps::clear_map_history,
            commands::stash::set_session_id,
            commands::stash::get_stash_tabs,
            commands::stash::take_stash_snapshot,
            commands::stash::take_selective_snapshot,
            commands::stash::delete_credentials,
            commands::stash::validate_credentials,
            commands::stash::refresh_prices,
            commands::stash::get_price,
            commands::stash::save_credentials,
            commands::stash::load_credentials,
            commands::stash::get_current_league,
            commands::stash::get_all_leagues,
            commands::stash::save_settings,
            commands::stash::load_settings,
            commands::stash::save_portfolio,
            commands::stash::load_portfolio,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PoeScout");
}
