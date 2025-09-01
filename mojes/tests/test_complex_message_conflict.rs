use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mirror the complex ServerMessage enum from the original code
#[js_type]
#[derive(Debug, Clone)]
enum ServerMessage {
    AuthSuccess { 
        client_id: String, 
        participants: Vec<String>, 
        auth_token: String,
        token_signature: String,
    },
    AuthFailed { message: String },
    ConferenceFull { message: String },
    Error { message: String },
    TestMessage { from: String, message: String },
}

#[js_type]
struct ConferenceApp;

#[js_object]
impl ConferenceApp {
    fn new() -> Self {
        Self
    }

    // This should reproduce the issue - multiple match arms with "message" variables
    fn app_handle_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::AuthSuccess { 
                client_id, 
                participants, 
                auth_token,
                token_signature,
            } => {
                println!("Authentication successful! Client ID: {}", client_id);
                println!("Participants: {}", participants.len());
            }
            
            ServerMessage::AuthFailed { message } => {
                println!("Authentication failed: {}", message);
                // This should cause variable conflict with the next match arm's "message"
            }
            
            ServerMessage::ConferenceFull { message } => {
                println!("Conference full: {}", message);
                // This "message" conflicts with the previous arm's "message" 
            }
            
            ServerMessage::Error { message } => {
                println!("Server error: {}", message);
                // This "message" also conflicts
            }
            
            ServerMessage::TestMessage { from, message } => {
                println!("Test message from {}: {}", from, message);
                println!("Again: {}", message); // This should show the conflict
                // This "message" should also conflict with previous "message" variables
            }
        }
    }
}

// Main test function
#[to_js]
pub fn test_complex_message_conflict() -> bool {
    println!("=== Testing Complex Message Variable Conflict ===");
    
    let mut app = ConferenceApp::new();
    
    // Test all the conflicting message cases
    app.app_handle_server_message(ServerMessage::AuthFailed { 
        message: "Auth failed".to_string() 
    });
    
    app.app_handle_server_message(ServerMessage::ConferenceFull { 
        message: "Conference is full".to_string() 
    });
    
    app.app_handle_server_message(ServerMessage::Error { 
        message: "Server error".to_string() 
    });
    
    app.app_handle_server_message(ServerMessage::TestMessage { 
        from: "user123".to_string(),
        message: "hello world".to_string()
    });
    
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
    fn test_complex_message_variable_conflict_transpilation() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_complex_message_conflict();
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
            println!("✅ REPRODUCED BUG: Found problematic message_1 variable reference in transpiled code!");
            
            // Find and print the problematic lines
            for (i, line) in full_js.lines().enumerate() {
                if line.contains("message_1") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
            panic!("SUCCESSFULLY REPRODUCED BUG: Variable conflict detected - 'message' was renamed to 'message_1' but usage sites weren't updated consistently");
        } else {
            println!("❌ Bug not reproduced - no message_1 variable conflicts found");
            
            // Show the actual transpiled code for analysis
            println!("\n=== Full Transpiled JavaScript ===");
            println!("{}", full_js);
        }
        
        // Also try to execute the JavaScript to see if it works
        println!("\n=== Testing JavaScript Execution ===");
        let test_js = format!(
            r#"
            {}
            
            // Test execution
            try {{
                test_complex_message_conflict();
                console.log("✅ Test executed successfully");
            }} catch (e) {{
                console.log("❌ Test execution error:", e.message);
                if (e.message.includes("message_1")) {{
                    throw new Error("REPRODUCED BUG: Found message_1 variable reference error: " + e.message);
                }}
                throw e;
            }}
            "#,
            full_js
        );
        
        match eval_js_with_context(&test_js) {
            Ok(_) => {
                if full_js.contains("message_1") {
                    panic!("REPRODUCED BUG: JavaScript contains message_1 references but still executed");
                } else {
                    println!("✅ JavaScript executed successfully with no message_1 conflicts");
                }
            }
            Err(e) => {
                let error_str = format!("{:?}", e);
                if error_str.contains("message_1") {
                    panic!("SUCCESSFULLY REPRODUCED BUG: JavaScript execution failed due to message_1 variable conflict: {:?}", e);
                } else {
                    println!("JavaScript execution failed for different reason: {:?}", e);
                }
            }
        }
    }
}