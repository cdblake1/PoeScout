use serde::{Deserialize, Serialize};

/// Canonical hideout names from PoE 1 (source: PoE wiki).
/// `is_hideout()` also has a suffix fallback (`ends_with("Hideout")`) so
/// custom hideouts and any new official hideouts are caught automatically.
static HIDEOUTS: &[&str] = &[
    "Hideout",
    "Coastal Hideout",
    "Overgrown Hideout",
    "Lush Hideout",
    "Desert Hideout",
    "Glacial Hideout",
    "Backstreet Hideout",
    "Immaculate Hideout",
    "Celestial Hideout",
    "Unearthed Hideout",
    "Stately Hideout",
    "Brutal Hideout",
    "Divided Hideout",
    "Baleful Hideout",
    "Sunspire Hideout",
    "Luxurious Hideout",
    "Skeletal Hideout",
    "Enlightened Hideout",
];

/// The 10 Act town zones from PoE 1 (source: PoE wiki). Stable since 3.0.
/// No suffix fallback — new towns require a code update.
static TOWNS: &[&str] = &[
    "Lioneye's Watch",
    "The Forest Encampment",
    "The Sarn Encampment",
    "Highgate",
    "Overseer's Tower",
    "The Bridge Encampment",
    "The Harbour Bridge",
    "Oriath Docks",
    "Oriath",
    "Karui Shores",
];

/// Non-combat hub areas where a *map run must not start* — league hubs and
/// content hubs you pass through but don't "run". Previously these were
/// mis-counted as maps (the bug this classifier fixes).
static HUBS: &[&str] = &[
    "Kingsmarch",         // Settlers / Kalguur town hub
    "The Rogue Harbour",  // Heist hub
    "Azurite Mine",       // Delve hub
];

/// Classification of a PoE area, used to decide run lifecycle and to tag runs.
/// Prefer the internal area id (e.g. `MapWorldsStrand`) when available — it is
/// the canonical identity; the display name is a lossy fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaType {
    Map,
    Town,
    Hideout,
    /// League + content hubs (Kingsmarch, Rogue Harbour, Azurite Mine, league areas).
    Hub,
    Campaign,
    Heist,
    Delve,
    Lab,
    Sanctum,
    Boss,
    Unknown,
}

impl AreaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AreaType::Map => "map",
            AreaType::Town => "town",
            AreaType::Hideout => "hideout",
            AreaType::Hub => "hub",
            AreaType::Campaign => "campaign",
            AreaType::Heist => "heist",
            AreaType::Delve => "delve",
            AreaType::Lab => "lab",
            AreaType::Sanctum => "sanctum",
            AreaType::Boss => "boss",
            AreaType::Unknown => "unknown",
        }
    }
}

pub fn is_hideout(area: &str) -> bool {
    // Exact match or suffix match for custom hideouts
    HIDEOUTS.iter().any(|h| area == *h)
        || area.ends_with("Hideout")
        || area.ends_with("hideout")
}

pub fn is_town(area: &str) -> bool {
    TOWNS.iter().any(|t| area == *t)
}

/// Classify an area, preferring the internal id. Returns `Unknown` for areas we
/// can't positively identify by name alone (e.g. a map's display name) — the id
/// is what confirms `Map`.
pub fn classify(area_id: Option<&str>, area_name: &str) -> AreaType {
    if let Some(id) = area_id {
        if id.starts_with("MapWorlds") {
            return AreaType::Map;
        }
        if id.starts_with("Hideout") || id.ends_with("Hideout") {
            return AreaType::Hideout;
        }
        if id.contains("town") {
            // e.g. `1_1_town`, `2_11_endgame_town`
            return AreaType::Town;
        }
        if id.ends_with("League") {
            // e.g. `ChayulaLeague`, `KalguuranSettlersLeague`
            return AreaType::Hub;
        }
    }
    classify_by_name(area_name)
}

fn classify_by_name(name: &str) -> AreaType {
    if is_hideout(name) {
        AreaType::Hideout
    } else if is_town(name) {
        AreaType::Town
    } else if HUBS.contains(&name) {
        AreaType::Hub
    } else {
        // Could be a map, but the display name alone can't confirm it — leave it
        // to the id-aware path. Treated as runnable content (not idle).
        AreaType::Unknown
    }
}

/// An "idle" zone ends the current run and starts idle time. True for towns,
/// hideouts, and hubs (where you don't run maps).
pub fn is_idle_zone(area_id: Option<&str>, area_name: &str) -> bool {
    matches!(
        classify(area_id, area_name),
        AreaType::Town | AreaType::Hideout | AreaType::Hub
    )
}

/// Map tier from the area level: T1 = level 68 … T16 = 83, T17 = 84.
/// Returns `None` for sub-68 (campaign / non-map) levels.
pub fn map_tier(area_level: u32) -> Option<u32> {
    if area_level >= 68 {
        Some((area_level - 67).min(17))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hideout_detection() {
        assert!(is_hideout("Hideout"));
        assert!(is_hideout("Celestial Hideout"));
        assert!(is_hideout("My Custom Hideout"));
        assert!(!is_hideout("Strand"));
    }

    #[test]
    fn town_detection() {
        assert!(is_town("Oriath"));
        assert!(is_town("Highgate"));
        assert!(!is_town("Strand"));
    }

    #[test]
    fn idle_zone_detection() {
        assert!(is_idle_zone(None, "Hideout"));
        assert!(is_idle_zone(None, "Oriath"));
        assert!(!is_idle_zone(None, "Strand"));
    }

    #[test]
    fn hub_areas_are_idle_not_maps() {
        // Regression: these were previously counted as map runs.
        assert!(is_idle_zone(None, "Kingsmarch"));
        assert!(is_idle_zone(None, "The Rogue Harbour"));
        assert!(is_idle_zone(None, "Azurite Mine"));
        assert_eq!(classify(None, "Azurite Mine"), AreaType::Hub);
    }

    #[test]
    fn classify_prefers_internal_id() {
        assert_eq!(classify(Some("MapWorldsStrand"), "Strand"), AreaType::Map);
        assert_eq!(
            classify(Some("HideoutWorldTurtle"), "Cosmic Turtle Hideout"),
            AreaType::Hideout
        );
        assert_eq!(classify(Some("1_1_town"), "Lioneye's Watch"), AreaType::Town);
        assert_eq!(classify(Some("2_11_endgame_town"), "Oriath"), AreaType::Town);
        assert_eq!(
            classify(Some("KalguuranSettlersLeague"), "Kingsmarch"),
            AreaType::Hub
        );
        // A map by id is not an idle zone.
        assert!(!is_idle_zone(Some("MapWorldsStrand"), "Strand"));
    }

    #[test]
    fn tier_mapping() {
        assert_eq!(map_tier(68), Some(1));
        assert_eq!(map_tier(83), Some(16));
        assert_eq!(map_tier(84), Some(17));
        assert_eq!(map_tier(100), Some(17)); // clamped
        assert_eq!(map_tier(67), None);
    }
}
