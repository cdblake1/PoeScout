use crate::state::{LootItem, MapEncounter, MapRun, MapSession, MapStats, MapTypeStat};
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

        // Base schema (v0) — matches the originally shipped table so existing DBs line up.
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

        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

        // v1 — Phase 6.1/6.2: richer run model, sessions, encounters.
        if version < 1 {
            conn.execute_batch(
                "ALTER TABLE map_runs ADD COLUMN area_id TEXT;
                 ALTER TABLE map_runs ADD COLUMN area_type TEXT;
                 ALTER TABLE map_runs ADD COLUMN map_tier INTEGER;
                 ALTER TABLE map_runs ADD COLUMN instance_id TEXT;
                 ALTER TABLE map_runs ADD COLUMN league TEXT;
                 ALTER TABLE map_runs ADD COLUMN session_id INTEGER;

                 CREATE TABLE IF NOT EXISTS map_sessions (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     label TEXT,
                     league TEXT,
                     started_at TEXT NOT NULL,
                     ended_at TEXT,
                     start_chaos REAL,
                     end_chaos REAL,
                     profit_chaos REAL,
                     active_secs REAL NOT NULL DEFAULT 0.0,
                     notes TEXT
                 );

                 CREATE TABLE IF NOT EXISTS map_encounters (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     run_id INTEGER NOT NULL REFERENCES map_runs(id),
                     category TEXT NOT NULL,
                     detail TEXT,
                     timestamp TEXT NOT NULL
                 );

                 CREATE INDEX IF NOT EXISTS idx_map_encounters_run ON map_encounters(run_id);
                 CREATE INDEX IF NOT EXISTS idx_map_runs_session ON map_runs(session_id);
                 PRAGMA user_version = 1;",
            )?;
        }

        // v2 — Phase 6.3: per-map loot.
        if version < 2 {
            conn.execute_batch(
                "ALTER TABLE map_runs ADD COLUMN loot_chaos REAL;
                 CREATE TABLE IF NOT EXISTS loot_items (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     run_id INTEGER NOT NULL REFERENCES map_runs(id),
                     name TEXT NOT NULL,
                     type_line TEXT,
                     stack_size INTEGER NOT NULL DEFAULT 1,
                     unit_chaos REAL,
                     total_chaos REAL,
                     frame_type INTEGER
                 );
                 CREATE INDEX IF NOT EXISTS idx_loot_items_run ON loot_items(run_id);
                 PRAGMA user_version = 2;",
            )?;
        }

        Ok(())
    }

    pub fn insert_map_run(&self, run: &MapRun) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let level_ups_json = serde_json::to_string(&run.level_ups)?;
        conn.execute(
            "INSERT INTO map_runs
                (map_name, area_id, area_level, area_type, map_tier, instance_id, league,
                 session_id, started_at, ended_at, duration_secs, hideout_secs, deaths, level_ups)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                run.map_name,
                run.area_id,
                run.area_level,
                run.area_type,
                run.map_tier,
                run.instance_id,
                run.league,
                run.session_id,
                run.started_at,
                run.ended_at,
                run.duration_secs,
                run.hideout_secs,
                run.deaths,
                level_ups_json,
            ],
        )?;
        let run_id = conn.last_insert_rowid();
        for enc in &run.encounters {
            conn.execute(
                "INSERT INTO map_encounters (run_id, category, detail, timestamp)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![run_id, enc.category, enc.detail, enc.timestamp],
            )?;
        }
        Ok(run_id)
    }

    fn encounters_for(conn: &Connection, run_id: i64) -> Result<Vec<MapEncounter>> {
        let mut stmt = conn.prepare(
            "SELECT category, detail, timestamp FROM map_encounters
             WHERE run_id = ?1 ORDER BY timestamp",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(MapEncounter {
                category: row.get(0)?,
                detail: row.get(1)?,
                timestamp: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<MapRun>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, map_name, area_id, area_level, area_type, map_tier, instance_id, league,
                    session_id, started_at, ended_at, duration_secs, hideout_secs, deaths, level_ups,
                    loot_chaos
             FROM map_runs ORDER BY started_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            let level_ups_str: String = row.get(14)?;
            let level_ups: Vec<u32> =
                serde_json::from_str(&level_ups_str).unwrap_or_default();
            Ok(MapRun {
                id: Some(row.get(0)?),
                map_name: row.get(1)?,
                area_id: row.get(2)?,
                area_level: row.get(3)?,
                area_type: row.get(4)?,
                map_tier: row.get(5)?,
                instance_id: row.get(6)?,
                league: row.get(7)?,
                session_id: row.get(8)?,
                started_at: row.get(9)?,
                ended_at: row.get(10)?,
                duration_secs: row.get(11)?,
                hideout_secs: row.get(12)?,
                deaths: row.get(13)?,
                level_ups,
                encounters: Vec::new(),
                loot_chaos: row.get(15)?,
            })
        })?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        drop(stmt);
        for run in &mut runs {
            if let Some(id) = run.id {
                run.encounters = Self::encounters_for(&conn, id)?;
            }
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

    /// Per-map-type aggregates, grouped by internal area id (falling back to name).
    pub fn get_map_type_stats(&self) -> Result<Vec<MapTypeStat>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT MAX(map_name) AS map_name, area_id,
                    COUNT(*) AS run_count,
                    COALESCE(AVG(duration_secs), 0) AS avg_duration,
                    AVG(loot_chaos) AS avg_loot,
                    COALESCE(SUM(deaths), 0) AS total_deaths
             FROM map_runs
             GROUP BY COALESCE(area_id, map_name)
             ORDER BY run_count DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MapTypeStat {
                map_name: row.get(0)?,
                area_id: row.get(1)?,
                run_count: row.get(2)?,
                avg_duration_secs: row.get(3)?,
                avg_loot_chaos: row.get(4)?,
                total_deaths: row.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // --- Sessions (6.2) ---

    pub fn start_session(
        &self,
        started_at: &str,
        league: Option<&str>,
        label: Option<&str>,
        start_chaos: Option<f64>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO map_sessions (label, league, started_at, start_chaos)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![label, league, started_at, start_chaos],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Close a session: profit = end − start; active_secs = Σ run durations in the session.
    pub fn end_session(&self, id: i64, ended_at: &str, end_chaos: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let active_secs: f64 = conn.query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM map_runs WHERE session_id = ?1",
            [id],
            |r| r.get(0),
        )?;
        let start_chaos: Option<f64> =
            conn.query_row("SELECT start_chaos FROM map_sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })?;
        let profit = match (start_chaos, end_chaos) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        };
        conn.execute(
            "UPDATE map_sessions
             SET ended_at = ?2, end_chaos = ?3, profit_chaos = ?4, active_secs = ?5
             WHERE id = ?1",
            rusqlite::params![id, ended_at, end_chaos, profit, active_secs],
        )?;
        Ok(())
    }

    /// The currently open session (no `ended_at`), if any.
    pub fn get_active_session(&self) -> Result<Option<MapSession>> {
        let conn = self.conn.lock().unwrap();
        let id = conn.query_row(
            "SELECT id FROM map_sessions WHERE ended_at IS NULL ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get::<_, i64>(0),
        );
        match id {
            Ok(id) => Ok(Some(Self::session_by_id(&conn, id)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_sessions(&self, limit: u32, offset: u32) -> Result<Vec<MapSession>> {
        let conn = self.conn.lock().unwrap();
        let ids: Vec<i64> = {
            let mut stmt = conn
                .prepare("SELECT id FROM map_sessions ORDER BY started_at DESC LIMIT ?1 OFFSET ?2")?;
            let rows = stmt.query_map(rusqlite::params![limit, offset], |r| r.get(0))?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };
        let mut out = Vec::new();
        for id in ids {
            out.push(Self::session_by_id(&conn, id)?);
        }
        Ok(out)
    }

    fn session_by_id(conn: &Connection, id: i64) -> Result<MapSession> {
        let mut s = conn.query_row(
            "SELECT id, label, league, started_at, ended_at, start_chaos, end_chaos,
                    profit_chaos, active_secs, notes
             FROM map_sessions WHERE id = ?1",
            [id],
            |row| {
                Ok(MapSession {
                    id: Some(row.get(0)?),
                    label: row.get(1)?,
                    league: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?,
                    start_chaos: row.get(5)?,
                    end_chaos: row.get(6)?,
                    profit_chaos: row.get(7)?,
                    active_secs: row.get(8)?,
                    notes: row.get(9)?,
                    run_count: 0,
                    chaos_per_hour: None,
                })
            },
        )?;
        s.run_count = conn.query_row(
            "SELECT COUNT(*) FROM map_runs WHERE session_id = ?1",
            [id],
            |r| r.get(0),
        )?;
        s.chaos_per_hour = match s.profit_chaos {
            Some(p) if s.active_secs > 0.0 => Some(p / (s.active_secs / 3600.0)),
            _ => None,
        };
        Ok(s)
    }

    pub fn get_session_runs(&self, session_id: i64) -> Result<Vec<MapRun>> {
        let conn = self.conn.lock().unwrap();
        let mut runs = {
            let mut stmt = conn.prepare(
                "SELECT id, map_name, area_id, area_level, area_type, map_tier, instance_id, league,
                        session_id, started_at, ended_at, duration_secs, hideout_secs, deaths, level_ups,
                        loot_chaos
                 FROM map_runs WHERE session_id = ?1 ORDER BY started_at DESC",
            )?;
            let rows = stmt.query_map([session_id], |row| {
                let level_ups_str: String = row.get(14)?;
                let level_ups: Vec<u32> = serde_json::from_str(&level_ups_str).unwrap_or_default();
                Ok(MapRun {
                    id: Some(row.get(0)?),
                    map_name: row.get(1)?,
                    area_id: row.get(2)?,
                    area_level: row.get(3)?,
                    area_type: row.get(4)?,
                    map_tier: row.get(5)?,
                    instance_id: row.get(6)?,
                    league: row.get(7)?,
                    session_id: row.get(8)?,
                    started_at: row.get(9)?,
                    ended_at: row.get(10)?,
                    duration_secs: row.get(11)?,
                    hideout_secs: row.get(12)?,
                    deaths: row.get(13)?,
                    level_ups,
                    encounters: Vec::new(),
                    loot_chaos: row.get(15)?,
                })
            })?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };
        for run in &mut runs {
            if let Some(rid) = run.id {
                run.encounters = Self::encounters_for(&conn, rid)?;
            }
        }
        Ok(runs)
    }

    /// Delete all map runs and their encounters (the "Recent Runs" history).
    /// Sessions are left intact.
    pub fn clear_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("DELETE FROM map_encounters; DELETE FROM map_runs;")?;
        Ok(())
    }

    /// Attach priced loot to a completed run (6.3): set its total and insert lines.
    pub fn set_run_loot(&self, run_id: i64, loot_chaos: f64, items: &[LootItem]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE map_runs SET loot_chaos = ?2 WHERE id = ?1",
            rusqlite::params![run_id, loot_chaos],
        )?;
        for it in items {
            conn.execute(
                "INSERT INTO loot_items
                    (run_id, name, type_line, stack_size, unit_chaos, total_chaos, frame_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    run_id,
                    it.name,
                    it.type_line,
                    it.stack_size,
                    it.unit_chaos,
                    it.total_chaos,
                    it.frame_type,
                ],
            )?;
        }
        Ok(())
    }

    pub fn get_run_loot(&self, run_id: i64) -> Result<Vec<LootItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, type_line, stack_size, unit_chaos, total_chaos, frame_type
             FROM loot_items WHERE run_id = ?1 ORDER BY total_chaos DESC",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(LootItem {
                name: row.get(0)?,
                type_line: row.get(1)?,
                stack_size: row.get(2)?,
                unit_chaos: row.get(3)?,
                total_chaos: row.get(4)?,
                frame_type: row.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
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
            ..Default::default()
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

    #[test]
    fn new_fields_roundtrip_and_migration_idempotent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        {
            let db = MapDb::open(&path).unwrap();
            let run = MapRun {
                area_id: Some("MapWorldsStrand".into()),
                area_type: Some("map".into()),
                map_tier: Some(16),
                instance_id: Some("8.8.8.8:6112".into()),
                league: Some("Mirage".into()),
                session_id: Some(7),
                hideout_secs: 42.0,
                ..test_run()
            };
            db.insert_map_run(&run).unwrap();
        }
        // Reopen: migrate() must run again without failing (ALTERs gated by user_version).
        let db = MapDb::open(&path).unwrap();
        let history = db.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        let r = &history[0];
        assert_eq!(r.area_id.as_deref(), Some("MapWorldsStrand"));
        assert_eq!(r.area_type.as_deref(), Some("map"));
        assert_eq!(r.map_tier, Some(16));
        assert_eq!(r.instance_id.as_deref(), Some("8.8.8.8:6112"));
        assert_eq!(r.league.as_deref(), Some("Mirage"));
        assert_eq!(r.session_id, Some(7));
        assert!((r.hideout_secs - 42.0).abs() < 0.001);
    }

    #[test]
    fn encounters_persist_and_load() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();
        let run = MapRun {
            encounters: vec![
                MapEncounter {
                    category: "Delve".into(),
                    detail: None,
                    timestamp: "2025-05-20T14:00:20".into(),
                },
                MapEncounter {
                    category: "Bestiary".into(),
                    detail: Some("yellow".into()),
                    timestamp: "2025-05-20T14:00:30".into(),
                },
            ],
            ..test_run()
        };
        db.insert_map_run(&run).unwrap();

        let history = db.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].encounters.len(), 2);
        assert!(history[0]
            .encounters
            .iter()
            .any(|e| e.category == "Bestiary" && e.detail.as_deref() == Some("yellow")));
    }

    #[test]
    fn session_lifecycle_profit_and_cphr() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();

        let sid = db
            .start_session("2025-05-20T14:00:00", Some("Mirage"), Some("test"), Some(100.0))
            .unwrap();
        assert!(db.get_active_session().unwrap().is_some());

        let r1 = MapRun {
            session_id: Some(sid),
            duration_secs: 110.0,
            ..test_run()
        };
        let r2 = MapRun {
            map_name: "Atoll".into(),
            session_id: Some(sid),
            duration_secs: 130.0,
            ..test_run()
        };
        db.insert_map_run(&r1).unwrap();
        db.insert_map_run(&r2).unwrap();

        db.end_session(sid, "2025-05-20T15:00:00", Some(340.0)).unwrap();

        let sessions = db.get_sessions(10, 0).unwrap();
        assert_eq!(sessions.len(), 1);
        let s = &sessions[0];
        assert_eq!(s.run_count, 2);
        assert!((s.profit_chaos.unwrap() - 240.0).abs() < 0.001);
        assert!((s.active_secs - 240.0).abs() < 0.001);
        // 240 chaos over 240s active = 3600 c/hr.
        assert!((s.chaos_per_hour.unwrap() - 3600.0).abs() < 1.0);

        assert!(db.get_active_session().unwrap().is_none());
        assert_eq!(db.get_session_runs(sid).unwrap().len(), 2);
    }

    #[test]
    fn history_pagination() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();
        for (i, t) in [
            "2025-05-20T14:00:10",
            "2025-05-20T14:01:10",
            "2025-05-20T14:02:10",
        ]
        .iter()
        .enumerate()
        {
            db.insert_map_run(&MapRun {
                map_name: format!("Map{i}"),
                started_at: t.to_string(),
                ..test_run()
            })
            .unwrap();
        }
        assert_eq!(db.get_history(2, 0).unwrap().len(), 2);
        assert_eq!(db.get_history(2, 2).unwrap().len(), 1);
        // Newest first.
        assert_eq!(db.get_history(1, 0).unwrap()[0].started_at, "2025-05-20T14:02:10");
    }

    #[test]
    fn session_active_secs_isolated_per_session() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();

        let s1 = db
            .start_session("2025-05-20T14:00:00", None, None, Some(0.0))
            .unwrap();
        db.insert_map_run(&MapRun {
            session_id: Some(s1),
            duration_secs: 100.0,
            ..test_run()
        })
        .unwrap();
        db.end_session(s1, "2025-05-20T14:30:00", Some(50.0)).unwrap();

        let s2 = db
            .start_session("2025-05-20T15:00:00", None, None, Some(50.0))
            .unwrap();
        db.insert_map_run(&MapRun {
            session_id: Some(s2),
            duration_secs: 200.0,
            ..test_run()
        })
        .unwrap();
        db.end_session(s2, "2025-05-20T15:30:00", Some(300.0)).unwrap();

        let sessions = db.get_sessions(10, 0).unwrap();
        let by_id = |id: i64| sessions.iter().find(|s| s.id == Some(id)).unwrap();
        assert!((by_id(s1).active_secs - 100.0).abs() < 0.001);
        assert!((by_id(s2).active_secs - 200.0).abs() < 0.001);
        assert!((by_id(s1).profit_chaos.unwrap() - 50.0).abs() < 0.001);
        assert!((by_id(s2).profit_chaos.unwrap() - 250.0).abs() < 0.001);
    }

    /// Integration: a database created by the ORIGINAL (pre-Phase-6) schema must
    /// upgrade cleanly — ALTERs add the new columns, old rows survive, and the
    /// new tables become usable.
    #[test]
    fn migrates_legacy_v0_database() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE map_runs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    map_name TEXT NOT NULL,
                    area_level INTEGER,
                    started_at TEXT NOT NULL,
                    ended_at TEXT NOT NULL,
                    duration_secs REAL NOT NULL,
                    deaths INTEGER NOT NULL DEFAULT 0,
                    level_ups TEXT NOT NULL DEFAULT '[]',
                    hideout_secs REAL NOT NULL DEFAULT 0.0
                );",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO map_runs
                    (map_name, area_level, started_at, ended_at, duration_secs, deaths, level_ups)
                 VALUES ('OldMap', 80, '2025-01-01T00:00:00', '2025-01-01T00:05:00', 300.0, 2, '[88]')",
                [],
            )
            .unwrap();
            // user_version is left at its default of 0 (a legacy DB).
        }

        // Opening via MapDb runs the v1 migration.
        let db = MapDb::open(&path).unwrap();

        let history = db.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        let r = &history[0];
        assert_eq!(r.map_name, "OldMap");
        assert_eq!(r.deaths, 2);
        assert_eq!(r.level_ups, vec![88]);
        assert!(r.area_id.is_none()); // new column, defaulted
        assert!(r.session_id.is_none());

        // New tables are usable post-migration.
        let sid = db
            .start_session("2025-01-01T01:00:00", None, None, Some(10.0))
            .unwrap();
        assert!(sid > 0);
        let run = MapRun {
            session_id: Some(sid),
            encounters: vec![MapEncounter {
                category: "Delve".into(),
                detail: None,
                timestamp: "2025-01-01T01:01:00".into(),
            }],
            ..test_run()
        };
        db.insert_map_run(&run).unwrap();
        assert_eq!(db.get_session_runs(sid).unwrap().len(), 1);
    }

    #[test]
    fn clear_history_removes_runs_and_encounters() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();
        let run = MapRun {
            encounters: vec![MapEncounter {
                category: "Delve".into(),
                detail: None,
                timestamp: "2025-05-20T14:00:20".into(),
            }],
            ..test_run()
        };
        db.insert_map_run(&run).unwrap();
        assert_eq!(db.get_history(10, 0).unwrap().len(), 1);

        db.clear_history().unwrap();
        assert_eq!(db.get_history(10, 0).unwrap().len(), 0);
    }

    #[test]
    fn run_loot_roundtrip() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();
        let id = db.insert_map_run(&test_run()).unwrap();

        let items = vec![
            LootItem {
                name: "Divine Orb".into(),
                type_line: "Divine Orb".into(),
                stack_size: 1,
                unit_chaos: Some(200.0),
                total_chaos: Some(200.0),
                frame_type: Some(5),
            },
            LootItem {
                name: "Chaos Orb".into(),
                type_line: "Chaos Orb".into(),
                stack_size: 10,
                unit_chaos: Some(1.0),
                total_chaos: Some(10.0),
                frame_type: Some(5),
            },
        ];
        db.set_run_loot(id, 210.0, &items).unwrap();

        let loot = db.get_run_loot(id).unwrap();
        assert_eq!(loot.len(), 2);
        assert_eq!(loot[0].name, "Divine Orb"); // ordered by total_chaos DESC

        let run = db
            .get_history(10, 0)
            .unwrap()
            .into_iter()
            .find(|r| r.id == Some(id))
            .unwrap();
        assert_eq!(run.loot_chaos, Some(210.0));
    }

    #[test]
    fn map_type_stats_aggregates() {
        let dir = tempdir().unwrap();
        let db = MapDb::open(&dir.path().join("test.db")).unwrap();
        let strand = |dur: f64, deaths: u32| MapRun {
            map_name: "Strand".into(),
            area_id: Some("MapWorldsStrand".into()),
            duration_secs: dur,
            deaths,
            ..test_run()
        };
        db.insert_map_run(&strand(100.0, 1)).unwrap();
        db.insert_map_run(&strand(200.0, 0)).unwrap();
        db.insert_map_run(&MapRun {
            map_name: "Atoll".into(),
            area_id: Some("MapWorldsAtoll".into()),
            duration_secs: 50.0,
            ..test_run()
        })
        .unwrap();

        let stats = db.get_map_type_stats().unwrap();
        // Ordered by run_count desc → Strand (2) first.
        assert_eq!(stats[0].area_id.as_deref(), Some("MapWorldsStrand"));
        let strand_stat = &stats[0];
        assert_eq!(strand_stat.run_count, 2);
        assert!((strand_stat.avg_duration_secs - 150.0).abs() < 0.1);
        assert_eq!(strand_stat.total_deaths, 1);
    }
}
