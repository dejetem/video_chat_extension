pub struct SimulcastManager;

impl SimulcastManager {
    pub fn select_layer(available_layers: &[u8], target_bandwidth: u32) -> u8 {
        // Simple logic to pick the best layer based on bandwidth
        *available_layers.iter().max().unwrap_or(&0)
    }
}
