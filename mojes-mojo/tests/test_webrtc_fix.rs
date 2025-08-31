use std::collections::HashMap;
use mojes_mojo::*;
use syn::parse_quote;

fn main() {
    println!("Testing WebRTC Manager transpilation fixes...");
    
    // Test the WebRTCManager impl that was causing issues
    let input_impl = parse_quote! {
        impl WebRTCManager {
            pub fn make() -> Self {
                console.log("Initializing WebRTC Manager");
                let manager = Self {
                    peer_connections: HashMap::new(),
                    local_stream: None,
                    turn_config: None,
                };
                manager
            }
        }
    };

    let js_code = generate_js_methods_for_impl(&input_impl);
    println!("Generated JavaScript:");
    println!("{}", js_code);
    
    // Check that the new constructor + field assignment pattern is used
    assert!(!js_code.contains("new Self("), "Self should not appear as constructor");
    assert!(!js_code.contains("new HashMap("), "HashMap should not appear as constructor");
    assert!(js_code.contains("new WebRTCManager()"), "Should use proper constructor");
    assert!(js_code.contains("obj.peer_connections = {}"), "Should assign fields individually");
    assert!(js_code.contains("obj.local_stream = null"), "Should assign local_stream field");
    assert!(js_code.contains("obj.turn_config = null"), "Should assign turn_config field");
    assert!(!js_code.contains("peer_connections: {}"), "Should not use old object literal pattern");
    
    println!("✅ All WebRTC Manager transpilation fixes work correctly!");
}