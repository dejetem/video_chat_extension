pub struct BandwidthStats {
    pub available_outgoing_bitrate: u64,
    pub current_bitrate: u64,
    pub packet_loss: f64,
}

impl BandwidthStats {
    pub fn new() -> Self {
        Self {
            available_outgoing_bitrate: 0,
            current_bitrate: 0,
            packet_loss: 0.0,
        }
    }

    pub fn update(&mut self, available: u64, current: u64, loss: f64) {
        self.available_outgoing_bitrate = available;
        self.current_bitrate = current;
        self.packet_loss = loss;
    }
}

impl Default for BandwidthStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandwidth_stats() {
        let mut stats = BandwidthStats::new();
        assert_eq!(stats.available_outgoing_bitrate, 0);

        stats.update(1000, 500, 0.05);
        assert_eq!(stats.available_outgoing_bitrate, 1000);
        assert_eq!(stats.current_bitrate, 500);
        assert_eq!(stats.packet_loss, 0.05);
    }
}
