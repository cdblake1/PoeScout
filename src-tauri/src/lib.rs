mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("poe_scout=debug,poe_data=debug,info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let rt = tokio::runtime::Runtime::new()?;
            let state = rt.block_on(async {
                AppState::init(&data_dir).await
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running PoeScout");
}
