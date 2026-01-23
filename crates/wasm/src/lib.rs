mod chrome_api;
mod js_interop;
pub mod media;
mod schema;
mod sqlite;
mod utils;

#[cfg(test)]
mod tests;

use wasm_bindgen::prelude::*;

pub mod sqlite;

use crate::sqlite::Database;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static DB: Lazy<Mutex<Option<Database>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
    utils::init_logging();
    log::info!("Video Chat Extension WASM initialized");

    match Database::open_persistent("video_chat_logs.db") {
        Ok(db) => {
            let mut guard = DB.lock().unwrap();
            *guard = Some(db);
            log::info!("SQLite database initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize SQLite database: {}", e);
        }
    }
}

#[wasm_bindgen]
pub fn create_room(room_id: String) {
    log::info!("Creating room: {}", room_id);
    // Future: Initialize SfuClient and store in global state
    js_interop::dispatch_event("roomCreated", &room_id);
}

#[wasm_bindgen]
pub fn join_room(room_id: String) {
    log::info!("Joining room: {}", room_id);
    js_interop::dispatch_event("roomJoined", &room_id);
}

#[wasm_bindgen]
pub fn toggle_microphone(enabled: bool) {
    log::info!("Microphone enabled: {}", enabled);
    js_interop::dispatch_event("mediaStatus", &format!("mic:{}", enabled));
}

#[wasm_bindgen]
pub fn toggle_camera(enabled: bool) {
    log::info!("Camera enabled: {}", enabled);
    js_interop::dispatch_event("mediaStatus", &format!("cam:{}", enabled));
}

#[wasm_bindgen]
pub fn send_message(text: String) {
    log::info!("Sending message: {}", text);
    // Future: Use SfuClient to send via DataChannel
    js_interop::dispatch_event("messageSent", &text);
}
