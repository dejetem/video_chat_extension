use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};
use video_chat_core::error::Result;

pub struct Recorder {
    active_recordings: Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, tokio::sync::mpsc::Sender<Vec<u8>>>>,
    >,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            active_recordings: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn start_recording(&self, track_id: &str) -> Result<()> {
        info!("Starting recording track: {}", track_id);

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        let track_id_clone = track_id.to_string();

        // Spawn recording task
        tokio::spawn(async move {
            let filename = format!(
                "recording_{}_{}.bin",
                track_id_clone,
                chrono::Utc::now().timestamp()
            );
            let mut file = match File::create(&filename).await {
                Ok(f) => f,
                Err(e) => {
                    warn!("Failed to create recording file: {}", e);
                    return;
                }
            };

            while let Some(packet) = rx.recv().await {
                if let Err(e) = file.write_all(&packet).await {
                    warn!("Failed to write to recording file: {}", e);
                    break;
                }
            }

            info!("Recording finished for {}", track_id_clone);
        });

        let mut recordings = self.active_recordings.write().await;
        recordings.insert(track_id.to_string(), tx);

        Ok(())
    }

    pub async fn stop_recording(&self, track_id: &str) -> Result<()> {
        info!("Stopping recording track: {}", track_id);
        let mut recordings = self.active_recordings.write().await;
        recordings.remove(track_id);
        Ok(())
    }

    pub async fn record_packet(&self, track_id: &str, packet: &[u8]) {
        let recordings = self.active_recordings.read().await;
        if let Some(tx) = recordings.get(track_id) {
            // Non-blocking send, drop if buffer full
            let _ = tx.try_send(packet.to_vec());
        }
    }
}
