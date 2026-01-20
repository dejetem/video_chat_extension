#![cfg(target_arch = "wasm32")]

use video_chat_media::{DataChannelManager, PeerConnectionManager, StreamManager};
use video_chat_signaling::stun_config;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_peer_connection_creation() {
    let stun_config = stun_config::default_stun_config();
    let manager = PeerConnectionManager::new(&stun_config);
    assert!(manager.is_ok());
}

#[wasm_bindgen_test]
async fn test_data_channel_negotiation() {
    let stun_config = stun_config::default_stun_config();
    let pc1_mgr = PeerConnectionManager::new(&stun_config).unwrap();
    let pc2_mgr = PeerConnectionManager::new(&stun_config).unwrap();

    let mut dc_mgr = DataChannelManager::new();
    // This would require more complex mocking or running in browser to fully test
    // data channel state transitions.
    // For now we just verify we can attempt creation.
    let result = dc_mgr.create_data_channel(pc1_mgr.get_connection(), "chat");
    assert!(result.is_ok());
}
