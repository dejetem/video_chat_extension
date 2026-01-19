use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For this build, we are manually setting it up if we added the dependency,
    // but for now we'll just use a simple logger hook or rely on default wasm-bindgen behavior
    // if the crate isn't present.
    // A more robust implementation often uses the `console_error_panic_hook` crate.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn init_logging() {
    // Initialize standard logging to console
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("WASM logging initialized");
}
