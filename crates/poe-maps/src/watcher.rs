use crate::parser::parse_line;
use crate::state::{StateEvent, StateMachine};
use anyhow::Result;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;

pub async fn watch_client_txt(
    path: PathBuf,
    event_tx: mpsc::UnboundedSender<StateEvent>,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    use tokio::io::BufReader;

    // Open file and seek to end
    let file = tokio::fs::File::open(&path).await?;
    let metadata = file.metadata().await?;
    let mut last_size = metadata.len();
    let mut position = last_size;

    tracing::info!("Watching Client.txt at {:?}, starting at byte {}", path, position);

    let now = chrono::Local::now().naive_local();
    let mut state_machine = StateMachine::new(now);

    loop {
        tokio::select! {
            _ = cancel.changed() => {
                if *cancel.borrow() {
                    tracing::info!("Watcher cancelled");
                    return Ok(());
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                // Check file size for truncation
                let file = match tokio::fs::File::open(&path).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::warn!("Failed to open Client.txt: {}", e);
                        continue;
                    }
                };
                let metadata = file.metadata().await?;
                let current_size = metadata.len();

                if current_size < last_size {
                    tracing::info!("Client.txt truncated, resetting position");
                    position = 0;
                }
                last_size = current_size;

                if current_size <= position {
                    continue;
                }

                // Read new content
                let file = tokio::fs::File::open(&path).await?;
                let mut reader = BufReader::new(file);

                // Seek to position
                use tokio::io::AsyncSeekExt;
                reader.seek(std::io::SeekFrom::Start(position)).await?;

                let mut line = String::new();
                loop {
                    line.clear();
                    let bytes_read = reader.read_line(&mut line).await?;
                    if bytes_read == 0 {
                        break;
                    }
                    position += bytes_read as u64;

                    if let Some(event) = parse_line(line.trim()) {
                        let state_events = state_machine.process(event);
                        for se in state_events {
                            if event_tx.send(se).is_err() {
                                tracing::info!("Event channel closed");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
}
