use tracing::info;
use video_chat_core::error::Result;

pub struct Recorder;

impl Recorder {
    pub fn new() -> Self {
        Self
    }

    pub async fn start_recording(&self, track_id: &str) -> Result<()> {
        info!("Started recording track: {}", track_id);
        // Future: Save RTP packets to WebM/MKV file
        Ok(())
    }

    pub async fn stop_recording(&self, track_id: &str) -> Result<()> {
        info!("Stopped recording track: {}", track_id);
        Ok(())
    }
}
