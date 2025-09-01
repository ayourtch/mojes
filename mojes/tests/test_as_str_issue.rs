use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mock RTCPeerConnection with connectionState property
#[js_type]
#[derive(Debug, Clone)]
pub struct RTCPeerConnection {
    pub connectionState: String,
}

#[js_object]
impl RTCPeerConnection {
    pub fn new() -> Self {
        Self {
            connectionState: "new".to_string(),
        }
    }

    pub fn set_connection_state(&mut self, state: &str) {
        self.connectionState = state.to_string();
    }

    pub fn addEventListener(&self, _event_type: &str, _callback: impl FnOnce()) {
        // Mock implementation - in real code this would register the callback
        println!("addEventListener called");
    }
}

// We'll use println! for Rust and let it transpile to console.log for JavaScript

#[js_type]
#[derive(Debug, Clone)]
pub struct ConnectionStateTest {
    pub participant_id: String,
}

#[js_object]
impl ConnectionStateTest {
    pub fn new(participant_id: String) -> Self {
        Self { participant_id }
    }

    // This method demonstrates the .as_str() issue
    pub fn test_connection_state_handling(&self) -> bool {
        let mut pc = RTCPeerConnection::new();
        let participant_id_clone = self.participant_id.clone();
        
        // Simulate different connection states
        let states = vec!["new", "connecting", "connected", "disconnected", "failed", "closed"];
        let mut all_passed = true;

        for state in states {
            pc.set_connection_state(state);
            let connection_state = &pc.connectionState;
            
            println!("🔌 PEER CONNECTION STATE CHANGED for {}: → {}", participant_id_clone, connection_state);

            // This is the problematic code - .as_str() on a String field
            match connection_state.as_str() {
                "new" => {
                    println!("  └─ {} Peer connection created, waiting for ICE", participant_id_clone);
                },
                "connecting" => {
                    println!("  └─ {} Peer connection is connecting", participant_id_clone);
                },
                "connected" => {
                    println!("  └─ {} Peer connection established successfully", participant_id_clone);
                },
                "disconnected" => {
                    println!("  └─ {} Peer connection temporarily lost", participant_id_clone);
                },
                "failed" => {
                    println!("  └─ {} Peer connection failed permanently", participant_id_clone);
                    // Note: Failed state is expected, don't mark as failure for this test
                },
                "closed" => {
                    println!("  └─ {} Peer connection closed", participant_id_clone);
                },
                unknown => {
                    println!("  └─ {} Unknown connection state: {}", participant_id_clone, unknown);
                    all_passed = false;
                }
            }
        }

        all_passed
    }

    // Test with event listener callback - more realistic scenario
    pub fn test_event_listener_with_as_str(&self) -> bool {
        let mut pc = RTCPeerConnection::new();
        let participant_id_clone3 = self.participant_id.clone();
        
        // Set a connection state to test
        pc.set_connection_state("connecting");
        
        // Simulate getting the connection state in a callback context
        let connection_state = pc.connectionState.clone();
        println!("🔌 PEER CONNECTION STATE CHANGED for {}: → {}", participant_id_clone3, connection_state);

        // This .as_str() call should fail in JavaScript transpilation
        match connection_state.as_str() {
            "new" => {
                println!("  └─ {} Peer connection created, waiting for ICE", participant_id_clone3);
            },
            "connecting" => {
                println!("  └─ {} Now connecting...", participant_id_clone3);
            },
            _ => {
                println!("  └─ {} Other state: {}", participant_id_clone3, connection_state);
            }
        }

        true
    }
}

// Main test function
#[to_js]
pub fn test_as_str_functionality() -> bool {
    println!("=== Testing .as_str() Method Transpilation ===");
    
    let test_manager = ConnectionStateTest::new("user123".to_string());
    let mut all_passed = true;
    
    println!("\n🧪 Test 1: Direct .as_str() usage");
    if !test_manager.test_connection_state_handling() {
        all_passed = false;
    }
    
    println!("\n🧪 Test 2: .as_str() in event listener callback");
    if !test_manager.test_event_listener_with_as_str() {
        all_passed = false;
    }
    
    println!("\n=== Test Results ===");
    if all_passed {
        println!("🎉 All .as_str() tests PASSED!");
    } else {
        println!("💥 Some .as_str() tests FAILED!");
    }
    
    all_passed
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source, JsResult, JsValue};
    
    // Helper to evaluate JavaScript with context and get result
    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        
        // Add console.log support
        let console_log = |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> JsResult<JsValue> {
            let message = args.iter()
                .map(|arg| arg.to_string(ctx).unwrap().to_std_string().unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            println!("JS Console: {}", message);
            Ok(JsValue::undefined())
        };
        
        let console_obj = boa_engine::object::ObjectInitializer::new(&mut context)
            .function(
                boa_engine::native_function::NativeFunction::from_fn_ptr(console_log),
                "log",
                0
            )
            .build();
        
        context.register_global_property(
            "console", 
            console_obj, 
            boa_engine::property::Attribute::all()
        ).unwrap();
        
        let result = context.eval(Source::from_bytes(code))?;
        Ok((result, context))
    }
    
    // Helper to convert JsValue to string
    fn js_to_string(value: &JsValue, ctx: &mut Context) -> String {
        value.to_string(ctx).unwrap().to_std_string().unwrap()
    }
    
    // Helper to convert JsValue to boolean  
    fn js_to_boolean(value: &JsValue, _ctx: &mut Context) -> bool {
        value.as_boolean().unwrap_or(false)
    }
    
    #[test]
    fn test_as_str_transpilation_issue() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_as_str_functionality();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut class_code = String::new();
        let mut method_code = String::new();  
        let mut function_code = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class ") {
                class_code.push_str(js_code);
                class_code.push('\n');
            } else if js_code.contains(".prototype.") || js_code.contains(" = function(") {
                method_code.push_str(js_code);
                method_code.push('\n');
            } else if js_code.contains("function test_as_str_functionality") {
                function_code.push_str(js_code);
                function_code.push('\n');
            }
        }
        
        // Test 1: Check if the generated JavaScript is syntactically valid
        println!("\n=== Testing JavaScript Syntax ===");
        let full_js = format!("{}\n{}\n{}\n\n// Call the test function\ntest_as_str_functionality();", class_code, method_code, function_code);
        
        println!("\n=== FULL GENERATED JAVASCRIPT ===");
        println!("{}", full_js);
        println!("=== END OF GENERATED JAVASCRIPT ===\n");
        
        match eval_js_with_context(&full_js) {
            Ok((result, mut ctx)) => {
                println!("✅ JavaScript syntax is valid");
                let js_result = js_to_boolean(&result, &mut ctx);
                println!("JavaScript test result: {}", js_result);
                
                if js_result {
                    println!("🎉 .as_str() works correctly in JavaScript!");
                } else {
                    println!("❌ .as_str() test failed in JavaScript execution");
                    panic!(".as_str() test failed in JavaScript execution");
                }
            }
            Err(e) => {
                println!("❌ JavaScript execution error: {:?}", e);
                println!("Generated JavaScript:\n{}", full_js);
                panic!(".as_str() method caused JavaScript execution error: {:?}", e);
            }
        }
    }
}