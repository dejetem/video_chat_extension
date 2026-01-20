use crate::utils;
use video_chat_media::PeerConnectionManager;
use video_chat_signaling::stun_config;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MediaController {
    pc_manager: PeerConnectionManager,
}

#[wasm_bindgen]
impl MediaController {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<MediaController, JsValue> {
        let stun_config = stun_config::default_stun_config();
        let pc_manager = PeerConnectionManager::new(&stun_config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self { pc_manager })
    }

    pub async fn create_offer(&self) -> Result<String, JsValue> {
        self.pc_manager
            .create_offer()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn create_answer(&self) -> Result<String, JsValue> {
        self.pc_manager
            .create_answer()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn set_remote_description(&self, sdp: &str, is_offer: bool) -> Result<(), JsValue> {
        let sdp_type = if is_offer {
            web_sys::RtcSdpType::Offer
        } else {
            web_sys::RtcSdpType::Answer
        };

        self.pc_manager
            .set_remote_description(sdp, sdp_type)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
