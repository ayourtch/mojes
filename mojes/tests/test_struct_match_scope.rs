use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Test struct with multiple fields
#[js_type]
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub event_type: String,
    pub data: String,
    pub timestamp: u64,
}

#[js_object]
impl TestEvent {
    pub fn new(event_type: String, data: String, timestamp: u64) -> Self {
        Self {
            event_type,
            data,
            timestamp,
        }
    }
}

// Another test struct
#[js_type]
#[derive(Debug, Clone)]
pub struct UserEvent {
    pub user_id: String,
    pub event_type: String,
    pub payload: String,
}

#[js_object]
impl UserEvent {
    pub fn new(user_id: String, event_type: String, payload: String) -> Self {
        Self {
            user_id,
            event_type,
            payload,
        }
    }
}

// Enum for testing
#[js_type]
#[derive(Debug, Clone)]
pub enum EventSource {
    Api { endpoint: String, data: String },
    WebSocket { channel: String, message: String },
    Internal { channel: String, event_type: String },
}

#[js_object]
impl EventSource {
    pub fn new_api(endpoint: String, data: String) -> Self {
        Self::Api { endpoint, data }
    }

    pub fn new_websocket(channel: String, message: String) -> Self {
        Self::WebSocket { channel, message }
    }

    pub fn new_internal(channel: String, event_type: String) -> Self {
        Self::Internal { channel, event_type }
    }
}

#[js_type]
#[derive(Debug, Clone)]
pub struct ScopeTestManager {
    pub results: Vec<String>,
}

#[js_object]
impl ScopeTestManager {
    pub fn new() -> Self {
        Self {
            results: vec![],
        }
    }

    // Test case 1: Multiple variables with same names in different match arms
    pub fn test_struct_destructuring_scope(&mut self) -> bool {
        let event1 = TestEvent::new("click".to_string(), "button1".to_string(), 100);
        let event2 = TestEvent::new("hover".to_string(), "button2".to_string(), 200);
        
        let mut all_passed = true;

        // Test same variable names in different arms - potential conflicts in JavaScript
        match event1.event_type.as_str() {
            "click" => {
                let event_type = &event2.event_type;
                println!("Case 1: Click event, other is: {}", event_type);
                if event_type != "hover" {
                    all_passed = false;
                }
            },
            "hover" => {
                let event_type = &event1.event_type; // Same variable name - potential conflict!
                println!("Case 2: Hover event: {}", event_type);
            },
            _ => {
                let event_type = "unknown"; // Another same variable name
                println!("Case 3: Unknown event: {}", event_type);
            }
        }

        all_passed
    }

    // Test case 2: Enum destructuring with overlapping field names
    pub fn test_enum_destructuring_scope(&mut self) -> bool {
        let sources = vec![
            EventSource::new_api("/users".to_string(), "user_data".to_string()),
            EventSource::new_websocket("chat".to_string(), "hello".to_string()),
            EventSource::new_internal("auth".to_string(), "login".to_string()),
        ];

        let mut all_passed = true;

        for source in sources {
            match source {
                EventSource::Api { endpoint, data } => {
                    println!("API: endpoint={}, data={}", endpoint, data);
                    if endpoint.is_empty() || data.is_empty() {
                        all_passed = false;
                    }
                },
                EventSource::WebSocket { channel, message } => {
                    println!("WebSocket: channel={}, message={}", channel, message);
                    // 'data' from previous case shouldn't conflict with 'message' here
                    if channel.is_empty() || message.is_empty() {
                        all_passed = false;
                    }
                },
                EventSource::Internal { channel, event_type } => {
                    // 'channel' here could conflict with other uses
                    println!("Internal: channel={}, event_type={}", channel, event_type);
                    if channel.is_empty() || event_type.is_empty() {
                        all_passed = false;
                    }
                }
            }
        }

        all_passed
    }

    // Test case 3: Nested struct matching with same variable names  
    pub fn test_nested_struct_matching(&mut self) -> bool {
        let user_event = UserEvent::new("user123".to_string(), "login".to_string(), "success".to_string());
        let test_event = TestEvent::new("click".to_string(), "clicked".to_string(), 300);

        let mut all_passed = true;

        // Test nested matches with potentially conflicting variable names
        match user_event.event_type.as_str() {
            "login" => {
                let event_type = &test_event.event_type;
                println!("Login with test event: {}", event_type);
                if event_type != "click" {
                    all_passed = false;
                }
                
                // Nested match with same variable name
                match test_event.event_type.as_str() {
                    "click" => {
                        let event_type = "nested_click"; // Same name as outer scope!
                        println!("Nested: {}", event_type);
                    },
                    _ => {
                        let event_type = "nested_other"; // Another conflict!
                        println!("Nested other: {}", event_type);
                    }
                }
            },
            _ => {
                let event_type = "outer_default";
                println!("Outer default: {}", event_type);
            }
        }

        all_passed
    }

    // Test case 4: Same variable names across different match arms
    pub fn test_variable_reuse_across_arms(&mut self) -> bool {
        let events = vec![
            TestEvent::new("click".to_string(), "data1".to_string(), 100),
            TestEvent::new("hover".to_string(), "data2".to_string(), 200),
            TestEvent::new("focus".to_string(), "data3".to_string(), 300),
        ];

        let mut all_passed = true;

        for event in events {
            match event.event_type.as_str() {
                "click" => {
                    let data = &event.data;
                    println!("Click data: {}", data);
                    if data != "data1" && data != "data2" && data != "data3" {
                        all_passed = false;
                    }
                },
                "hover" => {
                    // Same variable name 'data' - could conflict in JavaScript!
                    let data = format!("processed_{}", event.data);
                    println!("Hover data: {}", data);
                },
                _ => {
                    // Another 'data' variable
                    let data = event.data.to_uppercase();
                    println!("Other data: {}", data);
                }
            }
        }

        all_passed
    }
}

// Main test function
#[to_js]
pub fn test_struct_match_scope_functionality() -> bool {
    println!("=== Testing Struct Match Variable Scope ===");
    
    let mut manager = ScopeTestManager { results: vec![] };
    let mut all_passed = true;
    
    println!("\n🧪 Test 1: Struct destructuring scope");
    if !manager.test_struct_destructuring_scope() {
        all_passed = false;
    }
    
    println!("\n🧪 Test 2: Enum destructuring scope");
    if !manager.test_enum_destructuring_scope() {
        all_passed = false;
    }
    
    println!("\n🧪 Test 3: Nested struct matching");
    if !manager.test_nested_struct_matching() {
        all_passed = false;
    }
    
    println!("\n🧪 Test 4: Variable reuse across arms");
    if !manager.test_variable_reuse_across_arms() {
        all_passed = false;
    }
    
    println!("\n=== Test Results ===");
    if all_passed {
        println!("🎉 All struct match scope tests PASSED!");
    } else {
        println!("💥 Some struct match scope tests FAILED!");
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
    fn test_struct_match_scope_transpilation() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_struct_match_scope_functionality();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut class_code = String::new();
        let mut method_code = String::new();  
        let mut function_code = String::new();
        let mut enum_code = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class ") {
                class_code.push_str(js_code);
                class_code.push('\n');
            } else if js_code.starts_with("const EventSource") {
                enum_code.push_str(js_code);
                enum_code.push('\n');
            } else if js_code.contains(".prototype.") || js_code.contains(" = function(") {
                method_code.push_str(js_code);
                method_code.push('\n');
            } else if js_code.contains("function test_struct_match_scope_functionality") {
                function_code.push_str(js_code);
                function_code.push('\n');
            }
        }
        
        // Test: Check if the generated JavaScript is syntactically valid and executes
        println!("\n=== Testing JavaScript Execution ===");
        let full_js = format!("{}\n{}\n{}\n{}\n\n// Call the test function\ntest_struct_match_scope_functionality();", class_code, enum_code, method_code, function_code);
        
        println!("\n=== FULL GENERATED JAVASCRIPT ===");
        println!("{}", full_js);
        println!("=== END OF GENERATED JAVASCRIPT ===\n");
        
        match eval_js_with_context(&full_js) {
            Ok((result, mut ctx)) => {
                println!("✅ JavaScript syntax is valid");
                let js_result = js_to_boolean(&result, &mut ctx);
                println!("JavaScript test result: {}", js_result);
                
                if js_result {
                    println!("🎉 Struct match scope works correctly in JavaScript!");
                } else {
                    println!("❌ Struct match scope test failed in JavaScript execution");
                    panic!("Struct match scope test failed in JavaScript execution");
                }
            }
            Err(e) => {
                println!("❌ JavaScript execution error: {:?}", e);
                println!("Generated JavaScript:\n{}", full_js);
                panic!("Struct match scope caused JavaScript execution error: {:?}", e);
            }
        }
    }
}
