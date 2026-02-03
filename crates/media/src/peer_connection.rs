use crate::{MediaError, Result};
use js_sys::{Array, Reflect};
use log::info;
use video_chat_signaling::stun_config::StunConfig;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcConfiguration, RtcIceServer, RtcPeerConnection, RtcSessionDescriptionInit};

pub struct PeerConnectionManager {
    connection: RtcPeerConnection,
}

impl PeerConnectionManager {
    pub fn new(stun_config: &StunConfig) -> Result<Self> {
        let rtc_config = RtcConfiguration::new();

        let ice_servers = Array::new();
        for server_config in &stun_config.ice_servers {
            let server = RtcIceServer::new();

            // Set URLs
            let urls_array = Array::new();
            for url in &server_config.urls {
                urls_array.push(&JsValue::from_str(url));
            }
            server.set_urls(&urls_array);

            // Set Credentials if present
            if let Some(username) = &server_config.username {
                server.set_username(username);
            }
            if let Some(credential) = &server_config.credential {
                server.set_credential(credential);
            }

            ice_servers.push(&server);
        }
        rtc_config.set_ice_servers(&ice_servers);

        let connection = RtcPeerConnection::new_with_configuration(&rtc_config).map_err(|e| {
            MediaError::WebRtc(format!("Failed to create RTCPeerConnection: {:?}", e))
        })?;

        info!(
            "Created RTCPeerConnection with {} ICE servers",
            stun_config.ice_servers.len()
        );

        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &RtcPeerConnection {
        &self.connection
    }

    pub async fn create_offer(&self) -> Result<String> {
        let offer_promise = self.connection.create_offer();
        let offer_js = JsFuture::from(offer_promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to create offer: {:?}", e)))?;

        let offer_sdp = Reflect::get(&offer_js, &JsValue::from_str("sdp"))
            .map_err(|e| MediaError::WebRtc(format!("Failed to get SDP from offer: {:?}", e)))?
            .as_string()
            .ok_or_else(|| MediaError::WebRtc("SDP is not a string".into()))?;

        let local_desc = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
        local_desc.set_sdp(&offer_sdp);

        let set_local_promise = self.connection.set_local_description(&local_desc);
        JsFuture::from(set_local_promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to set local description: {:?}", e)))?;

        Ok(offer_sdp)
    }

    pub async fn create_answer(&self) -> Result<String> {
        let answer_promise = self.connection.create_answer();
        let answer_js = JsFuture::from(answer_promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to create answer: {:?}", e)))?;

        let answer_sdp = Reflect::get(&answer_js, &JsValue::from_str("sdp"))
            .map_err(|e| MediaError::WebRtc(format!("Failed to get SDP from answer: {:?}", e)))?
            .as_string()
            .ok_or_else(|| MediaError::WebRtc("SDP is not a string".into()))?;

        let local_desc = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
        local_desc.set_sdp(&answer_sdp);

        let set_local_promise = self.connection.set_local_description(&local_desc);
        JsFuture::from(set_local_promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to set local description: {:?}", e)))?;

        Ok(answer_sdp)
    }

    pub async fn set_remote_description(
        &self,
        sdp: &str,
        sdp_type: web_sys::RtcSdpType,
    ) -> Result<()> {
        let desc = RtcSessionDescriptionInit::new(sdp_type);
        desc.set_sdp(sdp);

        let promise = self.connection.set_remote_description(&desc);
        JsFuture::from(promise).await.map_err(|e| {
            MediaError::WebRtc(format!("Failed to set remote description: {:?}", e))
        })?;

        Ok(())
    }

    pub async fn add_ice_candidate(&self, candidate: &web_sys::RtcIceCandidateInit) -> Result<()> {
        // Create RtcIceCandidate from Init
        let candidate_obj = web_sys::RtcIceCandidate::new(candidate).map_err(|e| {
            MediaError::WebRtc(format!("Failed to create RtcIceCandidate: {:?}", e))
        })?;

        let promise = self
            .connection
            .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate_obj));
        JsFuture::from(promise)
            .await
            .map_err(|e| MediaError::WebRtc(format!("Failed to add ICE candidate: {:?}", e)))?;
        Ok(())
    }

    pub fn add_track(
        &self,
        track: &web_sys::MediaStreamTrack,
        _stream: &web_sys::MediaStream,
    ) -> Result<()> {
        // Use addTransceiver instead of addTrack to avoid simulcast and variadic arguments
        // This creates a transceiver in sendrecv mode without simulcast encodings
        let _ = self
            .connection
            .add_transceiver_with_media_stream_track(track);
        Ok(())
    }

    pub fn add_track_with_simulcast(
        &self,
        track: &web_sys::MediaStreamTrack,
        _stream: &web_sys::MediaStream,
    ) -> Result<()> {
        let init = web_sys::RtcRtpTransceiverInit::new();
        init.set_direction(web_sys::RtcRtpTransceiverDirection::Sendrecv);

        let encodings = Array::new();

        // High quality
        let h_enc = web_sys::RtcRtpEncodingParameters::new();
        h_enc.set_rid("h");
        h_enc.set_max_bitrate(2_500_000);
        encodings.push(&h_enc);

        // Medium quality
        let m_enc = web_sys::RtcRtpEncodingParameters::new();
        m_enc.set_rid("m");
        m_enc.set_max_bitrate(750_000);
        m_enc.set_scale_resolution_down_by(2.0);
        encodings.push(&m_enc);

        // Low quality
        let l_enc = web_sys::RtcRtpEncodingParameters::new();
        l_enc.set_rid("l");
        l_enc.set_max_bitrate(150_000);
        l_enc.set_scale_resolution_down_by(4.0);
        encodings.push(&l_enc);

        init.set_send_encodings(&encodings);

        let _ = self
            .connection
            .add_transceiver_with_media_stream_track_and_init(track, &init);
        Ok(())
    }

    pub fn close(&self) {
        self.connection.close();
    }

    pub fn set_audio_enabled(&self, enabled: bool) -> Result<()> {
        self.set_track_enabled("audio", enabled)
    }

    pub fn set_video_enabled(&self, enabled: bool) -> Result<()> {
        self.set_track_enabled("video", enabled)
    }

    fn set_track_enabled(&self, kind: &str, enabled: bool) -> Result<()> {
        let senders = self.connection.get_senders();
        for i in 0..senders.length() {
            let sender = Reflect::get(&senders, &i.into())
                .map_err(|e| MediaError::WebRtc(format!("Failed to get sender: {:?}", e)))?
                .dyn_into::<web_sys::RtcRtpSender>()
                .map_err(|_| MediaError::WebRtc("Invalid sender type".into()))?;

            if let Some(track) = sender.track() {
                if track.kind() == kind {
                    track.set_enabled(enabled);
                    info!("Set {} track enabled: {}", kind, enabled);
                }
            }
        }
        Ok(())
    }
    pub fn update_ice_servers(&self, stun_config: &StunConfig) -> Result<()> {
        let rtc_config = RtcConfiguration::new();
        let ice_servers = Array::new();

        for server_config in &stun_config.ice_servers {
            let server = RtcIceServer::new();

            let urls_array = Array::new();
            for url in &server_config.urls {
                urls_array.push(&JsValue::from_str(url));
            }
            server.set_urls(&urls_array);

            if let Some(username) = &server_config.username {
                server.set_username(username);
            }
            if let Some(credential) = &server_config.credential {
                server.set_credential(credential);
            }

            ice_servers.push(&server);
        }
        rtc_config.set_ice_servers(&ice_servers);

        // Use Reflect and JsCast to call setConfiguration to avoid web-sys version signature issues
        let set_config_fn = Reflect::get(&self.connection, &JsValue::from_str("setConfiguration"))
            .map_err(|e| MediaError::WebRtc(format!("Failed to get setConfiguration: {:?}", e)))?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| MediaError::WebRtc("setConfiguration is not a function".into()))?;

        let _ = set_config_fn
            .call1(&self.connection, &rtc_config)
            .map_err(|e| MediaError::WebRtc(format!("Failed to call setConfiguration: {:?}", e)))?;

        info!(
            "Updated RTCPeerConnection with {} ICE servers",
            stun_config.ice_servers.len()
        );
        Ok(())
    }

    pub fn set_onicecandidate<F>(&self, callback: F)
    where
        F: Fn(web_sys::RtcPeerConnectionIceEvent) + 'static,
    {
        let closure =
            Closure::wrap(Box::new(callback) as Box<dyn Fn(web_sys::RtcPeerConnectionIceEvent)>);
        self.connection
            .set_onicecandidate(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}
