use crate::{MediaError, Result};
use log::debug;

use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnection};

pub struct IceCandidateHandler;

impl IceCandidateHandler {
    pub async fn add_ice_candidate(
        connection: &RtcPeerConnection,
        candidate: &str,
        sdp_mid: Option<&str>,
        sdp_m_line_index: Option<u16>,
    ) -> Result<()> {
        let init = RtcIceCandidateInit::new(candidate);
        if let Some(mid) = sdp_mid {
            init.set_sdp_mid(Some(mid));
        }
        if let Some(idx) = sdp_m_line_index {
            init.set_sdp_m_line_index(Some(idx));
        }

        let candidate_obj = RtcIceCandidate::new(&init).map_err(|e| {
            MediaError::WebRtc(format!("Failed to create RtcIceCandidate: {:?}", e))
        })?;

        let promise = connection.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate_obj));
        JsFuture::from(promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to add ICE candidate: {:?}", e)))?;

        debug!("Added ICE candidate");
        Ok(())
    }
}
