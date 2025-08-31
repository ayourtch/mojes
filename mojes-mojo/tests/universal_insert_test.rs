// FIXME: These tests are currently broken due to BOA engine API changes
// The .get() method now requires a Context parameter that wasn't needed before
// Need to update these tests to use the correct BOA API

/*
use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

#[test]
fn test_universal_insert_on_arrays() {
    // Test that insert() works correctly on arrays (should use splice)
    let expr: Expr = parse_quote!(arr.insert(1, "inserted"));
    let js_code = rust_expr_to_js(&expr);
    println!("Array insert code: {}", js_code);
    
    let test_code = format!(r#"
        let arr = ["a", "b", "c"];
        {};
        arr;
    "#, js_code);
    
    let result = eval_js(&test_code).unwrap();
    let array = result.as_object().unwrap();
    
    // Should be ["a", "inserted", "b", "c"] after inserting at index 1
    assert_eq!(array.get("length").unwrap().as_number().unwrap(), 4.0);
    assert_eq!(array.get("0").unwrap().as_string().unwrap(), "a");
    assert_eq!(array.get("1").unwrap().as_string().unwrap(), "inserted");
    assert_eq!(array.get("2").unwrap().as_string().unwrap(), "b");
    assert_eq!(array.get("3").unwrap().as_string().unwrap(), "c");
}

#[test]
fn test_universal_insert_on_objects() {
    // Test that insert() works correctly on HashMap-like objects (should use property assignment)
    let expr: Expr = parse_quote!(map.insert("key", "value"));
    let js_code = rust_expr_to_js(&expr);
    println!("Object insert code: {}", js_code);
    
    let test_code = format!(r#"
        let map = {{ existing: "data" }};
        {};
        map;
    "#, js_code);
    
    let result = eval_js(&test_code).unwrap();
    let object = result.as_object().unwrap();
    
    // Should have both existing and new key
    assert_eq!(object.get("existing").unwrap().as_string().unwrap(), "data");
    assert_eq!(object.get("key").unwrap().as_string().unwrap(), "value");
}

#[test]
fn test_universal_insert_detects_array_vs_object() {
    // Test that the IIFE correctly detects arrays vs objects
    let expr: Expr = parse_quote!(container.insert("test", 42));
    let js_code = rust_expr_to_js(&expr);
    
    // Test with array (should use splice)
    let array_test = format!(r#"
        let container = [1, 2, 3];
        {};
        container;
    "#, js_code);
    
    let result = eval_js(&array_test).unwrap();
    let array = result.as_object().unwrap();
    // "test" becomes index 0 in splice, so should insert at beginning
    assert_eq!(array.get("length").unwrap().as_number().unwrap(), 4.0);
    
    // Test with object (should use property assignment)
    let object_test = format!(r#"
        let container = {{ initial: "value" }};
        {};
        container;
    "#, js_code);
    
    let result = eval_js(&object_test).unwrap();
    let object = result.as_object().unwrap();
    assert_eq!(object.get("initial").unwrap().as_string().unwrap(), "value");
    assert_eq!(object.get("test").unwrap().as_number().unwrap(), 42.0);
}

#[test]
fn test_hashmap_webrtc_case() {
    // Test the specific WebRTC case that motivated this fix
    let expr: Expr = parse_quote!(self.peer_connections.insert(key, pc));
    let js_code = rust_expr_to_js(&expr);
    println!("WebRTC HashMap insert: {}", js_code);
    
    let test_code = format!(r#"
        const self = {{
            peer_connections: {{ existing: "connection1" }}
        }};
        const key = "participant123";
        const pc = {{ type: "RTCPeerConnection", id: "pc1" }};
        {};
        self.peer_connections;
    "#, js_code);
    
    let result = eval_js(&test_code).unwrap();
    let connections = result.as_object().unwrap();
    
    // Should have both existing and new peer connection
    assert_eq!(connections.get("existing").unwrap().as_string().unwrap(), "connection1");
    
    let new_pc = connections.get("participant123").unwrap().as_object().unwrap();
    assert_eq!(new_pc.get("type").unwrap().as_string().unwrap(), "RTCPeerConnection");
    assert_eq!(new_pc.get("id").unwrap().as_string().unwrap(), "pc1");
}
*/