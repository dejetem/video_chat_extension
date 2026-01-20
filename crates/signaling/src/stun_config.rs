use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunConfig {
    pub urls: Vec<String>,
}

impl Default for StunConfig {
    fn default() -> Self {
        default_stun_config()
    }
}

pub fn default_stun_config() -> StunConfig {
    StunConfig {
        urls: vec!["stun:stun.l.google.com:19302".to_string()],
    }
}
