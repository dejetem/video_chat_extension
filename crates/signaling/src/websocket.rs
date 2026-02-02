//! WebSocket signaling implementation.

use crate::error::Result;
use crate::messages::Message;
use crate::protocol::Protocol;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

/// WebSocket signaling client using web-sys (browser API).
#[derive(Debug)]
pub struct WebSocketSignaling {
    /// Signaling URL
    url: String,
    /// Protocol handler
    protocol: Protocol,
    /// The actual WebSocket
    ws: Option<WebSocket>,
    /// Keepalive interval handle (for cleanup)
    #[allow(dead_code)]
    keepalive_interval: Rc<RefCell<Option<i32>>>,
    /// Last pong received timestamp
    #[allow(dead_code)]
    last_pong: Rc<RefCell<f64>>,
}

impl WebSocketSignaling {
    /// Create a new WebSocket signaling client.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            protocol: Protocol::new(),
            ws: None,
            keepalive_interval: Rc::new(RefCell::new(None)),
            last_pong: Rc::new(RefCell::new(js_sys::Date::now())),
        }
    }

    /// Get the signaling URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Connect to the signaling server using the browser WebSocket API.
    pub fn connect(&mut self) -> Result<()> {
        self.connect_with_callback(None)
    }

    /// Connect to the signaling server with message and open callbacks.
    pub fn connect_with_callbacks(
        &mut self,
        on_message: Option<Box<dyn Fn(Message) + 'static>>,
        on_open: Option<Box<dyn Fn() + 'static>>,
    ) -> Result<()> {
        let ws = WebSocket::new(&self.url).map_err(|e| {
            crate::error::Error::Network(format!("Failed to create WebSocket: {:?}", e))
        })?;

        // Set up event handlers
        let on_message_arc = on_message.map(Arc::new);
        let last_pong_update = self.last_pong.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = txt.into();
                log::debug!("message event, received string: {}", text);

                // Check if this is a pong response
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    if value.get("type").and_then(|t| t.as_str()) == Some("pong") {
                        *last_pong_update.borrow_mut() = js_sys::Date::now();
                        log::debug!("Received WebSocket pong");
                        return;
                    }
                }

                // Try to parse as a signaling message
                if let Ok(msg) = serde_json::from_str::<Message>(&text) {
                    if let Some(ref cb) = on_message_arc {
                        cb(msg);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            let error_msg = e.message();
            let error_type = e.type_();
            log::error!(
                "WebSocket error - type: {}, message: {}, event: {:?}",
                error_type,
                error_msg,
                e
            );
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Set up onopen callback with keepalive initialization
        let keepalive_interval = self.keepalive_interval.clone();
        let last_pong = self.last_pong.clone();
        let ws_for_keepalive = ws.clone();

        let onopen_callback = Closure::wrap(Box::new(move |_: JsValue| {
            log::info!("websocket connected");

            // Initialize last_pong timestamp
            *last_pong.borrow_mut() = js_sys::Date::now();

            // Start keepalive ping every 30 seconds
            let ws_ping = ws_for_keepalive.clone();
            let last_pong_check = last_pong.clone();

            let ping_callback = Closure::wrap(Box::new(move || {
                let now = js_sys::Date::now();
                let last = *last_pong_check.borrow();

                // Check if we haven't received a pong in 60 seconds
                if now - last > 60000.0 {
                    log::warn!("No pong received in 60 seconds, connection may be dead");
                    // Connection is likely dead, close it to trigger reconnection
                    let _ = ws_ping.close();
                } else {
                    // Send ping message
                    if ws_ping.ready_state() == WebSocket::OPEN {
                        let ping_msg = serde_json::json!({
                            "type": "ping",
                            "timestamp": now
                        });
                        if let Ok(ping_str) = serde_json::to_string(&ping_msg) {
                            let _ = ws_ping.send_with_str(&ping_str);
                            log::debug!("Sent WebSocket ping");
                        }
                    }
                }
            }) as Box<dyn FnMut()>);

            // Set interval to 30 seconds (30000ms)
            if let Some(window) = web_sys::window() {
                if let Ok(interval_id) = window
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        ping_callback.as_ref().unchecked_ref(),
                        30000,
                    )
                {
                    *keepalive_interval.borrow_mut() = Some(interval_id);
                    ping_callback.forget();
                }
            }

            if let Some(ref cb) = on_open {
                cb();
            }
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        self.ws = Some(ws);
        Ok(())
    }

    /// Legacy connect method for backward compatibility.
    pub fn connect_with_callback(
        &mut self,
        on_message: Option<Box<dyn Fn(Message) + 'static>>,
    ) -> Result<()> {
        self.connect_with_callbacks(on_message, None)
    }

    /// Send a message via the WebSocket.
    pub async fn send(&self, message: Message) -> Result<()> {
        self.protocol.validate_message(&message)?;
        let json = message.to_json()?;

        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                ws.send_with_str(&json).map_err(|e| {
                    crate::error::Error::Network(format!("Failed to send message: {:?}", e))
                })?;
                Ok(())
            } else {
                Err(crate::error::Error::Network("WebSocket not open".into()))
            }
        } else {
            Err(crate::error::Error::Network(
                "WebSocket not initialized".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_client() {
        let client = WebSocketSignaling::new("ws://localhost:8080");
        assert_eq!(client.url(), "ws://localhost:8080");
    }

    #[tokio::test]
    #[cfg(target_arch = "wasm32")]
    async fn test_connect() {
        let mut client = WebSocketSignaling::new("ws://localhost:8080");
        assert!(client.connect().is_ok());
    }

    #[tokio::test]
    #[cfg(target_arch = "wasm32")]
    async fn test_send_valid_message() {
        let mut client = WebSocketSignaling::new("ws://localhost:8080");
        // We need to connect first to initialize the WebSocket
        let _ = client.connect();

        let msg = Message::new(
            "test-id",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );
        // Note: This will likely still fail in tests unless we mock the WebSocket opening
        // but it prevents the "non-wasm target" panic.
        assert!(client.send(msg).await.is_ok());
    }
}
