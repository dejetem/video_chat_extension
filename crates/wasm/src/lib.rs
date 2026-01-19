mod chrome_api;
mod js_interop;
mod schema;
mod sqlite;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
    utils::init_logging();
    log::info!("Video Chat Extension WASM initialized");
}
