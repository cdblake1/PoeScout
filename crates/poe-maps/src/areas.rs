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

/// Special areas whose *entry* signals a league mechanic that emits no NPC line.
/// Each entry maps an exact area display name to `(category, detail)`. Entering
/// one records a `MapEncounter` on the current run (the parent map for sub-areas).
///
/// This is the area-based counterpart to the NPC-dialogue table in
/// `data/encounters.json`. Names are sourced from the detection catalog in
/// `docs/poe-mechanics-resources.md`; keep the two in sync each league.
static AREA_MECHANICS: &[(&str, &str, Option<&str>)] = &[
    // Legion — Timeless Conflict (5-splinter merged Domain entered from a map).
    ("Domain of Timeless Conflict", "Legion", Some("domain")),
    // Simulacrum — the wave areas (entered via Simulacrum/Splinters at the device).
    ("Lunacy's Watch", "Simulacrum", None),
    ("The Bridge Enraptured", "Simulacrum", None),
    ("The Syndrome Encampment", "Simulacrum", None),
    ("Hysteriagate", "Simulacrum", None),
    ("Oriath Delusion", "Simulacrum", None),
    // Breach — Breachstone Domains (one per Breachlord).
    ("Xoph's Domain", "Breach", Some("domain")),
    ("Tul's Domain", "Breach", Some("domain")),
    ("Esh's Domain", "Breach", Some("domain")),
    ("Uul-Netol's Domain", "Breach", Some("domain")),
    ("Chayula's Domain", "Breach", Some("domain")),
    // Sanctum — the Forbidden Sanctum floor areas.
    ("The Forbidden Sanctum", "Sanctum", None),
    ("Sanctum Archives", "Sanctum", None),
    ("Sanctum Cathedral", "Sanctum", None),
    ("Sanctum Necropolis", "Sanctum", None),
    ("Sanctum Vaults", "Sanctum", None),
    ("Sanctum Mausoleum", "Sanctum", None),
    // Incursion — the final Temple of Atzoatl run.
    ("The Temple of Atzoatl", "Temple", None),
    // Harvest / Ancestor / Abyss / Ultimatum — dedicated areas.
    ("The Sacred Grove", "Harvest", None),
    ("The Halls of the Dead", "Ancestor", None),
    ("Abyssal Depths", "Abyss", None),
    ("The Tower of Ordeals", "Ultimatum", Some("trialmaster")),
    // Expedition — Logbook areas (main expedition zones).
    ("Battleground Graves", "Expedition", Some("logbook")),
    ("Bluffs", "Expedition", Some("logbook")),
    ("Desert Ruins", "Expedition", Some("logbook")),
    ("Dried Riverbed", "Expedition", Some("logbook")),
    ("Forest Ruins", "Expedition", Some("logbook")),
    ("Karui Wargraves", "Expedition", Some("logbook")),
    ("Mountainside", "Expedition", Some("logbook")),
    ("Rotting Temple", "Expedition", Some("logbook")),
    ("Sarn Slums", "Expedition", Some("logbook")),
    ("Scrublands", "Expedition", Some("logbook")),
    ("Shipwreck Reef", "Expedition", Some("logbook")),
    ("Utzaal Outskirts", "Expedition", Some("logbook")),
    ("Volcanic Island", "Expedition", Some("logbook")),
    // Labyrinth — Trials of Ascendancy that appear inside maps.
    ("Trial of Piercing Truth", "Lab", Some("trial")),
    ("Trial of Swirling Fear", "Lab", Some("trial")),
    ("Trial of Crippling Grief", "Lab", Some("trial")),
    ("Trial of Burning Rage", "Lab", Some("trial")),
    ("Trial of Lingering Pain", "Lab", Some("trial")),
    ("Trial of Stinging Doubt", "Lab", Some("trial")),
    // Pinnacle boss arenas (apostrophe forms as they appear in Client.txt).
    ("Eye of the Storm", "Boss", Some("Sirus")),
    ("The Shaper's Realm", "Boss", Some("Shaper")),
    ("Absence of Value and Meaning", "Boss", Some("Elder")),
    ("Absence of Mercy and Empathy", "Boss", Some("Maven")),
    ("The Maven's Crucible", "Boss", Some("Maven")),
    ("Absence of Patience and Wisdom", "Boss", Some("Searing Exarch")),
    ("Absence of Symmetry and Harmony", "Boss", Some("Eater of Worlds")),
    ("Polaric Void", "Boss", Some("Black Star")),
    ("Seething Chyme", "Boss", Some("Infinite Hunger")),
    ("The Apex of Sacrifice", "Boss", Some("Atziri")),
    ("The Alluring Abyss", "Boss", Some("Uber Atziri")),
    ("Mastermind's Lair", "Boss", Some("Catarina")),
];

/// Detect a league mechanic from the area being entered, for mechanics that put
/// you in a dedicated area instead of printing an NPC line (Legion Domains,
/// Simulacrum, Breachstone Domains, …). Returns `(category, detail)`.
///
/// In-map Breach/Legion monoliths, Ritual, Metamorph, and Abyss cracks emit no
/// log signal at all and are intentionally absent — see the limitations section
/// of `docs/poe-mechanics-resources.md`.
pub fn mechanic_for_area(_area_id: Option<&str>, area_name: &str) -> Option<(String, Option<String>)> {
    AREA_MECHANICS
        .iter()
        .find(|(name, _, _)| *name == area_name)
        .map(|(_, category, detail)| (category.to_string(), detail.map(|d| d.to_string())))
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
    fn mechanic_for_special_areas() {
        assert_eq!(
            mechanic_for_area(None, "Domain of Timeless Conflict"),
            Some(("Legion".to_string(), Some("domain".to_string())))
        );
        assert_eq!(
            mechanic_for_area(None, "Oriath Delusion"),
            Some(("Simulacrum".to_string(), None))
        );
        assert_eq!(
            mechanic_for_area(None, "Xoph's Domain"),
            Some(("Breach".to_string(), Some("domain".to_string())))
        );
    }

    #[test]
    fn mechanic_for_expanded_areas() {
        assert_eq!(
            mechanic_for_area(None, "Sanctum Cathedral"),
            Some(("Sanctum".to_string(), None))
        );
        assert_eq!(
            mechanic_for_area(None, "The Temple of Atzoatl"),
            Some(("Temple".to_string(), None))
        );
        assert_eq!(
            mechanic_for_area(None, "Eye of the Storm"),
            Some(("Boss".to_string(), Some("Sirus".to_string())))
        );
        assert_eq!(
            mechanic_for_area(None, "Volcanic Island"),
            Some(("Expedition".to_string(), Some("logbook".to_string())))
        );
        assert_eq!(
            mechanic_for_area(None, "Trial of Burning Rage"),
            Some(("Lab".to_string(), Some("trial".to_string())))
        );
        // Corrected from the slice-1 guess: the 4th Simulacrum area is "Hysteriagate".
        assert_eq!(
            mechanic_for_area(None, "Hysteriagate"),
            Some(("Simulacrum".to_string(), None))
        );
    }

    #[test]
    fn mechanic_for_plain_map_is_none() {
        assert_eq!(mechanic_for_area(Some("MapWorldsStrand"), "Strand"), None);
        assert_eq!(mechanic_for_area(None, "Hideout"), None);
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
