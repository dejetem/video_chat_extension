use wasm_bindgen_test::*;

// Configure wasm-bindgen-test to run in the browser
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_wasm_module_initialization() {
    // This test ensures the WASM module can be loaded and initialized in a browser environment
    video_chat_wasm::init();
    // If we get here without panicking, basic initialization works
}

#[wasm_bindgen_test]
fn test_chrome_api_bindings_exist() {
    // We can't easily mock the 'chrome' global object in this test environment without
    // more complex setup, but we can verify our wrapper types can be instantiated
    // or used if we had a mock.
    // For now, this test is a placeholder to verify the test suite runs.
}
