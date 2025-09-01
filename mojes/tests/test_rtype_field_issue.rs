use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub r#type: String,
    pub data: String,
}

#[js_object]
impl TestEvent {
    pub fn new(event_type: String, data: String) -> Self {
        Self {
            r#type: event_type,
            data,
        }
    }

    pub fn get_type(&self) -> String {
        self.r#type.clone()
    }

    pub fn is_message_event(&self) -> bool {
        self.r#type == "message"
    }
}

// Main test function
#[to_js]
pub fn test_rtype_field_functionality() -> bool {
    println!("=== Testing r#type Field Functionality ===");
    
    let event = TestEvent::new("click".to_string(), "button clicked".to_string());
    
    // This should work - accessing via method
    let event_type = event.get_type();
    println!("Method access: type='{}'", event_type);
    
    // This is the problematic one - direct field access
    let direct_type = event.r#type;
    println!("Direct access: type='{}'", direct_type);
    
    event_type == "click" && direct_type == "click"
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
    fn test_rtype_field_transpilation_issue() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_rtype_field_functionality();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut class_code = String::new();
        let mut method_code = String::new();  
        let mut function_code = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class TestEvent") {
                class_code.push_str(js_code);
                class_code.push('\n');
            } else if js_code.contains("TestEvent.") {
                method_code.push_str(js_code);
                method_code.push('\n');
            } else if js_code.contains("function test_rtype_field_functionality") {
                function_code.push_str(js_code);
                function_code.push('\n');
            }
        }
        
        // Test 1: Check if the generated JavaScript is syntactically valid
        println!("\n=== Testing JavaScript Syntax ===");
        let full_js = format!("{}\n{}\n{}\n\n// Call the test function\ntest_rtype_field_functionality();", class_code, method_code, function_code);
        
        match eval_js_with_context(&full_js) {
            Ok((result, mut ctx)) => {
                println!("✅ JavaScript syntax is valid");
                let js_result = js_to_boolean(&result, &mut ctx);
                println!("JavaScript test result: {}", js_result);
                println!("Raw JavaScript result: {:?}", result);
                
                if js_result {
                    println!("🎉 r#type field works correctly in JavaScript!");
                } else {
                    println!("❌ r#type field test failed in JavaScript execution");
                    println!("Debug: Let's investigate what went wrong...");
                    
                    // Let's run a debug version to see the actual values
                    let debug_test = format!(
                        r#"
                        {}
                        {}
                        
                        const event = TestEvent.new("click", "button clicked");
                        console.log("event:", event);
                        console.log("event.type:", event.type);
                        const direct_type = event.type;
                        console.log("direct_type:", direct_type);
                        direct_type;
                        "#,
                        class_code, method_code
                    );
                    
                    match eval_js_with_context(&debug_test) {
                        Ok((result, mut ctx)) => {
                            let debug_value = js_to_string(&result, &mut ctx);
                            println!("Debug result: '{}'", debug_value);
                        }
                        Err(e) => {
                            println!("Debug error: {:?}", e);
                        }
                    }
                    
                    panic!("r#type field test failed in JavaScript execution");
                }
            }
            Err(e) => {
                println!("❌ JavaScript execution error: {:?}", e);
                println!("Generated JavaScript:\n{}", full_js);
                panic!("r#type field caused JavaScript execution error: {:?}", e);
            }
        }
        
        // Test 2: Test direct field access specifically
        println!("\n=== Testing Direct r#type Field Access ===");
        let direct_access_test = format!(
            r#"
            {}
            {}
            
            const event = TestEvent.new("test", "data");
            event.type;  // This should work if r#type transpiles to 'type'
            "#,
            class_code, method_code
        );
        
        match eval_js_with_context(&direct_access_test) {
            Ok((result, mut ctx)) => {
                let field_value = js_to_string(&result, &mut ctx);
                println!("✅ Direct field access works: event.type = '{}'", field_value);
                
                if field_value == "test" {
                    println!("🎉 r#type field transpiles correctly to 'type'!");
                } else {
                    println!("❌ r#type field access returned wrong value: expected 'test', got '{}'", field_value);
                    panic!("r#type field access returned wrong value: expected 'test', got '{}'", field_value);
                }
            }
            Err(e) => {
                println!("❌ Direct r#type field access error: {:?}", e);
                println!("Test JavaScript:\n{}", direct_access_test);
                panic!("Direct r#type field access failed: {:?}", e);
            }
        }
    }
}