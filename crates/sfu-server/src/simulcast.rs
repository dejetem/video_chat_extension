pub struct SimulcastManager;

impl SimulcastManager {
    /// Selects the best simulcast layer based on available bandwidth
    /// Layers are typically 0 (low), 1 (medium), 2 (high)
    pub fn select_layer(available_layers: &[u8], target_bandwidth: u32) -> u8 {
        if available_layers.is_empty() {
            return 0;
        }

        // Bandwidth thresholds in bps
        #[allow(dead_code)]
        const LOW_THRESHOLD: u32 = 150_000; // 150 kbps
        const MID_THRESHOLD: u32 = 500_000; // 500 kbps
        const HIGH_THRESHOLD: u32 = 1_500_000; // 1.5 Mbps

        let target_layer = if target_bandwidth >= HIGH_THRESHOLD {
            2
        } else if target_bandwidth >= MID_THRESHOLD {
            1
        } else {
            0
        };

        // Find the highest available layer that is <= target_layer
        // If exact target isn't found, fallback to next best available
        available_layers
            .iter()
            .filter(|&&layer| layer <= target_layer)
            .max()
            .copied()
            .unwrap_or(*available_layers.iter().min().unwrap())
    }
}
