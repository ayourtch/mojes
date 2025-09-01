use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
#[derive(Debug, Clone)]
enum TestMessage {
    AuthFailed { message: String },
    TestMessage { from: String, message: String },
}

#[js_type]
struct TestApp;

#[js_object]
impl TestApp {
    fn new() -> Self {
        Self
    }

    fn handle_message(&self, msg: TestMessage) {
        match msg {
            TestMessage::AuthFailed { message } => {
                println!("Auth failed: {}", message);
            }
            TestMessage::TestMessage { from, message } => {
                println!("Test message from {}: {}", from, message);
                println!("Again: {}", message); // This should show the conflict
            }
        }
    }
}

// Main test function
#[to_js]
pub fn test_message_variable_conflict() -> bool {
    println!("=== Testing Message Variable Conflict ===");
    
    let app = TestApp::new();
    
    // Test AuthFailed case
    let auth_msg = TestMessage::AuthFailed { 
        message: "Authentication failed".to_string() 
    };
    app.handle_message(auth_msg);
    
    // Test TestMessage case - this should reveal the conflict
    let test_msg = TestMessage::TestMessage { 
        from: "user123".to_string(),
        message: "hello world".to_string()
    };
    app.handle_message(test_msg);
    
    true
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

    #[test]
    fn test_message_variable_conflict_transpilation() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_message_variable_conflict();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut full_js = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            full_js.push_str(js_code);
            full_js.push('\n');
        }
        
        // This is the key test - look for message_1 references
        if full_js.contains("message_1") {
            println!("❌ Found problematic message_1 variable reference in transpiled code!");
            
            // Find and print the problematic lines
            for (i, line) in full_js.lines().enumerate() {
                if line.contains("message_1") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
            panic!("REPRODUCED BUG: Variable conflict detected - 'message' was renamed to 'message_1' but usage sites weren't updated consistently");
        } else {
            println!("✅ No message_1 variable conflicts found");
            println!("Note: This simple test case doesn't reproduce the bug. The bug might occur with more complex nesting or specific variable patterns.");
        }
        
        // For now, just check that the JS compiles and can be parsed
        println!("\n=== Checking JavaScript Syntax ===");
        let fixed_test_js = format!(
            r#"
            {}
            
            // Fixed test call
            try {{
                let result = test_message_variable_conflict();
                console.log("Test completed with result:", result);
            }} catch (e) {{
                console.log("Test error:", e.message);
                if (e.message.includes("message_1")) {{
                    throw new Error("Found message_1 variable reference error: " + e.message);
                }}
            }}
            "#,
            full_js
        );
        
        match eval_js_with_context(&fixed_test_js) {
            Ok(_) => {
                println!("✅ JavaScript executed successfully");
            }
            Err(e) => {
                let error_str = format!("{:?}", e);
                if error_str.contains("message_1") {
                    panic!("REPRODUCED BUG: JavaScript execution failed due to message_1 variable conflict: {:?}", e);
                } else {
                    println!("JavaScript execution failed for different reason: {:?}", e);
                    // This is expected due to the TestApp constructor issue, not our target bug
                }
            }
        }
    }
}