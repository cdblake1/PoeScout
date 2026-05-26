use chrono::NaiveDateTime;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq)]
pub enum LogEvent {
    AreaChange {
        timestamp: NaiveDateTime,
        area_name: String,
    },
    /// Emitted from the `Generating level N area "Id"` line, which precedes the
    /// matching `You have entered` line. Carries both the area level and the
    /// internal area id (e.g. `MapWorldsStrand`) — the canonical map identity.
    AreaLevelHint {
        timestamp: NaiveDateTime,
        area_level: u32,
        area_id: String,
    },
    /// `Connecting to instance server at <ip>:<port>` — the instance endpoint,
    /// used to resume the same run after a town portal instead of starting a new one.
    InstanceConnected {
        timestamp: NaiveDateTime,
        endpoint: String,
    },
    /// `AFK mode is now ON/OFF` — used to pause idle accounting.
    Afk {
        timestamp: NaiveDateTime,
        on: bool,
    },
    Death {
        timestamp: NaiveDateTime,
        /// Character named in the slain line, for attribution (party members also appear).
        character: Option<String>,
    },
    LevelUp {
        timestamp: NaiveDateTime,
        level: u32,
        /// Character named in the level-up line, for attribution.
        character: Option<String>,
    },
    /// A chat-channel NPC dialogue line `] NPC, Title: quote`, fed to the
    /// league-mechanic encounter dispatcher.
    NpcLine {
        timestamp: NaiveDateTime,
        npc: String,
        text: String,
    },
}

static RE_TIMESTAMP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2})").unwrap());

static RE_GENERATING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"Generating level (\d+) area "([^"]+)""#).unwrap());

// Anchored to the `] : ` system-message prefix so NPC dialogue can't false-match.
static RE_ENTERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\] : You have entered (.+?)\.\s*$").unwrap());

static RE_INSTANCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Connecting to instance server at (\S+)").unwrap());

static RE_AFK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"AFK mode is now (ON|OFF)").unwrap());

static RE_DEATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\] : (\S+) has been slain").unwrap());

static RE_LEVEL_UP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\] : (\S+) (?:\([^)]+\) )?is now level (\d+)").unwrap());

// NPC dialogue: after the `[... Client n]` bracket, `Name: text` (colon + space).
// `] : ` system lines can't match (the char after `] ` is the colon).
static RE_NPC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\] ([^:\]]+): (.+?)\s*$").unwrap());

pub fn parse_line(line: &str) -> Option<LogEvent> {
    let ts = parse_timestamp(line)?;

    if let Some(caps) = RE_GENERATING.captures(line) {
        let level: u32 = caps[1].parse().ok()?;
        return Some(LogEvent::AreaLevelHint {
            timestamp: ts,
            area_level: level,
            area_id: caps[2].to_string(),
        });
    }

    if let Some(caps) = RE_ENTERED.captures(line) {
        return Some(LogEvent::AreaChange {
            timestamp: ts,
            area_name: caps[1].to_string(),
        });
    }

    if let Some(caps) = RE_INSTANCE.captures(line) {
        return Some(LogEvent::InstanceConnected {
            timestamp: ts,
            endpoint: caps[1].to_string(),
        });
    }

    if let Some(caps) = RE_AFK.captures(line) {
        return Some(LogEvent::Afk {
            timestamp: ts,
            on: &caps[1] == "ON",
        });
    }

    if let Some(caps) = RE_DEATH.captures(line) {
        return Some(LogEvent::Death {
            timestamp: ts,
            character: Some(caps[1].to_string()),
        });
    }

    if let Some(caps) = RE_LEVEL_UP.captures(line) {
        let character = Some(caps[1].to_string());
        let level: u32 = caps[2].parse().ok()?;
        return Some(LogEvent::LevelUp {
            timestamp: ts,
            level,
            character,
        });
    }

    if let Some(caps) = RE_NPC.captures(line) {
        return Some(LogEvent::NpcLine {
            timestamp: ts,
            npc: caps[1].to_string(),
            text: caps[2].to_string(),
        });
    }

    None
}

fn parse_timestamp(line: &str) -> Option<NaiveDateTime> {
    let caps = RE_TIMESTAMP.captures(line)?;
    NaiveDateTime::parse_from_str(&caps[1], "%Y/%m/%d %H:%M:%S").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_generating_area() {
        let line = r#"2025/05/20 14:30:15 123456 abc [DEBUG Client 1234] Generating level 83 area "MapWorldsStrand" with seed 12345"#;
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::AreaLevelHint {
                area_level,
                area_id,
                ..
            } => {
                assert_eq!(area_level, 83);
                assert_eq!(area_id, "MapWorldsStrand");
            }
            _ => panic!("expected AreaLevelHint"),
        }
    }

    #[test]
    fn parse_entered_area() {
        let line = "2025/05/20 14:30:16 123456 abc [INFO Client 1234] : You have entered Strand.";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::AreaChange { area_name, .. } => {
                assert_eq!(area_name, "Strand");
            }
            _ => panic!("expected AreaChange"),
        }
    }

    #[test]
    fn parse_entered_custom_hideout() {
        let line = "2025/05/20 14:30:16 123456 abc [INFO Client 1234] : You have entered Cosmic Turtle Hideout.";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::AreaChange { area_name, .. } => {
                assert_eq!(area_name, "Cosmic Turtle Hideout");
            }
            _ => panic!("expected AreaChange"),
        }
    }

    #[test]
    fn parse_instance_connected() {
        let line = "2025/05/20 14:30:14 123456 abc [INFO Client 1234] Connecting to instance server at 8.8.8.8:6112";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::InstanceConnected { endpoint, .. } => {
                assert_eq!(endpoint, "8.8.8.8:6112");
            }
            _ => panic!("expected InstanceConnected"),
        }
    }

    #[test]
    fn parse_afk_toggle() {
        let on = parse_line(
            r#"2025/05/20 14:30:14 1 a [INFO Client 1] : AFK mode is now ON. Autoreply "This player is AFK.""#,
        )
        .unwrap();
        assert!(matches!(on, LogEvent::Afk { on: true, .. }));

        let off =
            parse_line("2025/05/20 14:30:14 1 a [INFO Client 1] : AFK mode is now OFF.").unwrap();
        assert!(matches!(off, LogEvent::Afk { on: false, .. }));
    }

    #[test]
    fn parse_death_captures_character() {
        let line = "2025/05/20 14:31:00 123456 abc [INFO Client 1234] : PlayerName has been slain.";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::Death { character, .. } => {
                assert_eq!(character.as_deref(), Some("PlayerName"));
            }
            _ => panic!("expected Death"),
        }
    }

    #[test]
    fn parse_level_up_captures_character() {
        let line =
            "2025/05/20 14:32:00 123456 abc [INFO Client 1234] : PlayerName (Hierophant) is now level 95";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::LevelUp {
                level, character, ..
            } => {
                assert_eq!(level, 95);
                assert_eq!(character.as_deref(), Some("PlayerName"));
            }
            _ => panic!("expected LevelUp"),
        }
    }

    #[test]
    fn parse_npc_line() {
        let line = "2025/07/24 20:25:32 58414218 cff945b9 [INFO Client 18624] Einhar, Beastmaster: Exile! You are a welcome omen.";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::NpcLine { npc, text, .. } => {
                assert_eq!(npc, "Einhar, Beastmaster");
                assert_eq!(text, "Exile! You are a welcome omen.");
            }
            _ => panic!("expected NpcLine"),
        }
    }

    #[test]
    fn system_message_is_not_npc_line() {
        // `] : ...` lines must never be misread as NPC dialogue.
        let line = "2025/05/20 14:30:15 123456 abc [INFO Client 1234] : 3 Items identified";
        assert!(!matches!(parse_line(line), Some(LogEvent::NpcLine { .. })));
    }

    #[test]
    fn parse_irrelevant_line() {
        let line = "2025/05/20 14:30:15 123456 abc [INFO Client 1234] Some random log message";
        assert!(parse_line(line).is_none());
    }
}
