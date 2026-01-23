use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::SDPType;

pub struct NegotiationManager;

impl NegotiationManager {
    pub fn parse_sdp(sdp_str: &str, sdp_type: SDPType) -> RTCSessionDescription {
        RTCSessionDescription {
            sdp: sdp_str.to_string(),
            sdp_type,
        }
    }

    // Future: Handle offer/answer logic specifically for the SFU (SFU as Answerer)
}
