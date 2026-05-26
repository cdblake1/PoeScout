//! End-to-end replay: feed a hand-authored Client.txt excerpt through the
//! parser + state machine and assert the resulting runs. Exercises area-id
//! typing/tier, NPC-encounter detection, death attribution, instance-resume
//! after a town trip, and run completion on entering a different map.

use chrono::NaiveDate;
use poe_maps::parser::parse_line;
use poe_maps::state::{StateEvent, StateMachine};

const LOG: &str = include_str!("fixtures/sample_client.txt");

#[test]
fn replay_sample_log() {
    let now = NaiveDate::from_ymd_opt(2025, 5, 20)
        .unwrap()
        .and_hms_opt(14, 0, 0)
        .unwrap();
    let mut sm = StateMachine::new(now);

    let mut completed = Vec::new();
    for line in LOG.lines() {
        if let Some(ev) = parse_line(line.trim()) {
            for se in sm.process(ev) {
                if let StateEvent::MapCompleted(run) = se {
                    completed.push(run);
                }
            }
        }
    }

    // Strand completes when we enter Atoll (a different map). The hideout trip
    // in between resumes rather than splitting the run.
    assert_eq!(completed.len(), 1, "exactly one run completed during replay");
    let strand = &completed[0];
    assert_eq!(strand.map_name, "Strand");
    assert_eq!(strand.area_type.as_deref(), Some("map"));
    assert_eq!(strand.map_tier, Some(16));
    assert_eq!(strand.deaths, 1, "Hero death attributed (no character filter)");
    assert!(
        strand.encounters.iter().any(|e| e.category == "Delve"),
        "Niko line recorded as a Delve encounter"
    );
    assert!(
        strand.hideout_secs >= 50.0,
        "town-trip idle attributed: {}",
        strand.hideout_secs
    );

    // The in-progress Atoll run finalizes cleanly.
    let end = NaiveDate::from_ymd_opt(2025, 5, 20)
        .unwrap()
        .and_hms_opt(14, 8, 0)
        .unwrap();
    let atoll = sm.finalize_current(end).unwrap();
    assert_eq!(atoll.map_name, "Atoll");
    assert_eq!(atoll.map_tier, Some(14));
}
