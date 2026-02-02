use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

/// Convert a Rust type to a JsValue.
#[allow(dead_code)]
pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert a JsValue to a Rust type.
#[allow(dead_code)]
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
        // Use global() instead of window() because Service Workers don't have a window
        let global = js_sys::global();
        if let Ok(chrome) = js_sys::Reflect::get(&global, &"chrome".into()) {
            if !chrome.is_undefined() {
                if let Ok(runtime) = js_sys::Reflect::get(&chrome, &"runtime".into()) {
                    if !runtime.is_undefined() {
                        // Reliable check: Are we on a chrome-extension:// page?
                        let mut is_ext = false;
                        if let Ok(location) = js_sys::Reflect::get(&global, &"location".into()) {
                            if let Ok(protocol) =
                                js_sys::Reflect::get(&location, &"protocol".into())
                            {
                                if let Some(p) = protocol.as_string() {
                                    if p == "chrome-extension:" {
                                        is_ext = true;
                                    }
                                }
                            }
                        }

                        if is_ext {
                            crate::chrome_api::CHROME.with(|c| {
                                c.runtime().send_message(&js_val, None);
                            });
                            return;
                        }
                    }
                }
            }
        }

        // Fallback: Dispatch to window (for standard web portal)
        if let Ok(window) = web_sys::window().ok_or("No window") {
            log::debug!("Dispatching CustomEvent to window: {}", event_type);

            // Create a CustomEvent with the payload in the 'detail' field
            let event_init = web_sys::CustomEventInit::new();
            event_init.set_detail(&js_val);

            if let Ok(event) =
                web_sys::CustomEvent::new_with_event_init_dict("rust-video-chat-event", &event_init)
            {
                let _ = window.dispatch_event(&event);
            }
        } else {
            log::warn!(
                "Nowhere to dispatch message: {}. Extension API missing and no window found.",
                event_type
            );
        }
    }
}
