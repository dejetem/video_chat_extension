use video_chat_signaling::{Message, MessageType, Protocol, SessionDescription, SdpType};

#[test]
fn test_signaling_flow() {
    let protocol = Protocol::new();

    // 1. Join Room
    let join_msg = Message::new(
        "join-1",
        MessageType::Join {
            room_id: "room-123".to_string(),
            participant_id: "user-abc".to_string(),
        },
    );
    assert!(protocol.validate_message(&join_msg).is_ok());

    // 2. Offer SDP
    let offer_sdp = SessionDescription {
        sdp_type: SdpType::Offer,
        sdp: "v=0...".to_string(),
    };
    let offer_msg = Message::new(
        "offer-1",
        MessageType::Offer {
            sdp: offer_sdp,
            participant_id: "user-abc".to_string(),
        },
    );
    assert!(protocol.validate_message(&offer_msg).is_ok());

    // 3. Answer SDP
    let answer_sdp = SessionDescription {
        sdp_type: SdpType::Answer,
        sdp: "v=0...".to_string(),
    };
    let answer_msg = Message::new(
        "answer-1",
        MessageType::Answer {
            sdp: answer_sdp,
            participant_id: "user-xyz".to_string(),
        },
    );
    assert!(protocol.validate_message(&answer_msg).is_ok());
}

#[test]
fn test_invalid_flow() {
    let protocol = Protocol::new();

    // Empty SDP should fail
    let invalid_sdp = SessionDescription {
        sdp_type: SdpType::Offer,
        sdp: String::new(), // Empty!
    };
    let msg = Message::new(
        "invalid-1",
        MessageType::Offer {
            sdp: invalid_sdp,
            participant_id: "user-abc".to_string(),
        },
    );

    assert!(protocol.validate_message(&msg).is_err());
}
