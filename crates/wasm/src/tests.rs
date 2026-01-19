use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_to_js_value() {
    let val = "test";
    let js_val = crate::js_interop::to_js_value(&val).unwrap();
    assert!(js_val.is_string());
    assert_eq!(js_val.as_string().unwrap(), "test");
}

#[wasm_bindgen_test]
fn test_from_js_value() {
    let js_val = JsValue::from_str("test");
    let val: String = crate::js_interop::from_js_value(js_val).unwrap();
    assert_eq!(val, "test");
}
