use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Simple struct to test contains_key transpilation
struct MyMap;

impl MyMap {
    fn contains_key(&self, _key: &str) -> bool { false }
}

#[to_js]
fn test_contains_key_pattern() {
    let map = MyMap;
    let result = map.contains_key("test_key");
    println!("Result: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_key_generates_universal_iife() {
        println!("=== Testing contains_key IIFE pattern generation ===");
        
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
        
        // Verify it checks for Map .has() method
        assert!(
            js_code.contains("typeof obj.has === \"function\""),
            "JavaScript should check for Map .has() method existence"
        );
        
        // Verify it falls back to .hasOwnProperty()
        assert!(
            js_code.contains("obj.hasOwnProperty(key)"),
            "JavaScript should fall back to .hasOwnProperty() for Objects"
        );
        
        // Verify the IIFE is called with correct arguments
        assert!(
            js_code.contains("(map, \"test_key\")"),
            "JavaScript should call the IIFE with correct arguments"
        );
        
        println!("✅ SUCCESS: Universal contains_key IIFE pattern generated correctly!");
        println!("📋 Pattern: ((obj, key)=>obj && typeof obj.has === \"function\" ? obj.has(key) : obj.hasOwnProperty(key))(receiver, key)");
        println!("🗺️  For Maps: Uses obj.has(key) when typeof obj.has === 'function'");
        println!("🔤 For Objects: Falls back to obj.hasOwnProperty(key)");
        println!("⚡ Universal: Works with any collection type automatically");
    }
}