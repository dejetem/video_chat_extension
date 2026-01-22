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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_config() {
        let config = default_stun_config();
        assert_eq!(config.urls[0], "stun:stun.l.google.com:19302");

        let default_config = StunConfig::default();
        assert_eq!(default_config.urls.len(), 1);
    }
}
