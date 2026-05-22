use crate::areas::is_idle_zone;
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
        started_at: String,
        deaths: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapRun {
    pub id: Option<i64>,
    pub map_name: String,
    pub area_level: Option<u32>,
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: f64,
    pub deaths: u32,
    pub level_ups: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapStats {
    pub total_runs: u32,
    pub avg_duration_secs: f64,
    pub maps_per_hour: f64,
    pub total_deaths: u32,
}

pub enum StateEvent {
    MapCompleted(MapRun),
    StateChanged(TrackerState),
    Death { map_name: String, total_deaths: u32 },
}

pub struct StateMachine {
    inner: InnerState,
    pending_level: Option<u32>,
}

enum InnerState {
    Idle {
        since: NaiveDateTime,
        zone_name: String,
    },
    InMap {
        map_name: String,
        area_level: Option<u32>,
        started_at: NaiveDateTime,
        deaths: u32,
        level_ups: Vec<u32>,
    },
}

impl StateMachine {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            inner: InnerState::Idle { since: now, zone_name: "Unknown".into() },
            pending_level: None,
        }
    }

    pub fn state(&self) -> TrackerState {
        match &self.inner {
            InnerState::Idle { since, zone_name } => TrackerState::Idle {
                since: since.format("%Y-%m-%dT%H:%M:%S").to_string(),
                zone_name: zone_name.clone(),
            },
            InnerState::InMap {
                map_name,
                area_level,
                started_at,
                deaths,
                ..
            } => TrackerState::InMap {
                map_name: map_name.clone(),
                area_level: *area_level,
                started_at: started_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                deaths: *deaths,
            },
        }
    }

    pub fn process(&mut self, event: LogEvent) -> Vec<StateEvent> {
        let mut events = Vec::new();

        match event {
            LogEvent::AreaLevelHint { area_level, .. } => {
                // Stash the level for the next AreaChange, or apply to current map
                match &mut self.inner {
                    InnerState::InMap { area_level: lvl, .. } if lvl.is_none() => {
                        *lvl = Some(area_level);
                        events.push(StateEvent::StateChanged(self.state()));
                    }
                    _ => {
                        self.pending_level = Some(area_level);
                    }
                }
            }
            LogEvent::AreaChange {
                timestamp,
                area_name,
            } => {
                let area_level = self.pending_level.take();

                if is_idle_zone(&area_name) {
                    if let InnerState::InMap {
                        ref map_name,
                        area_level,
                        started_at,
                        deaths,
                        ref level_ups,
                    } = self.inner
                    {
                        let duration = (timestamp - started_at).num_milliseconds() as f64 / 1000.0;
                        let run = MapRun {
                            id: None,
                            map_name: map_name.clone(),
                            area_level,
                            started_at: started_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                            ended_at: timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
                            duration_secs: duration,
                            deaths,
                            level_ups: level_ups.clone(),
                        };
                        events.push(StateEvent::MapCompleted(run));
                    }
                    self.inner = InnerState::Idle { since: timestamp, zone_name: area_name };
                } else {
                    if let InnerState::InMap {
                        ref map_name,
                        area_level: prev_level,
                        started_at,
                        deaths,
                        ref level_ups,
                    } = self.inner
                    {
                        if *map_name != area_name {
                            let duration =
                                (timestamp - started_at).num_milliseconds() as f64 / 1000.0;
                            let run = MapRun {
                                id: None,
                                map_name: map_name.clone(),
                                area_level: prev_level,
                                started_at: started_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                                ended_at: timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
                                duration_secs: duration,
                                deaths,
                                level_ups: level_ups.clone(),
                            };
                            events.push(StateEvent::MapCompleted(run));
                            self.inner = InnerState::InMap {
                                map_name: area_name,
                                area_level,
                                started_at: timestamp,
                                deaths: 0,
                                level_ups: Vec::new(),
                            };
                        }
                    } else {
                        self.inner = InnerState::InMap {
                            map_name: area_name,
                            area_level,
                            started_at: timestamp,
                            deaths: 0,
                            level_ups: Vec::new(),
                        };
                    }
                }
                events.push(StateEvent::StateChanged(self.state()));
            }
            LogEvent::Death { .. } => {
                if let InnerState::InMap {
                    ref map_name,
                    ref mut deaths,
                    ..
                } = self.inner
                {
                    *deaths += 1;
                    events.push(StateEvent::Death {
                        map_name: map_name.clone(),
                        total_deaths: *deaths,
                    });
                    events.push(StateEvent::StateChanged(self.state()));
                }
            }
            LogEvent::LevelUp { level, .. } => {
                if let InnerState::InMap {
                    ref mut level_ups, ..
                } = self.inner
                {
                    level_ups.push(level);
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

    fn enter_map(sm: &mut StateMachine, time: u32, name: &str, level: u32) -> Vec<StateEvent> {
        sm.process(LogEvent::AreaLevelHint {
            timestamp: ts(time),
            area_level: level,
        });
        sm.process(LogEvent::AreaChange {
            timestamp: ts(time + 1),
            area_name: name.into(),
        })
    }

    #[test]
    fn idle_to_map_to_idle() {
        let mut sm = StateMachine::new(ts(0));

        let evts = enter_map(&mut sm, 10, "Strand", 83);
        assert!(matches!(sm.state(), TrackerState::InMap { .. }));
        assert!(evts.iter().any(|e| matches!(e, StateEvent::StateChanged(_))));

        let evts = sm.process(LogEvent::AreaChange {
            timestamp: ts(120),
            area_name: "Hideout".into(),
        });
        assert!(matches!(sm.state(), TrackerState::Idle { .. }));
        let completed: Vec<_> = evts
            .iter()
            .filter_map(|e| match e {
                StateEvent::MapCompleted(run) => Some(run),
                _ => None,
            })
            .collect();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].map_name, "Strand");
        assert_eq!(completed[0].area_level, Some(83));
    }

    #[test]
    fn death_increments() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);

        let evts = sm.process(LogEvent::Death { timestamp: ts(30) });
        assert!(evts.iter().any(|e| matches!(e, StateEvent::Death { total_deaths: 1, .. })));

        let evts = sm.process(LogEvent::Death { timestamp: ts(40) });
        assert!(evts.iter().any(|e| matches!(e, StateEvent::Death { total_deaths: 2, .. })));
    }

    #[test]
    fn map_to_different_map() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);

        let evts = enter_map(&mut sm, 60, "Atoll", 81);

        let completed: Vec<_> = evts
            .iter()
            .filter_map(|e| match e {
                StateEvent::MapCompleted(run) => Some(run),
                _ => None,
            })
            .collect();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].map_name, "Strand");

        assert!(matches!(sm.state(), TrackerState::InMap { map_name, .. } if map_name == "Atoll"));
    }

    #[test]
    fn same_map_reentry_ignored() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);

        let evts = sm.process(LogEvent::AreaChange {
            timestamp: ts(30),
            area_name: "Strand".into(),
        });

        assert!(!evts.iter().any(|e| matches!(e, StateEvent::MapCompleted(_))));
        assert!(matches!(sm.state(), TrackerState::InMap { .. }));
    }

    #[test]
    fn area_level_hint_applied() {
        let mut sm = StateMachine::new(ts(0));
        enter_map(&mut sm, 10, "Strand", 83);

        match sm.state() {
            TrackerState::InMap { area_level, .. } => assert_eq!(area_level, Some(83)),
            _ => panic!("expected InMap"),
        }
    }
}
