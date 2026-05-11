use crate::state::AppState;
use poe_core::types::{AffixesForBaseResult, BaseItem, BaseSearchQuery, BaseSearchResult, Mod, SearchQuery, SearchResult};
use tauri::State;

#[tauri::command]
pub fn search_mods(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    state
        .engine
        .db
        .search_mods(&query)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_bases(
    query: BaseSearchQuery,
    state: State<'_, AppState>,
) -> Result<BaseSearchResult, String> {
    state
        .engine
        .db
        .search_bases(&query)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_mod_by_id(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<Mod>, String> {
    let index = state.engine.index.blocking_read();
    Ok(index.get_mod(&id).cloned())
}

#[tauri::command]
pub fn list_item_classes(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    state
        .engine
        .db
        .list_item_classes()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_bases_by_class(
    item_class: String,
    state: State<'_, AppState>,
) -> Result<Vec<BaseItem>, String> {
    state
        .engine
        .db
        .list_bases_by_class(&item_class)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_affixes_for_base(
    base_tags: Vec<String>,
    state: State<'_, AppState>,
) -> Result<AffixesForBaseResult, String> {
    state
        .engine
        .db
        .get_affixes_for_base(&base_tags)
        .map_err(|e| e.to_string())
}
