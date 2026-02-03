// WASM integration tests
// Note: These tests verify the WASM module structure and integration points
// Full browser-based tests would require wasm-pack test

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_module_initialization() {
    // Test that the WASM module can be initialized
    video_chat_wasm::init();
    // If we get here without panicking, initialization succeeded
}

#[wasm_bindgen_test]
fn test_create_room() {
    video_chat_wasm::init();
    video_chat_wasm::create_room("test-room-123".to_string());
    // Verify no panic occurs
}

#[wasm_bindgen_test]
fn test_join_room() {
    video_chat_wasm::init();
    video_chat_wasm::join_room("test-room-456".to_string());
    // Verify no panic occurs
}

#[wasm_bindgen_test]
fn test_toggle_microphone() {
    video_chat_wasm::init();
    video_chat_wasm::toggle_microphone(true);
    video_chat_wasm::toggle_microphone(false);
    // Verify no panic occurs
}

#[wasm_bindgen_test]
fn test_toggle_camera() {
    video_chat_wasm::init();
    video_chat_wasm::toggle_camera(true);
    video_chat_wasm::toggle_camera(false);
    // Verify no panic occurs
}

#[wasm_bindgen_test]
fn test_send_message() {
    video_chat_wasm::init();
    video_chat_wasm::send_message("Hello from WASM test!".to_string());
    // Verify no panic occurs
}

// Note: Full SQLite WASM tests would require browser environment
// These are smoke tests to ensure the API surface is correct
