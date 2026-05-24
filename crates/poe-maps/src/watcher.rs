use crate::parser::{parse_line, LogEvent};
use anyhow::Result;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;

/// Tail Client.txt and forward parsed [`LogEvent`]s. The state machine lives in
/// `MapTracker` (so it can be driven and finalized from the command layer).
pub async fn watch_client_txt(
    path: PathBuf,
    event_tx: mpsc::UnboundedSender<LogEvent>,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    use tokio::io::BufReader;

    let file = tokio::fs::File::open(&path).await?;
    let metadata = file.metadata().await?;
    let mut last_size = metadata.len();
    let mut position = last_size;

    tracing::info!(
        "Watching Client.txt at {:?}, starting at byte {}",
        path,
        position
    );

    loop {
        tokio::select! {
            _ = cancel.changed() => {
                if *cancel.borrow() {
                    tracing::info!("Watcher cancelled");
                    return Ok(());
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
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

                let file = tokio::fs::File::open(&path).await?;
                let mut reader = BufReader::new(file);

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
                        if event_tx.send(event).is_err() {
                            tracing::info!("Event channel closed");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
