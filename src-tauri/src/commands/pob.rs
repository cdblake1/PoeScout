use poe_pob::{decode_build_code, BuildSummary};
use std::path::PathBuf;

#[tauri::command]
pub fn decode_pob_code(input: String) -> Result<BuildSummary, String> {
    decode_build_code(&input)
}

#[tauri::command]
pub fn detect_pob() -> Option<String> {
    poe_pob::detect_pob_path().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn launch_pob_app(
    pob_path: String,
    build_code: Option<String>,
) -> Result<(), String> {
    let path = PathBuf::from(pob_path);
    poe_pob::launch_pob(&path, build_code.as_deref())
}
