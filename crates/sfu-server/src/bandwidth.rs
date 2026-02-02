use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct BandwidthEstimator {
    window_size: Duration,
    packet_history: Arc<RwLock<Vec<(Instant, usize)>>>,
}

impl Default for BandwidthEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl BandwidthEstimator {
    pub fn new() -> Self {
        Self {
            window_size: Duration::from_secs(1),
            packet_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_packet(&self, size_bytes: usize) {
        let mut history = self.packet_history.write().await;
        history.push((Instant::now(), size_bytes));

        // Cleanup old packets
        let now = Instant::now();
        history.retain(|(time, _)| now.duration_since(*time) <= self.window_size);
    }

    pub async fn estimate_bandwidth(&self) -> u32 {
        let history = self.packet_history.read().await;
        let total_bytes: usize = history.iter().map(|(_, size)| size).sum();

        if total_bytes == 0 {
            return 0;
        }

        // Calculate bits per second
        total_bytes as u32 * 8
    }
}
