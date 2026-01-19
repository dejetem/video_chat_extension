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
