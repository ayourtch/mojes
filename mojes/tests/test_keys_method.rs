use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;
use std::collections::HashMap;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
struct TestApp {
    data: HashMap<String, String>,
}

#[js_object]
impl TestApp {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn test_keys_method(&mut self) {
        // Insert some data
        self.data.insert("key1".to_string(), "value1".to_string());
        self.data.insert("key2".to_string(), "value2".to_string());
        
        // Try to get keys - this might fail in current transpilation
        let keys = self.data.keys();
        println!("Keys: {:?}", keys);
    }
}

#[to_js]
pub fn test_keys_transpilation() -> bool {
    let mut app = TestApp::new();
    app.test_keys_method();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source, JsResult, JsValue};
    
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
    fn test_keys_method_transpilation() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_keys_transpilation();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut classes = String::new();
        let mut methods = String::new(); 
        let mut functions = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class ") {
                classes.push_str(js_code);
                classes.push('\n');
            } else if js_code.contains(".prototype.") {
                methods.push_str(js_code);
                methods.push('\n');
            } else {
                functions.push_str(js_code);
                functions.push('\n');
            }
        }
        
        let full_js = format!("{}\n{}\n{}", classes, methods, functions);
        
        // Look for how .keys() was transpiled
        if full_js.contains(".keys()") {
            println!("❌ Found raw .keys() call - this will fail in JavaScript!");
            
            // Find and print the problematic lines
            for (i, line) in full_js.lines().enumerate() {
                if line.contains(".keys()") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
            panic!("FOUND BUG: .keys() method not transpiled correctly");
            
        } else if full_js.contains("Object.keys(") {
            println!("✅ GOOD: Found Object.keys() usage");
            
            // Show the Object.keys patterns
            for (i, line) in full_js.lines().enumerate() {
                if line.contains("Object.keys(") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
        } else {
            println!("⚠️  No .keys() method found in transpiled code");
            println!("Generated JavaScript:");
            println!("{}", full_js);
        }
    }
}