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

#[wasm_bindgen_test]
async fn test_sdp_negotiation_flow() {
    let stun_config = stun_config::default_stun_config();
    let pc1 = PeerConnectionManager::new(&stun_config).unwrap();
    let pc2 = PeerConnectionManager::new(&stun_config).unwrap();

    // 1. Create Offer
    let offer_sdp = pc1.create_offer().await.expect("Failed to create offer");
    assert!(!offer_sdp.is_empty());

    // 2. Set Remote on PC2
    pc2.set_remote_description(&offer_sdp, web_sys::RtcSdpType::Offer)
        .await
        .expect("Failed to set remote offer");

    // 3. Create Answer
    let answer_sdp = pc2.create_answer().await.expect("Failed to create answer");
    assert!(!answer_sdp.is_empty());

    // 4. Set Remote on PC1
    pc1.set_remote_description(&answer_sdp, web_sys::RtcSdpType::Answer)
        .await
        .expect("Failed to set remote answer");
}
