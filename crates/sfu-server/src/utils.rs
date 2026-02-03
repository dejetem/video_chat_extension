use video_chat_signaling::stun_config::{IceServerConfig, StunConfig};
use webrtc::ice_transport::ice_credential_type::RTCIceCredentialType;
use webrtc::ice_transport::ice_server::RTCIceServer;

pub fn load_ice_servers() -> StunConfig {
    let stun_url =
        std::env::var("STUN_URL").unwrap_or_else(|_| "stun:stun.l.google.com:19302".to_string());
    let turn_username = std::env::var("TURN_USERNAME").ok();
    let turn_credential = std::env::var("TURN_CREDENTIAL").ok();

    let mut ice_servers = vec![IceServerConfig {
        urls: vec![stun_url],
        ..Default::default()
    }];

    // Add TURN servers if credentials provided
    if let (Some(u), Some(c)) = (turn_username, turn_credential) {
        // Add multiple variations as we used to
        for i in 1..=4 {
            if let Ok(url) = std::env::var(format!("TURN_URL_{}", i)) {
                ice_servers.push(IceServerConfig {
                    urls: vec![url],
                    username: Some(u.clone()),
                    credential: Some(c.clone()),
                });
            }
        }
    }

    StunConfig { ice_servers }
}

pub fn stun_to_webrtc(stun: &StunConfig) -> Vec<RTCIceServer> {
    stun.ice_servers
        .iter()
        .map(|s| RTCIceServer {
            urls: s.urls.clone(),
            username: s.username.clone().unwrap_or_default(),
            credential: s.credential.clone().unwrap_or_default(),
            credential_type: if s.username.is_some() {
                RTCIceCredentialType::Password
            } else {
                RTCIceCredentialType::Unspecified
            },
        })
        .collect()
}
