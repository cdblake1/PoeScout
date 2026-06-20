//! League-mechanic encounter detection from NPC dialogue.
//!
//! Data lives in `data/encounters.json` (embedded at compile time). `by_npc`
//! gives title-agnostic presence detection (matched on the NPC's first name);
//! `by_quote` gives fine-grained detail for specific dialogue lines. Encounters
//! are stored as raw rows on a run; counts and start/finish pairing are derived
//! on read.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize)]
pub struct EncounterDef {
    pub category: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EncountersFile {
    #[serde(default)]
    by_npc: HashMap<String, EncounterDef>,
    #[serde(default)]
    by_quote: HashMap<String, EncounterDef>,
}

static TABLE: LazyLock<EncountersFile> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/encounters.json"))
        .expect("data/encounters.json must be valid")
});

/// Match an NPC dialogue line to an encounter definition.
/// Returns `(def, specific)` where `specific` is true for an exact `by_quote`
/// match (distinct event, always recorded) and false for a `by_npc` presence
/// match (recorded once per category per run by the caller).
pub fn match_encounter(npc: &str, text: &str) -> Option<(EncounterDef, bool)> {
    if let Some(d) = TABLE.by_quote.get(text) {
        return Some((d.clone(), true));
    }
    let name = npc.split(',').next().unwrap_or(npc).trim();
    TABLE.by_npc.get(name).map(|d| (d.clone(), false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_npc_first_name_title_agnostic() {
        // 3.28 changed Tujen's title to "the Harbourmaster" — first-name match still works.
        let (def, specific) =
            match_encounter("Tujen, the Harbourmaster", "Got some goods in from the Karui.")
                .unwrap();
        assert_eq!(def.category, "Expedition");
        assert!(!specific);
    }

    #[test]
    fn matches_specific_quote_with_detail() {
        let (def, specific) =
            match_encounter("Einhar, Beastmaster", "Haha! You are captured, stupid beast.")
                .unwrap();
        assert_eq!(def.category, "Bestiary");
        assert_eq!(def.detail.as_deref(), Some("yellow"));
        assert!(specific);
    }

    #[test]
    fn matches_heist_rogue_by_first_name() {
        let (def, specific) =
            match_encounter("Karst, the Lockpick", "I'll get that lock open.").unwrap();
        assert_eq!(def.category, "Heist");
        assert!(!specific);
    }

    #[test]
    fn matches_sanctum_and_ancestor_npcs() {
        let (sanctum, _) =
            match_encounter("Lycia, Unholy Heretic", "None are innocent.").unwrap();
        assert_eq!(sanctum.category, "Sanctum");
        let (ancestor, _) = match_encounter("Navali", "The Trial continues!").unwrap();
        assert_eq!(ancestor.category, "Ancestor");
        let (ultimatum, _) =
            match_encounter("The Trialmaster", "A battlefield chilled by winter's hate.").unwrap();
        assert_eq!(ultimatum.category, "Ultimatum");
    }

    #[test]
    fn red_beast_capture_quote_has_detail() {
        let (def, specific) = match_encounter(
            "Einhar, Beastmaster",
            "Great job, Exile! Einhar will take the captured beast to the Menagerie.",
        )
        .unwrap();
        assert_eq!(def.category, "Bestiary");
        assert_eq!(def.kind.as_deref(), Some("capture"));
        assert_eq!(def.detail.as_deref(), Some("red"));
        assert!(specific);
    }

    #[test]
    fn maven_witness_dialogue_no_longer_tags() {
        // The Maven / The Envoy narrate map bosses during normal atlas play; their
        // presence must NOT be tagged as a mechanic (genuine Maven = boss arena).
        assert!(match_encounter("The Maven", "Violence...").is_none());
        assert!(match_encounter("The Envoy", "I followed her though I did not want to.").is_none());
    }

    #[test]
    fn no_match_for_player_chat() {
        assert!(match_encounter("RandomPlayer", "wtb 6l chest 50c").is_none());
    }

    #[test]
    fn falls_back_to_by_npc_for_unknown_quote() {
        // A known NPC saying an unrecognized line → presence match (not specific).
        let (def, specific) =
            match_encounter("Einhar, Beastmaster", "some unrecognized einhar banter").unwrap();
        assert_eq!(def.category, "Bestiary");
        assert!(!specific);
    }

    #[test]
    fn whisper_line_is_not_an_encounter() {
        // Whisper channel prefixes must not be treated as NPC dialogue.
        assert!(match_encounter("@From SomePlayer", "selling maps 1c").is_none());
        assert!(match_encounter("@To SomePlayer", "ty").is_none());
    }
}
