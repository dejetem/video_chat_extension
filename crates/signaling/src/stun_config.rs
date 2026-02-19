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

/// Build the ICE server list from environment variables.
///
/// Expected `.env` / environment variables:
/// ```text
/// STUN_URL=stun:stun.relay.metered.ca:80
/// TURN_URL_1=turn:global.relay.metered.ca:80
/// TURN_URL_2=turn:global.relay.metered.ca:80?transport=tcp
/// TURN_URL_3=turn:global.relay.metered.ca:443
/// TURN_URL_4=turns:global.relay.metered.ca:443?transport=tcp
/// TURN_USERNAME=<username>
/// TURN_CREDENTIAL=<password>
/// ```
///
/// Falls back to a single Google STUN server if the env vars are not set.
pub fn default_stun_config() -> StunConfig {
    // Read STUN URL (required for NAT traversal without relay)
    let stun_url =
        std::env::var("STUN_URL").unwrap_or_else(|_| "stun:stun.l.google.com:19302".to_string());

    // Read TURN credentials (shared across all TURN URLs)
    let turn_username = std::env::var("TURN_USERNAME").ok();
    let turn_credential = std::env::var("TURN_CREDENTIAL").ok();

    // Collect every TURN_URL_N that is set in the environment
    let turn_urls: Vec<String> = (1..=10)
        .filter_map(|i| std::env::var(format!("TURN_URL_{i}")).ok())
        .collect();

    let mut ice_servers = vec![
        // STUN server (no credentials needed)
        IceServerConfig {
            urls: vec![stun_url],
            username: None,
            credential: None,
        },
    ];

    // Add a TURN server entry for each URL found in the environment
    for url in turn_urls {
        ice_servers.push(IceServerConfig {
            urls: vec![url],
            username: turn_username.clone(),
            credential: turn_credential.clone(),
        });
    }

    if ice_servers.len() == 1 && turn_username.is_none() {
        eprintln!(
            "[stun_config] WARNING: No TURN_URL_* env vars found. Only STUN is configured; \
             connections through strict NATs may fail."
        );
    } else {
        eprintln!(
            "[stun_config] ICE servers loaded: 1 STUN + {} TURN",
            ice_servers.len() - 1
        );
    }

    StunConfig { ice_servers }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_config_fallback() {
        // When no env vars are set the config must contain exactly 1 server
        // (the fallback STUN) and no TURN entries.
        // Use temp_env or simply unset the variables for this unit test.
        std::env::remove_var("STUN_URL");
        std::env::remove_var("TURN_USERNAME");
        std::env::remove_var("TURN_CREDENTIAL");
        for i in 1..=10 {
            std::env::remove_var(format!("TURN_URL_{i}"));
        }

        let config = default_stun_config();
        assert_eq!(config.ice_servers.len(), 1);
        assert_eq!(
            config.ice_servers[0].urls[0],
            "stun:stun.l.google.com:19302"
        );
        assert!(config.ice_servers[0].username.is_none());
    }

    #[test]
    fn test_stun_config_from_env() {
        std::env::set_var("STUN_URL", "stun:stun.example.com:3478");
        std::env::set_var("TURN_URL_1", "turn:turn.example.com:3478");
        std::env::set_var("TURN_USERNAME", "testuser");
        std::env::set_var("TURN_CREDENTIAL", "testpass");

        let config = default_stun_config();

        assert_eq!(config.ice_servers[0].urls[0], "stun:stun.example.com:3478");
        assert_eq!(config.ice_servers[1].urls[0], "turn:turn.example.com:3478");
        assert_eq!(config.ice_servers[1].username.as_deref(), Some("testuser"));
        assert_eq!(
            config.ice_servers[1].credential.as_deref(),
            Some("testpass")
        );

        // Cleanup
        std::env::remove_var("STUN_URL");
        std::env::remove_var("TURN_URL_1");
        std::env::remove_var("TURN_USERNAME");
        std::env::remove_var("TURN_CREDENTIAL");
    }

    #[test]
    fn test_default_stun_config_struct() {
        let default_config = StunConfig::default();
        // Derived Default produces an empty vector
        assert_eq!(default_config.ice_servers.len(), 0);
    }
}
