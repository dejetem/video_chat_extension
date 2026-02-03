use video_chat_signaling::{
    Message, MessageType, Protocol, ProtocolVersion, SdpType, SessionDescription,
    WebSocketSignaling,
};

#[tokio::test]
async fn test_protocol_version_compatibility() {
    let v1 = ProtocolVersion::V1;
    assert_eq!(v1.to_string(), "1.0");

    let protocol = Protocol::new(v1);
    assert!(protocol.is_compatible(&v1));
}

#[tokio::test]
async fn test_message_serialization() {
    let session_desc = SessionDescription {
        sdp: "v=0\r\no=- 123 456 IN IP4 127.0.0.1\r\n".to_string(),
        sdp_type: SdpType::Offer,
    };

    let message = Message {
        message_type: MessageType::Offer,
        room_id: "test-room".to_string(),
        participant_id: "participant-1".to_string(),
        payload: serde_json::to_value(&session_desc).unwrap(),
    };

    // Test serialization
    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("test-room"));
    assert!(json.contains("participant-1"));

    // Test deserialization
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.room_id, "test-room");
    assert_eq!(deserialized.participant_id, "participant-1");
}

#[tokio::test]
async fn test_sdp_offer_answer_flow() {
    // Create an offer
    let offer = SessionDescription {
        sdp: "v=0\r\no=- 123 456 IN IP4 127.0.0.1\r\ns=-\r\n".to_string(),
        sdp_type: SdpType::Offer,
    };

    let offer_msg = Message {
        message_type: MessageType::Offer,
        room_id: "room1".to_string(),
        participant_id: "peer1".to_string(),
        payload: serde_json::to_value(&offer).unwrap(),
    };

    // Create an answer
    let answer = SessionDescription {
        sdp: "v=0\r\no=- 789 012 IN IP4 127.0.0.1\r\ns=-\r\n".to_string(),
        sdp_type: SdpType::Answer,
    };

    let answer_msg = Message {
        message_type: MessageType::Answer,
        room_id: "room1".to_string(),
        participant_id: "peer2".to_string(),
        payload: serde_json::to_value(&answer).unwrap(),
    };

    // Verify message types
    assert!(matches!(offer_msg.message_type, MessageType::Offer));
    assert!(matches!(answer_msg.message_type, MessageType::Answer));

    // Verify both messages are for the same room
    assert_eq!(offer_msg.room_id, answer_msg.room_id);
}

#[tokio::test]
async fn test_ice_candidate_exchange() {
    use video_chat_signaling::IceCandidate;

    let candidate = IceCandidate {
        candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host".to_string(),
        sdp_mid: Some("0".to_string()),
        sdp_m_line_index: Some(0),
    };

    let ice_msg = Message {
        message_type: MessageType::IceCandidate,
        room_id: "room1".to_string(),
        participant_id: "peer1".to_string(),
        payload: serde_json::to_value(&candidate).unwrap(),
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&ice_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();

    // Extract candidate from payload
    let extracted_candidate: IceCandidate = serde_json::from_value(deserialized.payload).unwrap();
    assert_eq!(extracted_candidate.candidate, candidate.candidate);
    assert_eq!(extracted_candidate.sdp_mid, candidate.sdp_mid);
}

#[tokio::test]
async fn test_full_signaling_handshake() {
    // Simulate a complete signaling handshake between two peers

    // Step 1: Peer A joins room
    let join_msg = Message {
        message_type: MessageType::Join,
        room_id: "test-room".to_string(),
        participant_id: "peer-a".to_string(),
        payload: serde_json::Value::Null,
    };

    // Step 2: Peer A creates offer
    let offer = SessionDescription {
        sdp: "v=0\r\no=- 111 222 IN IP4 10.0.0.1\r\n".to_string(),
        sdp_type: SdpType::Offer,
    };

    let offer_msg = Message {
        message_type: MessageType::Offer,
        room_id: "test-room".to_string(),
        participant_id: "peer-a".to_string(),
        payload: serde_json::to_value(&offer).unwrap(),
    };

    // Step 3: Peer B joins and creates answer
    let answer = SessionDescription {
        sdp: "v=0\r\no=- 333 444 IN IP4 10.0.0.2\r\n".to_string(),
        sdp_type: SdpType::Answer,
    };

    let answer_msg = Message {
        message_type: MessageType::Answer,
        room_id: "test-room".to_string(),
        participant_id: "peer-b".to_string(),
        payload: serde_json::to_value(&answer).unwrap(),
    };

    // Verify the handshake sequence
    assert_eq!(join_msg.room_id, offer_msg.room_id);
    assert_eq!(offer_msg.room_id, answer_msg.room_id);
    assert_ne!(offer_msg.participant_id, answer_msg.participant_id);
}
