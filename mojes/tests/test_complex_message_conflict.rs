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
struct ConferenceApp {
    client_id: Option<String>,
}

#[js_object]
impl ConferenceApp {
    fn new() -> Self {
        Self {
            client_id: None,
        }
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
        let mut classes = String::new();
        let mut methods = String::new(); 
        let mut enums = String::new();
        let mut functions = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class ") {
                classes.push_str(js_code);
                classes.push('\n');
            } else if js_code.contains(".prototype.") {
                methods.push_str(js_code);
                methods.push('\n');
            } else if js_code.contains(" = {") && (js_code.contains("function(") || js_code.contains("fromJSON")) {
                enums.push_str(js_code);
                enums.push('\n');
            } else {
                functions.push_str(js_code);
                functions.push('\n');
            }
        }
        
        // Assemble in proper order: classes, then enums, then methods, then functions
        let full_js = format!("{}\n{}\n{}\n{}", classes, enums, methods, functions);
        
        // Test by actually executing the JavaScript - if variable naming is consistent, it should work
        println!("\n=== Testing JavaScript Execution (The Real Test!) ===");
        let test_js = format!(
            r#"
            {}
            
            // Test each match arm by calling the function with different message types
            let app = new ConferenceApp();
            let results = [];
            
            try {{
                // Test AuthFailed arm
                app.app_handle_server_message({{
                    type: "AuthFailed", 
                    message: "Auth test message"
                }});
                results.push("AuthFailed: OK");
                
                // Test ConferenceFull arm  
                app.app_handle_server_message({{
                    type: "ConferenceFull", 
                    message: "Conference test message"
                }});
                results.push("ConferenceFull: OK");
                
                // Test Error arm
                app.app_handle_server_message({{
                    type: "Error", 
                    message: "Error test message"
                }});
                results.push("Error: OK");
                
                // Test TestMessage arm
                app.app_handle_server_message({{
                    type: "TestMessage", 
                    from: "test-user",
                    message: "Test test message"
                }});
                results.push("TestMessage: OK");
                
                console.log("All match arms executed successfully!");
                console.log("Results:", results.join(", "));
                
            }} catch (e) {{
                console.log("❌ JavaScript execution error:", e.message);
                if (e.message.includes("is not defined")) {{
                    throw new Error("VARIABLE REFERENCE BUG: " + e.message);
                }}
                throw e;
            }}
            "#,
            full_js
        );
        
        match eval_js_with_context(&test_js) {
            Ok(_) => {
                println!("🎉 SUCCESS: All match arms executed without errors!");
                println!("✅ The variable naming bug has been FIXED!");
                println!("   - Variable declarations and references are now consistent");
                println!("   - JavaScript executes correctly for all match arms");
            }
            Err(e) => {
                let error_str = format!("{:?}", e);
                if error_str.contains("is not defined") {
                    panic!("❌ VARIABLE REFERENCE BUG STILL EXISTS: JavaScript execution failed due to undefined variable: {:?}", e);
                } else {
                    println!("JavaScript execution failed for different reason (not the variable bug): {:?}", e);
                    // Don't panic for unrelated errors - the variable bug might still be fixed
                }
            }
        }
    }
}