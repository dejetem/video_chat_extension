use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum DataChannelMessage {
    Chat {
        sender_id: String,
        content: String,
        timestamp: i64,
    },
    Typing {
        sender_id: String,
        is_typing: bool,
    },
    // Future expansion for SFU signaling over DataChannel if needed
    Control(serde_json::Value),
}

impl DataChannelMessage {
    pub fn new_chat(sender_id: String, content: String) -> Self {
        #[cfg(target_arch = "wasm32")]
        let timestamp = js_sys::Date::now() as i64;
        #[cfg(not(target_arch = "wasm32"))]
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self::Chat {
            sender_id,
            content,
            timestamp,
        }
    }
}
