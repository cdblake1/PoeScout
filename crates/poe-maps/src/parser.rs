use chrono::NaiveDateTime;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq)]
pub enum LogEvent {
    AreaChange {
        timestamp: NaiveDateTime,
        area_name: String,
    },
    AreaLevelHint {
        timestamp: NaiveDateTime,
        area_level: u32,
    },
    Death {
        timestamp: NaiveDateTime,
    },
    LevelUp {
        timestamp: NaiveDateTime,
        level: u32,
    },
}

static RE_TIMESTAMP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2})").unwrap());

static RE_GENERATING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"Generating level (\d+) area "([^"]+)""#).unwrap());

static RE_ENTERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You have entered (.+)\.\s*$").unwrap());

static RE_DEATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(has been slain|You have died)").unwrap());

static RE_LEVEL_UP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"is now level (\d+)").unwrap());

pub fn parse_line(line: &str) -> Option<LogEvent> {
    let ts = parse_timestamp(line)?;

    if let Some(caps) = RE_GENERATING.captures(line) {
        let level: u32 = caps[1].parse().ok()?;
        return Some(LogEvent::AreaLevelHint {
            timestamp: ts,
            area_level: level,
        });
    }

    if let Some(caps) = RE_ENTERED.captures(line) {
        let area = caps[1].to_string();
        return Some(LogEvent::AreaChange {
            timestamp: ts,
            area_name: area,
        });
    }

    if RE_DEATH.is_match(line) {
        return Some(LogEvent::Death { timestamp: ts });
    }

    if let Some(caps) = RE_LEVEL_UP.captures(line) {
        let level: u32 = caps[1].parse().ok()?;
        return Some(LogEvent::LevelUp {
            timestamp: ts,
            level,
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
        let line = r#"2025/05/20 14:30:15 123456 abc [INFO Client 1234] : Generating level 83 area "MapWorldsStrand""#;
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::AreaLevelHint { area_level, .. } => {
                assert_eq!(area_level, 83);
            }
            _ => panic!("expected AreaLevelHint"),
        }
    }

    #[test]
    fn parse_entered_area() {
        let line = "2025/05/20 14:30:16 123456 abc [INFO Client 1234] You have entered Strand.";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::AreaChange { area_name, .. } => {
                assert_eq!(area_name, "Strand");
            }
            _ => panic!("expected AreaChange"),
        }
    }

    #[test]
    fn parse_death() {
        let line = "2025/05/20 14:31:00 123456 abc [INFO Client 1234] PlayerName has been slain";
        let evt = parse_line(line).unwrap();
        assert!(matches!(evt, LogEvent::Death { .. }));
    }

    #[test]
    fn parse_level_up() {
        let line = "2025/05/20 14:32:00 123456 abc [INFO Client 1234] PlayerName is now level 95";
        let evt = parse_line(line).unwrap();
        match evt {
            LogEvent::LevelUp { level, .. } => assert_eq!(level, 95),
            _ => panic!("expected LevelUp"),
        }
    }

    #[test]
    fn parse_irrelevant_line() {
        let line = "2025/05/20 14:30:15 123456 abc [INFO Client 1234] Some random log message";
        assert!(parse_line(line).is_none());
    }
}
