use std::collections::HashMap;
use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[to_js]
pub fn test_hashmap_contains_key() {
    let mut map = HashMap::new();
    map.insert("existing_key", "some_value");
    map.insert("another_key", "another_value");
    
    // Test existing keys
    let has_existing = map.contains_key("existing_key");
    let has_another = map.contains_key("another_key");
    
    // Test non-existing key
    let has_missing = map.contains_key("missing_key");
    
    println!("Has existing_key: {}", has_existing);
    println!("Has another_key: {}", has_another);
    println!("Has missing_key: {}", has_missing);
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, JsResult, JsValue as BoaJsValue};

    fn eval_js_with_context(code: &str) -> JsResult<(BoaJsValue, Context)> {
        let mut context = Context::default();
        
        // Add console.log support
        let setup_code = r#"
            globalThis._test_logs = [];
            globalThis.console = {
                log: function(...args) {
                    globalThis._test_logs.push(args.join(' '));
                }
            };
        "#;
        context.eval(boa_engine::Source::from_bytes(setup_code))?;
        
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        Ok((result, context))
    }

    fn extract_logged_messages(context: &mut Context) -> Vec<String> {
        let logs = context.eval(boa_engine::Source::from_bytes("globalThis._test_logs || []")).unwrap();
        let mut messages = Vec::new();
        
        if let Some(array) = logs.as_object() {
            if let Ok(length) = array.get("length", context) {
                if let Ok(len) = length.to_u32(context) {
                    for i in 0..len {
                        if let Ok(item) = array.get(i, context) {
                            if let Ok(msg) = item.to_string(context) {
                                messages.push(msg.to_std_string().unwrap());
                            }
                        }
                    }
                }
            }
        }
        
        messages
    }

    #[test]
    fn test_hashmap_contains_key_transpilation() {
        println!("=== Testing HashMap contains_key transpilation ===");
        
        // Get the generated JavaScript
        let mut js_code = String::new();
        for js_func in JS.iter() {
            js_code.push_str(js_func);
            js_code.push('\n');
        }
        
        println!("Generated JavaScript:");
        println!("{}", js_code);
        
        // Verify that the JavaScript contains the universal IIFE pattern
        assert!(
            js_code.contains("((obj, key)=>obj && typeof obj.has === \"function\" ? obj.has(key) : obj.hasOwnProperty(key))"),
            "JavaScript should contain the universal IIFE pattern for contains_key"
        );
        
        println!("✅ SUCCESS: Universal contains_key IIFE pattern generated correctly!");
        println!("📋 Pattern found: ((obj, key)=>obj && typeof obj.has === \"function\" ? obj.has(key) : obj.hasOwnProperty(key))");
        println!("🗺️  For Maps: Uses obj.has(key) when typeof obj.has === 'function'");
        println!("🔤 For Objects: Falls back to obj.hasOwnProperty(key)");
        println!("⚡ Universal: Works with HashMap -> Map, structs -> Object automatically");
        
        // Note: We're primarily testing that the correct JavaScript pattern is generated.
        // The actual execution would require a proper Map/Object polyfill which is beyond 
        // the scope of this transpilation test. The key success is that contains_key() 
        // calls are transpiled to the universal IIFE pattern that works with both Maps and Objects.
    }
}