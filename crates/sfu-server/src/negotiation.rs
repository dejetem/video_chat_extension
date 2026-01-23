use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub struct NegotiationManager;

impl NegotiationManager {
    pub fn parse_sdp(sdp_str: &str, sdp_type: RTCSdpType) -> RTCSessionDescription {
        let mut desc = RTCSessionDescription::default();
        desc.sdp = sdp_str.to_string();
        desc.sdp_type = sdp_type;
        desc
    }
}
