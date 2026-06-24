use crate::areas::{classify, map_tier, mechanic_for_area, AreaType};
use crate::parser::LogEvent;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TrackerState {
    Stopped,
    Idle {
        since: String,
        zone_name: String,
    },
    InMap {
        map_name: String,
        area_level: Option<u32>,
        map_tier: Option<u32>,
        started_at: String,
        deaths: u32,
        /// Mechanics detected so far in the in-progress run (for the live row).
        #[serde(default)]
        encounters: Vec<MapEncounter>,
    },
}

/// A league-mechanic encounter detected during a run (raw row; counts and
/// start/finish pairing are derived on read).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapEncounter {
    pub category: String,
    #[serde(default)]
    pub detail: Option<String>,
    pub timestamp: String,
}

/// A timed sub-area entered inside a map (Vaal side area, Sanctum floor, lab
/// trial, Abyssal Depths, Legion Domain, …) — its own start/end/duration while
/// the parent map run continues (TraXile's nested-activity model).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubActivity {
    /// Mechanic category if known (e.g. "Vaal", "Sanctum", "Lab"), else "subarea".
    pub kind: String,
    pub name: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: f64,
}

/// A priced loot line for a run (from the per-map inventory diff, 6.3).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LootItem {
    pub name: String,
    pub type_line: String,
    pub stack_size: u32,
    pub unit_chaos: Option<f64>,
    pub total_chaos: Option<f64>,
    pub frame_type: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapRun {
    pub id: Option<i64>,
    pub map_name: String,
    #[serde(default)]
    pub area_id: Option<String>,
    pub area_level: Option<u32>,
    /// `AreaType::as_str()` (map/town/hideout/hub/…).
    #[serde(default)]
    pub area_type: Option<String>,
    #[serde(default)]
    pub map_tier: Option<u32>,
    /// Instance endpoint (ip:port) — used to resume a run after a town portal.
    #[serde(default)]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub league: Option<String>,
    #[serde(default)]
    pub session_id: Option<i64>,
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: f64,
    /// Idle seconds in town/hideout attributed to this run.
    #[serde(default)]
    pub hideout_secs: f64,
    pub deaths: u32,
    pub level_ups: Vec<u32>,
    #[serde(default)]
    pub encounters: Vec<MapEncounter>,
    /// Timed sub-areas entered inside this map (6.10).
    #[serde(default)]
    pub subactivities: Vec<SubActivity>,
    /// Total chaos value of loot from this run (set post-completion by the
    /// inventory-diff pricing in 6.3b; None until then).
    #[serde(default)]
    pub loot_chaos: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapStats {
    pub total_runs: u32,
    pub avg_duration_secs: f64,
    pub maps_per_hour: f64,
    pub total_deaths: u32,
}

/// A farming session: stash snapshot at start and end; profit = end − start;
/// currency/hour uses *active map time* (sum of run durations, idle excluded).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapSession {
    pub id: Option<i64>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub league: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub ended_at: Option<String>,
    #[serde(default)]
    pub start_chaos: Option<f64>,
    #[serde(default)]
    pub end_chaos: Option<f64>,
    #[serde(default)]
    pub profit_chaos: Option<f64>,
    /// Sum of run durations in the session (seconds, idle excluded).
    #[serde(default)]
    pub active_secs: f64,
    #[serde(default)]
    pub notes: Option<String>,
    /// Derived: number of runs linked to this session.
    #[serde(default)]
    pub run_count: u32,
    /// Derived: profit_chaos ÷ (active_secs / 3600).
    #[serde(default)]
    pub chaos_per_hour: Option<f64>,
}

/// Aggregated stats for one map type (grouped by internal area id, falling back
/// to display name).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapTypeStat {
    pub map_name: String,
    pub area_id: Option<String>,
    pub run_count: u32,
    pub avg_duration_secs: f64,
    pub avg_loot_chaos: Option<f64>,
    pub total_deaths: u32,
}

/// Aggregated stats for one league mechanic (grouped by encounter `category`
/// across all runs). `encounter_count` counts raw encounter rows (so repeated
/// detail events like beast captures add up); `maps_with` is the number of
/// distinct maps that contained the mechanic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MechanicStat {
    pub category: String,
    pub encounter_count: u32,
    pub maps_with: u32,
    /// `maps_with` as a percentage of all runs.
    pub pct_of_maps: f64,
    pub avg_duration_secs: f64,
    pub avg_loot_chaos: Option<f64>,
    pub total_deaths: u32,
}

/// A point-in-time stash valuation (recorded whenever a stash scan finalizes —
/// manual or via the auto-session start/end snapshot). Feeds the net-worth chart.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub id: Option<i64>,
    pub timestamp: String,
    pub total_chaos: f64,
    pub total_divine: f64,
}

/// Generic numeric resource time-series row (6.6). Source examples:
/// `ocr:hiveblood`, `ocr:kingsmarch_gold`, `experience`. Lets us record any
/// integer-valued game resource by key without a per-resource schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub id: Option<i64>,
    pub source: String,
    pub value: i64,
    pub timestamp: String,
}

/// Per-item-name aggregate over a chosen scope (Phase 6.7a). Drops, total chaos
/// value, and the *per-hour* rates (using active map time as the denominator).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemRate {
    pub name: String,
    /// Where this row came from. For 6.7a always `"inventory"`; later sources
    /// will be `"stash:bestiary"` (6.7b) and `"ocr:<key>"` (6.7c).
    pub source: String,
    /// Σ stack_size across all drops of this item in scope.
    pub stacks: u32,
    /// Number of distinct loot_items rows (independent of stack size).
    pub drops: u32,
    /// Σ total_chaos across all drops (NULL prices counted as 0).
    pub total_chaos: f64,
    /// Scope-wide sum of `map_runs.duration_secs` (idle excluded) — same value
    /// across every row in the response. Returned so callers don't have to
    /// recompute the rate themselves.
    pub active_secs: f64,
    /// `stacks / (active_secs / 3600)`. Zero when `active_secs == 0`.
    pub items_per_hour: f64,
    /// `total_chaos / (active_secs / 3600)`. Zero when `active_secs == 0`.
    pub chaos_per_hour: f64,
}

/// Scope picker for `get_items_per_hour`. `CurrentSession` falls back to
/// `AllTime` when no session is active. Wire-format is a serde-tagged enum so
/// the TS binding stays a discriminated union.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemRateScope {
    CurrentSession,
    Session { id: i64 },
    LastSessions { n: u32 },
    AllTime,
    /// Calendar-day range, inclusive of both ends. `start`/`end` are
    /// `YYYY-MM-DD` strings compared against the date prefix of `started_at`.
    DateRange { start: String, end: String },
}

impl Default for ItemRateScope {
    fn default() -> Self {
        Self::CurrentSession
    }
}

pub enum StateEvent {
    MapCompleted(MapRun),
    StateChanged(TrackerState),
    Death { map_name: String, total_deaths: u32 },
}

/// In-progress run accumulator (internal).
#[derive(Clone)]
struct RunAcc {
    map_name: String,
    area_id: Option<String>,
    area_level: Option<u32>,
    area_type: AreaType,
    map_tier: Option<u32>,
    instance_id: Option<String>,
    /// Per-instance map seed — the run-identity key (see `same_run`).
    seed: Option<u64>,
    started_at: NaiveDateTime,
    deaths: u32,
    level_ups: Vec<u32>,
    hideout_secs: f64,
    encounters: Vec<MapEncounter>,
    /// A sub-area currently open inside this map (timed until we leave it).
    current_sub: Option<SubAcc>,
    subactivities: Vec<SubActivity>,
}

/// An open (not-yet-closed) sub-area inside a map run.
#[derive(Clone)]
struct SubAcc {
    kind: String,
    name: String,
    started_at: NaiveDateTime,
}

impl RunAcc {
    /// Close the open sub-area (if any), recording its duration.
    fn close_sub(&mut self, ended_at: NaiveDateTime) {
        if let Some(sub) = self.current_sub.take() {
            let dur = (ended_at - sub.started_at).num_milliseconds().max(0) as f64 / 1000.0;
            self.subactivities.push(SubActivity {
                kind: sub.kind,
                name: sub.name,
                started_at: fmt(sub.started_at),
                ended_at: fmt(ended_at),
                duration_secs: dur,
            });
        }
    }

    /// Open a new sub-area, closing any previously-open one first.
    fn open_sub(&mut self, kind: String, name: String, started_at: NaiveDateTime) {
        self.close_sub(started_at);
        self.current_sub = Some(SubAcc {
            kind,
            name,
            started_at,
        });
    }

    fn to_map_run(&self, ended_at: NaiveDateTime, league: Option<String>) -> MapRun {
        let duration = (ended_at - self.started_at).num_milliseconds().max(0) as f64 / 1000.0;
        MapRun {
            id: None,
            map_name: self.map_name.clone(),
            area_id: self.area_id.clone(),
            area_level: self.area_level,
            area_type: Some(self.area_type.as_str().to_string()),
            map_tier: self.map_tier,
            instance_id: self.instance_id.clone(),
            league,
            session_id: None,
            started_at: fmt(self.started_at),
            ended_at: fmt(ended_at),
            duration_secs: duration,
            hideout_secs: self.hideout_secs,
            deaths: self.deaths,
            level_ups: self.level_ups.clone(),
            encounters: self.encounters.clone(),
            subactivities: self.subactivities.clone(),
            loot_chaos: None,
        }
    }
}

enum InnerState {
    Idle {
        /// When we became idle (town/hideout entry) — also the suspended run's end time.
        since: NaiveDateTime,
        zone_name: String,
        /// A run paused when we stepped into town/hideout; resumed if we return to the same instance.
        suspended: Option<RunAcc>,
    },
    InMap {
        run: RunAcc,
    },
}

pub struct StateMachine {
    inner: InnerState,
    pending_level: Option<u32>,
    pending_area_id: Option<String>,
    /// Seed from the most recent `Generating … with seed N`, applied to the next area.
    pending_seed: Option<u64>,
    /// Most recent `Connecting to instance server at` endpoint, applied to the next area.
    current_instance: Option<String>,
    league: Option<String>,
    /// Player's character name; when set, deaths/level-ups are attributed only to it.
    character: Option<String>,
    afk: bool,
}

fn fmt(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn is_idle_type(t: AreaType) -> bool {
    matches!(t, AreaType::Town | AreaType::Hideout | AreaType::Hub)
}

/// Count/outcome events (beast captures, Ultimatum rounds, boss kills) record
/// every occurrence; everything else is presence (deduped per category/detail).
fn is_count_kind(def: &crate::encounters::EncounterDef) -> bool {
    matches!(def.kind.as_deref(), Some("capture" | "count" | "outcome"))
}

/// Push an encounter onto a run, deduping presence per (category, detail) unless
/// `always`. Returns whether a row was added.
fn push_encounter(
    run: &mut RunAcc,
    def: crate::encounters::EncounterDef,
    always: bool,
    ts: NaiveDateTime,
) -> bool {
    let dup = !always
        && run
            .encounters
            .iter()
            .any(|e| e.category == def.category && e.detail == def.detail);
    if dup {
        return false;
    }
    run.encounters.push(MapEncounter {
        category: def.category,
        detail: def.detail,
        timestamp: fmt(ts),
    });
    true
}

/// Is this the same map instance (resume) or a different one (new run)?
///
/// Identity is the **seed**: distinct instances have distinct seeds. The
/// "instance server" endpoint is a shared gateway address, NOT per-instance, so
/// it must not be used — two different maps routed through the same gateway would
/// otherwise merge. When the incoming area carries no fresh seed (re-entering an
/// existing instance via a town portal logs no `Generating` line), fall back to
/// area identity to resume the suspended run.
fn same_run(run: &RunAcc, area_id: Option<&str>, area_name: &str, seed: Option<u64>) -> bool {
    match (run.seed, seed) {
        // Both freshly generated → same instance iff identical seed.
        (Some(a), Some(b)) => a == b,
        // No fresh seed (portal back to an existing instance) → same if same area.
        _ => match (run.area_id.as_deref(), area_id) {
            (Some(a), Some(b)) => a == b,
            _ => run.map_name == area_name,
        },
    }
}

impl StateMachine {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            inner: InnerState::Idle {
                since: now,
                zone_name: "Unknown".into(),
                suspended: None,
            },
            pending_level: None,
            pending_area_id: None,
            pending_seed: None,
            current_instance: None,
            league: None,
            character: None,
            afk: false,
        }
    }

    pub fn set_league(&mut self, league: Option<String>) {
        self.league = league;
    }

    pub fn set_character(&mut self, character: Option<String>) {
        self.character = character;
    }

    pub fn state(&self) -> TrackerState {
        match &self.inner {
            InnerState::Idle {
                since, zone_name, ..
            } => TrackerState::Idle {
                since: fmt(*since),
                zone_name: zone_name.clone(),
            },
            InnerState::InMap { run } => TrackerState::InMap {
                map_name: run.map_name.clone(),
                area_level: run.area_level,
                map_tier: run.map_tier,
                started_at: fmt(run.started_at),
                deaths: run.deaths,
                encounters: run.encounters.clone(),
            },
        }
    }

    fn new_run(
        &self,
        area_name: String,
        area_id: Option<String>,
        area_level: Option<u32>,
        atype: AreaType,
        endpoint: Option<String>,
        seed: Option<u64>,
        started_at: NaiveDateTime,
    ) -> RunAcc {
        let map_tier = if atype == AreaType::Map {
            area_level.and_then(map_tier)
        } else {
            None
        };
        RunAcc {
            map_name: area_name,
            area_id,
            area_level,
            area_type: atype,
            map_tier,
            instance_id: endpoint,
            seed,
            started_at,
            deaths: 0,
            level_ups: Vec::new(),
            hideout_secs: 0.0,
            encounters: Vec::new(),
            current_sub: None,
            subactivities: Vec::new(),
        }
    }

    fn is_player(&self, character: Option<&str>) -> bool {
        match (&self.character, character) {
            (Some(p), Some(c)) => p == c,
            (Some(_), None) => false,
            (None, _) => true, // no character configured → count all (solo-accurate)
        }
    }

    /// Force-complete the current or suspended run (called on session-end / tracker stop).
    pub fn finalize_current(&mut self, now: NaiveDateTime) -> Option<MapRun> {
        let prev = std::mem::replace(
            &mut self.inner,
            InnerState::Idle {
                since: now,
                zone_name: "Unknown".into(),
                suspended: None,
            },
        );
        match prev {
            InnerState::InMap { mut run } => {
                run.close_sub(now);
                Some(run.to_map_run(now, self.league.clone()))
            }
            InnerState::Idle {
                since,
                zone_name,
                suspended,
            } => {
                let completed = suspended.map(|mut run| {
                    run.close_sub(since);
                    run.to_map_run(since, self.league.clone())
                });
                self.inner = InnerState::Idle {
                    since,
                    zone_name,
                    suspended: None,
                };
                completed
            }
        }
    }

    pub fn process(&mut self, event: LogEvent) -> Vec<StateEvent> {
        let mut events = Vec::new();

        match event {
            LogEvent::AreaLevelHint {
                area_level,
                area_id,
                seed,
                ..
            } => {
                self.pending_level = Some(area_level);
                self.pending_area_id = Some(area_id);
                self.pending_seed = seed;
                // If already in a map whose level is unknown, backfill it.
                if let InnerState::InMap { ref mut run } = self.inner {
                    if run.area_level.is_none() {
                        run.area_level = Some(area_level);
                        if run.area_type == AreaType::Map && run.map_tier.is_none() {
                            run.map_tier = map_tier(area_level);
                        }
                        events.push(StateEvent::StateChanged(self.state()));
                    }
                }
            }
            LogEvent::InstanceConnected { endpoint, .. } => {
                self.current_instance = Some(endpoint);
            }
            LogEvent::Afk { on, .. } => {
                self.afk = on;
            }
            LogEvent::AreaChange {
                timestamp,
                area_name,
            } => {
                let area_id = self.pending_area_id.take();
                let area_level = self.pending_level.take();
                let seed = self.pending_seed.take();
                let atype = classify(area_id.as_deref(), &area_name);
                let endpoint = self.current_instance.clone();
                // Area-based mechanic (Legion Domain, Simulacrum, Breachstone, …):
                // computed before area_name/area_id are moved into the run below;
                // recorded on the resulting run once the lifecycle settles.
                let mechanic = mechanic_for_area(area_id.as_deref(), &area_name);
                // Sub-activity kind for a sub-area (the mechanic category, else "subarea").
                let sub_kind = mechanic
                    .as_ref()
                    .map(|(c, _)| c.clone())
                    .unwrap_or_else(|| "subarea".to_string());

                if is_idle_type(atype) {
                    // Entering town/hideout/hub: suspend the run (do NOT complete it yet).
                    let prev = std::mem::replace(
                        &mut self.inner,
                        InnerState::Idle {
                            since: timestamp,
                            zone_name: area_name.clone(),
                            suspended: None,
                        },
                    );
                    self.inner = match prev {
                        InnerState::InMap { mut run } => {
                            run.close_sub(timestamp); // close any open sub-area on town entry
                            InnerState::Idle {
                                since: timestamp,
                                zone_name: area_name,
                                suspended: Some(run),
                            }
                        }
                        // Already idle: keep the earliest `since` and the suspended run.
                        InnerState::Idle {
                            since, suspended, ..
                        } => InnerState::Idle {
                            since,
                            zone_name: area_name,
                            suspended,
                        },
                    };
                    events.push(StateEvent::StateChanged(self.state()));
                } else {
                    // Entering a non-idle area (a map, or a sub-area like Vaal/lab/abyss).
                    let prev = std::mem::replace(
                        &mut self.inner,
                        InnerState::Idle {
                            since: timestamp,
                            zone_name: String::new(),
                            suspended: None,
                        },
                    );
                    match prev {
                        InnerState::Idle {
                            since, suspended, ..
                        } => {
                            if let Some(mut run) = suspended {
                                if same_run(&run, area_id.as_deref(), &area_name, seed)
                                {
                                    // Returned to the same instance after a town trip → resume.
                                    run.hideout_secs +=
                                        (timestamp - since).num_milliseconds().max(0) as f64 / 1000.0;
                                    self.inner = InnerState::InMap { run };
                                } else {
                                    // Moved on to a different map → finalize the suspended run.
                                    events.push(StateEvent::MapCompleted(
                                        run.to_map_run(since, self.league.clone()),
                                    ));
                                    self.inner = InnerState::InMap {
                                        run: self.new_run(
                                            area_name, area_id, area_level, atype, endpoint, seed,
                                            timestamp,
                                        ),
                                    };
                                }
                            } else {
                                self.inner = InnerState::InMap {
                                    run: self.new_run(
                                        area_name, area_id, area_level, atype, endpoint, seed,
                                        timestamp,
                                    ),
                                };
                            }
                        }
                        InnerState::InMap { mut run } => {
                            if atype == AreaType::Map {
                                if same_run(&run, area_id.as_deref(), &area_name, seed)
                                {
                                    // Re-entered the same map (no town between) → ignore.
                                    run.close_sub(timestamp);
                                    self.inner = InnerState::InMap { run };
                                } else {
                                    run.close_sub(timestamp);
                                    events.push(StateEvent::MapCompleted(
                                        run.to_map_run(timestamp, self.league.clone()),
                                    ));
                                    self.inner = InnerState::InMap {
                                        run: self.new_run(
                                            area_name, area_id, area_level, atype, endpoint, seed,
                                            timestamp,
                                        ),
                                    };
                                }
                            } else if same_run(&run, area_id.as_deref(), &area_name, seed) {
                                // Returned to the parent map (no regen line, so it
                                // classifies as non-Map) → close the open sub-area.
                                run.close_sub(timestamp);
                                self.inner = InnerState::InMap { run };
                            } else {
                                // A genuinely different sub-area (Vaal/lab/Sanctum/…):
                                // stay in the run, open a timed sub-activity.
                                run.open_sub(sub_kind, area_name.clone(), timestamp);
                                self.inner = InnerState::InMap { run };
                            }
                        }
                    }
                    // Record an area-based mechanic on the now-current run (the parent
                    // map for sub-areas). Deduped per (category, detail) like NPC presence.
                    if let Some((category, detail)) = mechanic {
                        if let InnerState::InMap { ref mut run } = self.inner {
                            let dup = run
                                .encounters
                                .iter()
                                .any(|e| e.category == category && e.detail == detail);
                            if !dup {
                                run.encounters.push(MapEncounter {
                                    category,
                                    detail,
                                    timestamp: fmt(timestamp),
                                });
                            }
                        }
                    }
                    events.push(StateEvent::StateChanged(self.state()));
                }
            }
            LogEvent::Death { character, .. } => {
                if self.is_player(character.as_deref()) {
                    if let InnerState::InMap { ref mut run } = self.inner {
                        run.deaths += 1;
                        events.push(StateEvent::Death {
                            map_name: run.map_name.clone(),
                            total_deaths: run.deaths,
                        });
                        events.push(StateEvent::StateChanged(self.state()));
                    }
                }
            }
            LogEvent::LevelUp {
                level, character, ..
            } => {
                if self.is_player(character.as_deref()) {
                    if let InnerState::InMap { ref mut run } = self.inner {
                        run.level_ups.push(level);
                        events.push(StateEvent::StateChanged(self.state()));
                    }
                }
            }
            LogEvent::NpcLine {
                npc,
                text,
                timestamp,
            } => {
                let mut added = false;
                if let InnerState::InMap { ref mut run } = self.inner {
                    // Structured NPC dispatch: by_quote (exact, always) / by_npc (presence).
                    if let Some((def, specific)) = crate::encounters::match_encounter(&npc, &text) {
                        added |= push_encounter(run, def, specific, timestamp);
                    }
                    // Whole-line substring signals can also live inside NPC dialogue
                    // (e.g. "So be it. Keep your precious sanity" → Simulacrum clear).
                    for def in crate::encounters::match_line(&text) {
                        let always = is_count_kind(&def);
                        added |= push_encounter(run, def, always, timestamp);
                    }
                }
                if added {
                    events.push(StateEvent::StateChanged(self.state()));
                }
            }
            LogEvent::SystemLine { text, timestamp } => {
                // System / tagged lines (Mirage, Nameless Seer, Reflecting Mist, …)
                // matched by substring against `by_line`.
                let mut added = false;
                if let InnerState::InMap { ref mut run } = self.inner {
                    for def in crate::encounters::match_line(&text) {
                        let always = is_count_kind(&def);
                        added |= push_encounter(run, def, always, timestamp);
                    }
                }
                if added {
                    events.push(StateEvent::StateChanged(self.state()));
                }
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn ts(total_secs: u32) -> NaiveDateTime {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        NaiveDate::from_ymd_opt(2025, 5, 20)
            .unwrap()
            .and_hms_opt(14, mins, secs)
            .unwrap()
    }

    /// Enter a map with an explicit instance endpoint (full sequence: connect → generate → enter).
    fn enter_map_inst(
        sm: &mut StateMachine,
        time: u32,
        name: &str,
        level: u32,
        endpoint: &str,
    ) -> Vec<StateEvent> {
        sm.process(LogEvent::InstanceConnected {
            timestamp: ts(time),
            endpoint: endpoint.into(),
        });
        sm.process(LogEvent::AreaLevelHint {
            timestamp: ts(time),
            area_level: level,
            area_id: format!("MapWorlds{name}"),
            // Distinct instances get distinct seeds; key it off `time` so each
            // fresh map entry in a test is a unique instance.
            seed: Some(time as u64),
        });
        sm.process(LogEvent::AreaChange {
            timestamp: ts(time + 1),
            area_name: name.into(),
        })
    }

    fn enter_map(sm: &mut StateMachine, time: u32, name: &str, level: u32) -> Vec<StateEvent> {
        enter_map_inst(sm, time, name, level, &format!("10.0.0.{time}:6112"))
    }

    fn enter_hideout(sm: &mut StateMachine, time: u32, endpoint: &str) -> Vec<StateEvent> {
        sm.process(LogEvent::InstanceConnected {
            timestamp: ts(time),
            endpoint: endpoint.into(),
        });
        sm.process(LogEvent::AreaChange {
            timestamp: ts(time),
            area_name: "Hideout".into(),
        })
    }

    fn completed(evts: &[StateEvent]) -> Vec<&MapRun> {
        evts.iter()
            .filter_map(|e| match e {
                StateEvent::MapCompleted(run) => Some(run),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn enter_map_sets_type_and_tier() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        match sm.state() {
            TrackerState::InMap {
                area_level,
                map_tier,
                ..
            } => {
                assert_eq!(area_level, Some(83));
                assert_eq!(map_tier, Some(16));
            }
            _ => panic!("expected InMap"),
        }
    }

    #[test]
    fn entering_hideout_suspends_not_completes() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        let evts = enter_hideout(&mut sm, 120, "10.0.0.250:6112");
        assert!(completed(&evts).is_empty(), "town entry must not complete a run");
        assert!(matches!(sm.state(), TrackerState::Idle { .. }));
    }

    #[test]
    fn town_portal_resumes_same_instance() {
        let mut sm = StateMachine::new(ts(0));
        enter_map_inst(&mut sm, 10, "Strand", 83, "1.1.1.1:6112");
        enter_hideout(&mut sm, 120, "9.9.9.9:6112");
        // Reconnect to the ORIGINAL map instance, then re-enter Strand → resume (no completion).
        sm.process(LogEvent::InstanceConnected {
            timestamp: ts(180),
            endpoint: "1.1.1.1:6112".into(),
        });
        let evts = sm.process(LogEvent::AreaChange {
            timestamp: ts(181),
            area_name: "Strand".into(),
        });
        assert!(completed(&evts).is_empty(), "resume must not complete a run");
        assert!(matches!(sm.state(), TrackerState::InMap { .. }));

        let run = sm.finalize_current(ts(241)).unwrap();
        assert!(
            run.hideout_secs >= 59.0,
            "idle time should be attributed: {}",
            run.hideout_secs
        );
    }

    #[test]
    fn new_map_after_town_completes_previous() {
        let mut sm = StateMachine::new(ts(0));
        enter_map_inst(&mut sm, 10, "Strand", 83, "1.1.1.1:6112");
        enter_hideout(&mut sm, 120, "9.9.9.9:6112");
        let evts = enter_map_inst(&mut sm, 180, "Atoll", 81, "2.2.2.2:6112");
        let done = completed(&evts);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].map_name, "Strand");
        assert_eq!(done[0].area_type.as_deref(), Some("map"));
        assert!(matches!(sm.state(), TrackerState::InMap { map_name, .. } if map_name == "Atoll"));
    }

    #[test]
    fn distinct_instances_same_map_do_not_merge() {
        // Regression: two different Canyon maps share the SAME gateway endpoint
        // (the "instance server" address is not per-instance), with a town trip
        // between. They must be two runs — only the seed distinguishes instances.
        let mut sm = StateMachine::new(ts(0));
        let gw = "64.87.41.225:6112";
        enter_map_inst(&mut sm, 10, "Canyon", 83, gw); // seed 10
        enter_hideout(&mut sm, 120, gw); // town trip on the same gateway
        let evts = enter_map_inst(&mut sm, 180, "Canyon", 83, gw); // seed 180 → new instance
        let done = completed(&evts);
        assert_eq!(done.len(), 1, "first Canyon must finalize, not merge");
        assert_eq!(done[0].map_name, "Canyon");
        assert!(matches!(sm.state(), TrackerState::InMap { map_name, .. } if map_name == "Canyon"));
    }

    #[test]
    fn direct_map_to_map_completes_previous() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        let evts = enter_map(&mut sm, 60, "Atoll", 81);
        let done = completed(&evts);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].map_name, "Strand");
    }

    #[test]
    fn same_map_reentry_ignored() {
        let mut sm = StateMachine::new(ts(0));
        enter_map_inst(&mut sm, 10, "Strand", 83, "1.1.1.1:6112");
        let evts = sm.process(LogEvent::AreaChange {
            timestamp: ts(30),
            area_name: "Strand".into(),
        });
        assert!(completed(&evts).is_empty());
        assert!(matches!(sm.state(), TrackerState::InMap { .. }));
    }

    #[test]
    fn subarea_does_not_split_run() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        // A Vaal side area: generated with a non-map id → classified non-Map → stays in run.
        sm.process(LogEvent::AreaLevelHint {
            timestamp: ts(40),
            area_level: 83,
            area_id: "VaalCity".into(),
            seed: Some(40),
        });
        let evts = sm.process(LogEvent::AreaChange {
            timestamp: ts(41),
            area_name: "Vaal City".into(),
        });
        assert!(completed(&evts).is_empty());
        assert!(matches!(sm.state(), TrackerState::InMap { map_name, .. } if map_name == "Strand"));
    }

    #[test]
    fn vaal_subarea_records_timed_subactivity() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83); // seed 10
        // Enter a Vaal side area (its own seed, non-map id).
        sm.process(LogEvent::AreaLevelHint {
            timestamp: ts(40),
            area_level: 83,
            area_id: "VaalCity".into(),
            seed: Some(999),
        });
        sm.process(LogEvent::AreaChange {
            timestamp: ts(41),
            area_name: "Side Chapel".into(),
        });
        // Return to the parent map (no Generating line → no seed) → close the sub.
        sm.process(LogEvent::AreaChange {
            timestamp: ts(101),
            area_name: "Strand".into(),
        });
        let run = sm.finalize_current(ts(200)).unwrap();
        assert_eq!(run.map_name, "Strand");
        assert_eq!(run.subactivities.len(), 1);
        let sub = &run.subactivities[0];
        assert_eq!(sub.kind, "Vaal"); // "Side Chapel" is a Vaal area in AREA_MECHANICS
        assert_eq!(sub.name, "Side Chapel");
        assert!(sub.duration_secs >= 59.0, "sub duration = {}", sub.duration_secs);
    }

    #[test]
    fn hub_area_is_not_a_run() {
        let mut sm = StateMachine::new(ts(0));
        // Azurite Mine (Delve hub) must be idle, not a map run.
        sm.process(LogEvent::AreaChange {
            timestamp: ts(10),
            area_name: "Azurite Mine".into(),
        });
        assert!(matches!(sm.state(), TrackerState::Idle { .. }));
    }

    #[test]
    fn death_increments_and_attribution() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);

        let evts = sm.process(LogEvent::Death {
            timestamp: ts(30),
            character: Some("Me".into()),
        });
        assert!(evts
            .iter()
            .any(|e| matches!(e, StateEvent::Death { total_deaths: 1, .. })));

        // With a configured character, other players' deaths are ignored.
        sm.set_character(Some("Me".into()));
        let evts = sm.process(LogEvent::Death {
            timestamp: ts(40),
            character: Some("SomeoneElse".into()),
        });
        assert!(!evts.iter().any(|e| matches!(e, StateEvent::Death { .. })));

        let evts = sm.process(LogEvent::Death {
            timestamp: ts(45),
            character: Some("Me".into()),
        });
        assert!(evts
            .iter()
            .any(|e| matches!(e, StateEvent::Death { total_deaths: 2, .. })));
    }

    #[test]
    fn finalize_current_completes_open_run() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        let run = sm.finalize_current(ts(130)).unwrap();
        assert_eq!(run.map_name, "Strand");
        assert!(run.duration_secs > 0.0);
        assert!(matches!(sm.state(), TrackerState::Idle { .. }));
    }

    #[test]
    fn npc_line_records_encounter_presence_once() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        // Two Niko lines in the same map → one Delve presence row.
        sm.process(LogEvent::NpcLine {
            timestamp: ts(20),
            npc: "Niko, Master of the Depths".into(),
            text: "Food for the machine, heheh!".into(),
        });
        sm.process(LogEvent::NpcLine {
            timestamp: ts(25),
            npc: "Niko, Master of the Depths".into(),
            text: "Plenty more sulphite where that came from.".into(),
        });
        // A specific beast-capture quote → its own detail row.
        sm.process(LogEvent::NpcLine {
            timestamp: ts(30),
            npc: "Einhar, Beastmaster".into(),
            text: "Haha! You are captured, stupid beast.".into(),
        });
        let run = sm.finalize_current(ts(130)).unwrap();
        let delve = run.encounters.iter().filter(|e| e.category == "Delve").count();
        assert_eq!(delve, 1, "presence recorded once per category");
        assert!(run
            .encounters
            .iter()
            .any(|e| e.category == "Bestiary" && e.detail.as_deref() == Some("yellow")));
    }

    #[test]
    fn legion_domain_tags_parent_run() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        // Enter the Legion Domain from inside the map (no level hint → sub-area).
        sm.process(LogEvent::AreaChange {
            timestamp: ts(60),
            area_name: "Domain of Timeless Conflict".into(),
        });
        let run = sm.finalize_current(ts(180)).unwrap();
        assert_eq!(run.map_name, "Strand", "stays the parent map");
        assert!(run
            .encounters
            .iter()
            .any(|e| e.category == "Legion" && e.detail.as_deref() == Some("domain")));
    }

    #[test]
    fn simulacrum_from_hideout_is_tagged_run() {
        let mut sm = StateMachine::new(ts(0));
        // Open a Simulacrum from the hideout device → its own run, tagged.
        sm.process(LogEvent::AreaChange {
            timestamp: ts(10),
            area_name: "Oriath Delusion".into(),
        });
        assert!(matches!(sm.state(), TrackerState::InMap { .. }));
        let run = sm.finalize_current(ts(400)).unwrap();
        assert!(run.encounters.iter().any(|e| e.category == "Simulacrum"));
    }

    #[test]
    fn plain_map_records_no_area_mechanic() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        let run = sm.finalize_current(ts(130)).unwrap();
        assert!(run.encounters.is_empty(), "no spurious mechanic on a plain map");
    }

    #[test]
    fn domain_reentry_deduped() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        for t in [60, 90] {
            sm.process(LogEvent::AreaChange {
                timestamp: ts(t),
                area_name: "Domain of Timeless Conflict".into(),
            });
            // Bounce back to the map between Domain entries (same instance → resume).
            sm.process(LogEvent::AreaChange {
                timestamp: ts(t + 5),
                area_name: "Strand".into(),
            });
        }
        let run = sm.finalize_current(ts(200)).unwrap();
        let legion = run.encounters.iter().filter(|e| e.category == "Legion").count();
        assert_eq!(legion, 1, "re-entering the Domain must not double-count");
    }

    #[test]
    fn system_line_tags_mirage_and_seer() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        sm.process(LogEvent::SystemLine {
            timestamp: ts(20),
            text: "[Faridun] Blocking terrain outside mirage area".into(),
        });
        sm.process(LogEvent::SystemLine {
            timestamp: ts(25),
            text: ": The Nameless Seer has appeared nearby.".into(),
        });
        let run = sm.finalize_current(ts(130)).unwrap();
        assert!(run.encounters.iter().any(|e| e.category == "Mirage"));
        assert!(run
            .encounters
            .iter()
            .any(|e| e.category == "Delirium" && e.detail.as_deref() == Some("nameless_seer")));
    }

    #[test]
    fn substring_in_npc_dialogue_tags_simulacrum_clear() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        sm.process(LogEvent::NpcLine {
            timestamp: ts(40),
            npc: "Strange Voice".into(),
            text: "So be it. Keep your precious sanity, my agent of chaos.".into(),
        });
        let run = sm.finalize_current(ts(130)).unwrap();
        assert!(run.encounters.iter().any(|e| e.category == "Delirium")); // Strange Voice presence
        assert!(run.encounters.iter().any(|e| e.category == "Simulacrum")); // by_line fullclear
    }

    #[test]
    fn mirage_presence_deduped() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);
        for t in [20, 30, 40] {
            sm.process(LogEvent::SystemLine {
                timestamp: ts(t),
                text: "[Faridun] Blocking terrain outside mirage area".into(),
            });
        }
        let run = sm.finalize_current(ts(130)).unwrap();
        assert_eq!(run.encounters.iter().filter(|e| e.category == "Mirage").count(), 1);
    }

    #[test]
    fn level_up_attribution() {
        let mut sm = StateMachine::new(ts(0));
        sm.set_character(Some("Me".into()));
        enter_map(&mut sm, 10, "Strand", 83);
        sm.process(LogEvent::LevelUp {
            timestamp: ts(20),
            level: 90,
            character: Some("Someone".into()),
        });
        sm.process(LogEvent::LevelUp {
            timestamp: ts(25),
            level: 91,
            character: Some("Me".into()),
        });
        let run = sm.finalize_current(ts(130)).unwrap();
        assert_eq!(run.level_ups, vec![91]);
    }

    #[test]
    fn hideout_secs_accumulates_across_town_hops() {
        let mut sm = StateMachine::new(ts(0));
        enter_map_inst(&mut sm, 10, "Strand", 83, "1.1.1.1:6112");
        enter_hideout(&mut sm, 100, "9.9.9.9:6112"); // suspend; idle since 100
        enter_hideout(&mut sm, 150, "8.8.8.8:6112"); // town→town; keep idle since 100
        // Reconnect the ORIGINAL instance and return → resume.
        sm.process(LogEvent::InstanceConnected {
            timestamp: ts(200),
            endpoint: "1.1.1.1:6112".into(),
        });
        sm.process(LogEvent::AreaChange {
            timestamp: ts(201),
            area_name: "Strand".into(),
        });
        let run = sm.finalize_current(ts(260)).unwrap();
        // Idle from 100 → 201 (~101s) is attributed to the run.
        assert!(run.hideout_secs >= 100.0, "hideout_secs = {}", run.hideout_secs);
    }

    #[test]
    fn finalize_suspended_run_ends_at_suspend_time() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83); // started_at = ts(11)
        enter_hideout(&mut sm, 130, "9.9.9.9:6112"); // suspend at 130
        let run = sm.finalize_current(ts(300)).unwrap(); // finalize much later
        assert_eq!(run.map_name, "Strand");
        // Duration runs to the suspend time (~119s), not to ts(300).
        assert!(run.duration_secs < 130.0, "duration = {}", run.duration_secs);
        assert!(run.duration_secs >= 118.0, "duration = {}", run.duration_secs);
    }

    #[test]
    fn league_stamped_on_completed_run() {
        let mut sm = StateMachine::new(ts(0));
        sm.set_league(Some("Mirage".into()));
        enter_map(&mut sm, 10, "Strand", 83);
        let run = sm.finalize_current(ts(130)).unwrap();
        assert_eq!(run.league.as_deref(), Some("Mirage"));
    }
}
