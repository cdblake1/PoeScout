pub mod areas;
pub mod db;
pub mod encounters;
pub mod parser;
pub mod session;
pub mod state;
pub mod watcher;

use anyhow::Result;
use chrono::NaiveDateTime;
use db::MapDb;
use parser::LogEvent;
use state::{MapRun, MapSession, MapStats, StateEvent, StateMachine, TrackerState};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

fn now_local() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

pub struct MapTracker {
    db: Arc<MapDb>,
    cancel_tx: Option<watch::Sender<bool>>,
    log_rx: Option<mpsc::UnboundedReceiver<LogEvent>>,
    state_machine: StateMachine,
    current_state: TrackerState,
    current_session_id: Option<i64>,
    client_txt_path: PathBuf,
}

impl MapTracker {
    pub fn new(db_path: &Path, client_txt_path: PathBuf) -> Result<Self> {
        let db = Arc::new(MapDb::open(db_path)?);
        // Resume an open session if the app restarted mid-session.
        let current_session_id = db.get_active_session()?.and_then(|s| s.id);
        let now = chrono::Local::now().naive_local();
        Ok(Self {
            db,
            cancel_tx: None,
            log_rx: None,
            state_machine: StateMachine::new(now),
            current_state: TrackerState::Stopped,
            current_session_id,
            client_txt_path,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if self.cancel_tx.is_some() {
            return Ok(());
        }

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let path = self.client_txt_path.clone();
        tokio::spawn(async move {
            if let Err(e) = watcher::watch_client_txt(path, event_tx, cancel_rx).await {
                tracing::error!("Watcher error: {}", e);
            }
        });

        self.cancel_tx = Some(cancel_tx);
        self.log_rx = Some(event_rx);
        self.current_state = self.state_machine.state();

        tracing::info!("Map tracker started");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(true);
        }
        self.log_rx = None;
        self.current_state = TrackerState::Stopped;
        tracing::info!("Map tracker stopped");
    }

    pub fn is_running(&self) -> bool {
        self.cancel_tx.is_some()
    }

    /// Drain raw log events, run them through the state machine, persist completed
    /// runs (stamped with the active session), and return events for the frontend.
    pub fn poll_events(&mut self) -> Vec<StateEvent> {
        let mut logs = Vec::new();
        if let Some(rx) = &mut self.log_rx {
            while let Ok(ev) = rx.try_recv() {
                logs.push(ev);
            }
        }

        let mut out = Vec::new();
        for log in logs {
            for mut se in self.state_machine.process(log) {
                match &mut se {
                    StateEvent::MapCompleted(run) => {
                        run.session_id = self.current_session_id;
                        if let Err(e) = self.db.insert_map_run(run) {
                            tracing::error!("Failed to save map run: {}", e);
                        }
                    }
                    StateEvent::StateChanged(state) => {
                        self.current_state = state.clone();
                    }
                    StateEvent::Death { .. } => {}
                }
                out.push(se);
            }
        }
        out
    }

    pub fn state(&self) -> TrackerState {
        self.current_state.clone()
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<MapRun>> {
        self.db.get_history(limit, offset)
    }

    pub fn get_stats(&self) -> Result<MapStats> {
        self.db.get_stats()
    }

    // --- Sessions (6.2) ---

    pub fn active_session_id(&self) -> Option<i64> {
        self.current_session_id
    }

    pub fn get_active_session(&self) -> Result<Option<MapSession>> {
        self.db.get_active_session()
    }

    pub fn start_session(
        &mut self,
        league: Option<&str>,
        label: Option<&str>,
        start_chaos: Option<f64>,
    ) -> Result<i64> {
        let started_at = now_local().format("%Y-%m-%dT%H:%M:%S").to_string();
        let id = self.db.start_session(&started_at, league, label, start_chaos)?;
        self.current_session_id = Some(id);
        Ok(id)
    }

    /// Finalize the in-progress run (so its active time counts) and close the
    /// session. Returns the run that was finalized, if any.
    pub fn end_session(&mut self, end_chaos: Option<f64>) -> Result<Option<MapRun>> {
        let now = now_local();
        let ended_at = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut finalized = None;
        if let Some(mut run) = self.state_machine.finalize_current(now) {
            run.session_id = self.current_session_id;
            self.db.insert_map_run(&run)?;
            self.current_state = self.state_machine.state();
            finalized = Some(run);
        }
        if let Some(id) = self.current_session_id.take() {
            self.db.end_session(id, &ended_at, end_chaos)?;
        }
        Ok(finalized)
    }

    pub fn get_sessions(&self, limit: u32, offset: u32) -> Result<Vec<MapSession>> {
        self.db.get_sessions(limit, offset)
    }

    pub fn get_session_runs(&self, session_id: i64) -> Result<Vec<MapRun>> {
        self.db.get_session_runs(session_id)
    }

    pub fn clear_history(&self) -> Result<()> {
        self.db.clear_history()
    }

    pub fn set_character(&mut self, character: Option<String>) {
        self.state_machine.set_character(character);
    }

    pub fn set_league(&mut self, league: Option<String>) {
        self.state_machine.set_league(league);
    }
}
