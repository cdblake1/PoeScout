use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub league: String,
    pub client_txt_path: Option<PathBuf>,
    pub pob_path: Option<PathBuf>,
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = dirs_data_dir().join("PoeScout");
        Self {
            league: "Phrecia".to_string(),
            client_txt_path: detect_client_txt(),
            pob_path: None,
            data_dir: data_dir.clone(),
            db_path: data_dir.join("poescout.db"),
        }
    }
}

fn dirs_data_dir() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn detect_client_txt() -> Option<PathBuf> {
    let steam = PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile\logs\Client.txt");
    let standalone = PathBuf::from(r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile\logs\Client.txt");
    if steam.exists() {
        Some(steam)
    } else if standalone.exists() {
        Some(standalone)
    } else {
        None
    }
}
