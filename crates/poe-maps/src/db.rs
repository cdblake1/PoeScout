use crate::state::{MapRun, MapStats};
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct MapDb {
    conn: Mutex<Connection>,
}

impl MapDb {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS map_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                map_name TEXT NOT NULL,
                area_level INTEGER,
                started_at TEXT NOT NULL,
                ended_at TEXT NOT NULL,
                duration_secs REAL NOT NULL,
                deaths INTEGER NOT NULL DEFAULT 0,
                level_ups TEXT NOT NULL DEFAULT '[]',
                hideout_secs REAL NOT NULL DEFAULT 0.0
            );
            CREATE INDEX IF NOT EXISTS idx_map_runs_started ON map_runs(started_at DESC);",
        )?;
        Ok(())
    }

    pub fn insert_map_run(&self, run: &MapRun) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let level_ups_json = serde_json::to_string(&run.level_ups)?;
        conn.execute(
            "INSERT INTO map_runs (map_name, area_level, started_at, ended_at, duration_secs, deaths, level_ups)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                run.map_name,
                run.area_level,
                run.started_at,
                run.ended_at,
                run.duration_secs,
                run.deaths,
                level_ups_json,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<MapRun>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, map_name, area_level, started_at, ended_at, duration_secs, deaths, level_ups
             FROM map_runs ORDER BY started_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            let level_ups_str: String = row.get(7)?;
            let level_ups: Vec<u32> =
                serde_json::from_str(&level_ups_str).unwrap_or_default();
            Ok(MapRun {
                id: Some(row.get(0)?),
                map_name: row.get(1)?,
                area_level: row.get(2)?,
                started_at: row.get(3)?,
                ended_at: row.get(4)?,
                duration_secs: row.get(5)?,
                deaths: row.get(6)?,
                level_ups,
            })
        })?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        Ok(runs)
    }

    pub fn get_stats(&self) -> Result<MapStats> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), COALESCE(AVG(duration_secs), 0), COALESCE(SUM(deaths), 0),
                    MIN(started_at), MAX(ended_at)
             FROM map_runs",
        )?;
        let stats = stmt.query_row([], |row| {
            let total_runs: u32 = row.get(0)?;
            let avg_duration_secs: f64 = row.get(1)?;
            let total_deaths: u32 = row.get(2)?;
            let first_start: Option<String> = row.get(3)?;
            let last_end: Option<String> = row.get(4)?;

            let maps_per_hour = if let (Some(start), Some(end)) = (first_start, last_end) {
                if let (Ok(s), Ok(e)) = (
                    chrono::NaiveDateTime::parse_from_str(&start, "%Y-%m-%dT%H:%M:%S"),
                    chrono::NaiveDateTime::parse_from_str(&end, "%Y-%m-%dT%H:%M:%S"),
                ) {
                    let total_secs = (e - s).num_seconds() as f64;
                    if total_secs > 0.0 {
                        total_runs as f64 / (total_secs / 3600.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            Ok(MapStats {
                total_runs,
                avg_duration_secs,
                maps_per_hour,
                total_deaths,
            })
        })?;
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_run() -> MapRun {
        MapRun {
            id: None,
            map_name: "Strand".into(),
            area_level: Some(83),
            started_at: "2025-05-20T14:00:10".into(),
            ended_at: "2025-05-20T14:02:00".into(),
            duration_secs: 110.0,
            deaths: 1,
            level_ups: vec![95],
        }
    }

    #[test]
    fn insert_and_retrieve() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();

        let id = db.insert_map_run(&test_run()).unwrap();
        assert!(id > 0);

        let history = db.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].map_name, "Strand");
        assert_eq!(history[0].deaths, 1);
        assert_eq!(history[0].level_ups, vec![95]);
    }

    #[test]
    fn stats_calculation() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();

        db.insert_map_run(&test_run()).unwrap();
        db.insert_map_run(&MapRun {
            map_name: "Atoll".into(),
            area_level: Some(81),
            started_at: "2025-05-20T14:03:00".into(),
            ended_at: "2025-05-20T14:05:00".into(),
            duration_secs: 120.0,
            deaths: 0,
            level_ups: vec![],
            ..test_run()
        })
        .unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_runs, 2);
        assert_eq!(stats.total_deaths, 1);
        assert!((stats.avg_duration_secs - 115.0).abs() < 0.1);
    }
}
