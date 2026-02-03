use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StunConfig {
    pub ice_servers: Vec<IceServerConfig>,
}

pub fn default_stun_config() -> StunConfig {
    StunConfig {
        ice_servers: vec![IceServerConfig {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_config() {
        let config = default_stun_config();
        assert_eq!(
            config.ice_servers[0].urls[0],
            "stun:stun.l.google.com:19302"
        );

        let default_config = StunConfig::default();
        // Since we derive Default, the vector should be empty
        assert_eq!(default_config.ice_servers.len(), 0);
    }
}
