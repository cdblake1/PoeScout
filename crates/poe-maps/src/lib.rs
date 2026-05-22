pub mod areas;
pub mod db;
pub mod parser;
pub mod state;
pub mod watcher;

use anyhow::Result;
use db::MapDb;
use state::{MapRun, MapStats, StateEvent, TrackerState};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub struct MapTracker {
    db: Arc<MapDb>,
    cancel_tx: Option<watch::Sender<bool>>,
    event_rx: Option<mpsc::UnboundedReceiver<StateEvent>>,
    current_state: TrackerState,
    client_txt_path: PathBuf,
}

impl MapTracker {
    pub fn new(db_path: &Path, client_txt_path: PathBuf) -> Result<Self> {
        let db = Arc::new(MapDb::open(db_path)?);
        Ok(Self {
            db,
            cancel_tx: None,
            event_rx: None,
            current_state: TrackerState::Stopped,
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
        self.event_rx = Some(event_rx);

        tracing::info!("Map tracker started");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(true);
        }
        self.event_rx = None;
        self.current_state = TrackerState::Stopped;
        tracing::info!("Map tracker stopped");
    }

    pub fn is_running(&self) -> bool {
        self.cancel_tx.is_some()
    }

    /// Drain pending events, save completed runs, return events for frontend emission
    pub fn poll_events(&mut self) -> Vec<StateEvent> {
        let mut events = Vec::new();
        if let Some(rx) = &mut self.event_rx {
            while let Ok(event) = rx.try_recv() {
                match &event {
                    StateEvent::MapCompleted(run) => {
                        if let Err(e) = self.db.insert_map_run(run) {
                            tracing::error!("Failed to save map run: {}", e);
                        }
                    }
                    StateEvent::StateChanged(state) => {
                        self.current_state = state.clone();
                    }
                    StateEvent::Death { .. } => {}
                }
                events.push(event);
            }
        }
        events
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
}
