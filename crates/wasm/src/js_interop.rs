use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

/// Convert a Rust type to a JsValue.
#[allow(dead_code)]
pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert a JsValue to a Rust type.
pub fn from_js_value<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Dispatch an event to the Chrome extension runtime.
pub fn dispatch_event<T: Serialize>(event_type: &str, payload: &T) {
    #[derive(Serialize)]
    struct EventWrapper<'a, P: Serialize> {
        event: &'a str,
        data: &'a P,
    }

    let wrapper = EventWrapper {
        event: event_type,
        data: payload,
    };

    if let Ok(js_val) = to_js_value(&wrapper) {
        // Safe check for Chrome extension environment
        let window = web_sys::window().unwrap();
        if let Ok(chrome) = js_sys::Reflect::get(&window, &"chrome".into()) {
            if !chrome.is_undefined() {
                if let Ok(runtime) = js_sys::Reflect::get(&chrome, &"runtime".into()) {
                    if !runtime.is_undefined() {
                        if let Ok(send_message) =
                            js_sys::Reflect::get(&runtime, &"sendMessage".into())
                        {
                            if !send_message.is_undefined() {
                                crate::chrome_api::CHROME.with(|c| {
                                    c.runtime().send_message(&js_val, None);
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }
        log::warn!(
            "Chrome runtime.sendMessage not available, skipping message: {}",
            event_type
        );
    }
}
