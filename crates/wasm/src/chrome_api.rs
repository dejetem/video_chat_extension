#![allow(deprecated)]
#![allow(dead_code)]
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(deprecated)]
#[allow(dead_code)]
extern "C" {
    // Top-level chrome object
    pub type Chrome;
    #[wasm_bindgen(js_name = chrome)]
    #[allow(deprecated)]
    #[allow(dead_code)]
    pub static CHROME: Chrome;

    // chrome.runtime
    pub type Runtime;
    #[wasm_bindgen(method, getter)]
    pub fn runtime(this: &Chrome) -> Runtime;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(this: &Runtime, message: &JsValue, response_callback: Option<&JsValue>);

    #[wasm_bindgen(method, getter, js_name = onMessage)]
    pub fn on_message(this: &Runtime) -> ChromeEvent;

    // chrome.tabs
    pub type Tabs;
    #[wasm_bindgen(method, getter)]
    pub fn tabs(this: &Chrome) -> Tabs;

    #[wasm_bindgen(method, js_name = query)]
    pub fn query(this: &Tabs, query_info: &JsValue, callback: &JsValue);

    #[wasm_bindgen(method, js_name = create)]
    pub fn create(this: &Tabs, create_properties: &JsValue, callback: Option<&JsValue>);

    // chrome.storage
    pub type Storage;
    #[wasm_bindgen(method, getter)]
    pub fn storage(this: &Chrome) -> Storage;

    #[wasm_bindgen(method, getter)]
    pub fn local(this: &Storage) -> StorageArea;

    #[wasm_bindgen(method, getter)]
    pub fn sync(this: &Storage) -> StorageArea;

    // Generic Event
    pub type ChromeEvent;
    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &ChromeEvent, callback: &JsValue);

    #[wasm_bindgen(method, js_name = removeListener)]
    pub fn remove_listener(this: &ChromeEvent, callback: &JsValue);

    // StorageArea
    pub type StorageArea;
    #[wasm_bindgen(method)]
    pub fn get(this: &StorageArea, keys: &JsValue, callback: &JsValue);

    #[wasm_bindgen(method)]
    pub fn set(this: &StorageArea, items: &JsValue, callback: Option<&JsValue>);
}
