use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[to_js]
pub fn test_conditional_in_format() {
    let status = true;
    let priority = false;
    
    // These conditional expressions in format! should return actual values, not undefined
    let message1 = format!("Status: {}", if status { "ACTIVE" } else { "INACTIVE" });
    let message2 = format!("Priority: {}", if priority { "HIGH" } else { "LOW" });
    let message3 = format!("Combined: {} - {}", 
        if status { "ON" } else { "OFF" },
        if priority { "URGENT" } else { "NORMAL" }
    );
    
    println!("{}", message1);
    println!("{}", message2);
    println!("{}", message3);
}

#[to_js]
pub fn test_nested_conditionals_in_format() {
    let score = 85;
    
    // Nested conditional should also work correctly
    let grade = format!("Grade: {}", 
        if score >= 90 { 
            "A" 
        } else if score >= 80 { 
            "B" 
        } else if score >= 70 { 
            "C" 
        } else { 
            "F" 
        }
    );
    
    println!("{}", grade);
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, JsResult, JsValue};

    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        
        // Add console.log support for testing
        let console_code = r#"
            globalThis._test_logs = [];
            globalThis.console = {
                log: function(...args) {
                    globalThis._test_logs.push(args.join(' '));
                }
            };
        "#;
        context.eval(boa_engine::Source::from_bytes(console_code))?;
        
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
    fn test_conditional_expressions_return_actual_values() {
        // This test demonstrates that conditional expressions in format! now return
        // their actual values instead of 'undefined' (which was the old broken behavior)
        
        // Get the generated JavaScript and execute it
        let mut js_code = String::new();
        for js_func in JS.iter() {
            js_code.push_str(js_func);
            js_code.push('\n');
        }
        
        let (_, mut context) = eval_js_with_context(&js_code).expect("Failed to execute JavaScript");
        
        // Call the test functions
        context.eval(boa_engine::Source::from_bytes("test_conditional_in_format();")).unwrap();
        context.eval(boa_engine::Source::from_bytes("test_nested_conditionals_in_format();")).unwrap();
        
        // Extract the console output
        let messages = extract_logged_messages(&mut context);
        
        println!("Console output from transpiled JavaScript:");
        for msg in &messages {
            println!("  {}", msg);
        }
        
        // Verify the expected outputs (these would FAIL with the old approach)
        assert_eq!(messages.len(), 4, "Expected 4 console messages");
        
        assert_eq!(messages[0], "Status: ACTIVE", "First message should show ACTIVE status");
        assert_eq!(messages[1], "Priority: LOW", "Second message should show LOW priority");  
        assert_eq!(messages[2], "Combined: ON - NORMAL", "Third message should show combined values");
        assert_eq!(messages[3], "Grade: B", "Fourth message should show grade B");
        
        // Verify NO messages contain 'undefined' 
        for (i, msg) in messages.iter().enumerate() {
            assert!(!msg.contains("undefined"), 
                   "Message {} should not contain 'undefined': {}", i + 1, msg);
        }
        
        println!("✅ SUCCESS: All conditional expressions returned their actual values!");
        println!("   (This test would FAIL with the old 'return undefined;' safety net approach)");
    }
}