use crate::{MediaError, Result};
use js_sys::{Array, Reflect};
use log::info;
use video_chat_signaling::stun_config::StunConfig;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcConfiguration, RtcIceServer, RtcPeerConnection, RtcSessionDescriptionInit};

pub struct PeerConnectionManager {
    connection: RtcPeerConnection,
}

impl PeerConnectionManager {
    pub fn new(stun_config: &StunConfig) -> Result<Self> {
        let rtc_config = RtcConfiguration::new();

        let ice_servers = Array::new();
        for url in &stun_config.urls {
            let server = RtcIceServer::new();
            server.set_urls(&JsValue::from_str(url));
            ice_servers.push(&server);
        }
        rtc_config.set_ice_servers(&ice_servers);

        let connection = RtcPeerConnection::new_with_configuration(&rtc_config).map_err(|e| {
            MediaError::WebRtc(format!("Failed to create RTCPeerConnection: {:?}", e))
        })?;

        info!(
            "Created RTCPeerConnection with STUN servers: {:?}",
            stun_config.urls
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

    pub fn close(&self) {
        self.connection.close();
    }
}
