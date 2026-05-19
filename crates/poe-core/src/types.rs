use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub generation_type: String,
    pub group: String,
    pub required_level: i32,
    pub text: String,
    pub stats: Vec<ModStat>,
    pub spawn_weights: Vec<SpawnWeight>,
    pub tags: Vec<String>,
    pub is_essence_only: bool,
    pub mod_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModStat {
    pub id: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItem {
    pub id: String,
    pub name: String,
    pub item_class: String,
    pub drop_level: i32,
    pub tags: Vec<String>,
    pub implicits: Vec<String>,
    pub implicit_stats: Vec<ImplicitStat>,
    pub implicit_text: Vec<String>,
    pub properties: BaseItemProperties,
    pub requirements: BaseItemRequirements,
    pub image_url: Option<String>,
    pub inventory_width: Option<i32>,
    pub inventory_height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItemProperties {
    #[serde(default)]
    pub armour_min: Option<i32>,
    #[serde(default)]
    pub armour_max: Option<i32>,
    #[serde(default)]
    pub evasion_min: Option<i32>,
    #[serde(default)]
    pub evasion_max: Option<i32>,
    #[serde(default)]
    pub energy_shield_min: Option<i32>,
    #[serde(default)]
    pub energy_shield_max: Option<i32>,
    #[serde(default)]
    pub movement_speed: Option<i32>,
    #[serde(default)]
    pub block: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItemRequirements {
    #[serde(default)]
    pub level: Option<i32>,
    #[serde(default)]
    pub strength: Option<i32>,
    #[serde(default)]
    pub dexterity: Option<i32>,
    #[serde(default)]
    pub intelligence: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub generation_type: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub min_level: Option<i32>,
    #[serde(default)]
    pub max_level: Option<i32>,
    #[serde(default)]
    pub min_weight: Option<i32>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub mods: Vec<Mod>,
    pub total: usize,
    pub query_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicitStat {
    pub stat_id: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffixEntry {
    pub mod_data: Mod,
    pub effective_weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffixesForBaseResult {
    pub affixes: Vec<AffixEntry>,
    pub query_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSearchQuery {
    pub text: String,
    #[serde(default)]
    pub item_class: Option<String>,
    #[serde(default)]
    pub min_level: Option<i32>,
    #[serde(default)]
    pub max_level: Option<i32>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSearchResult {
    pub items: Vec<BaseItem>,
    pub total: usize,
    pub query_ms: f64,
}
