#[cfg(test)]
mod webrtc_constructor_tests {
    use mojes_mojo::*;
    use syn::{ItemImpl, parse_quote};

    #[test]
    fn test_self_struct_in_constructor_method() {
        // Test the WebRTC Manager case where Self { ... } should use constructor + field assignment
        let input_impl: ItemImpl = parse_quote! {
            impl WebRTCManager {
                pub fn new() -> Self {
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
        println!("DEBUG WebRTC constructor JS code: {}", &js_code);

        // Check that Self { ... } uses constructor + field assignment for proper prototype chain
        assert!(js_code.contains("new WebRTCManager()"), "Should use 'new WebRTCManager()' constructor");
        assert!(js_code.contains("obj.peer_connections = {}"), "Should assign peer_connections field");
        assert!(js_code.contains("obj.local_stream = null"), "Should assign local_stream field");
        assert!(js_code.contains("obj.turn_config = null"), "Should assign turn_config field");
        
        // Make sure it's NOT using old object literal pattern
        assert!(!js_code.contains("new Self("), "Should not use 'new Self(' constructor");
        assert!(!js_code.contains("peer_connections: {}"), "Should not use object literal pattern");
        assert!(!js_code.contains("new HashMap("), "Should not use 'new HashMap(' constructor");
    }

    #[test]
    fn test_regular_struct_still_uses_constructor() {
        // Test that regular struct expressions still use constructors (not Self)
        let input_impl: ItemImpl = parse_quote! {
            impl SomeClass {
                pub fn create_point() -> Point {
                    Point { x: 10, y: 20 }
                }
            }
        };

        let js_code = generate_js_methods_for_impl(&input_impl);
        println!("DEBUG regular struct constructor JS code: {}", &js_code);

        // Regular structs should still use constructor pattern
        assert!(js_code.contains("new Point("), "Regular structs should use constructor pattern");
    }
}