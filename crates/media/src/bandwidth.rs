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
