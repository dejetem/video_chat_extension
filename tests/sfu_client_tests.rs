use video_chat_sfu_client::SfuClient;
use video_chat_signaling::stun_config::StunConfig;

#[tokio::test]
async fn test_sfu_client_and_room_integration() {
    let stun = StunConfig {
        urls: vec!["stun:stun.l.google.com:19302".into()],
    };

    // Test client creation
    let client = SfuClient::new("integration-room".into(), &stun);
    assert!(client.is_ok(), "Client should be created successfully");

    let client = client.unwrap();
    assert_eq!(client.room_id, "integration-room");

    // Simulated subscribe/unsubscribe
    let sub_result = client.subscribe_to_stream("participant-1").await;
    assert!(sub_result.is_ok());

    let unsub_result = client.unsubscribe_from_stream("participant-1").await;
    assert!(unsub_result.is_ok());
}
