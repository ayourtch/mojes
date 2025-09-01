use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;
use std::collections::HashMap;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
struct SimpleTest;

#[js_object]
impl SimpleTest {
    fn new() -> Self {
        Self
    }

    fn test_basic_hashmap(&self) {
        println!("=== Basic HashMap Test ===");
        let mut map = HashMap::new();
        
        // Test insert
        map.insert("key1".to_string(), "value1".to_string());
        println!("Inserted key1 -> value1");
        
        // Test get  
        if let Some(value) = map.get("key1") {
            println!("Retrieved: key1 -> {}", value);
        }
        
        // Test remove
        if let Some(removed) = map.remove("key1") {
            println!("Removed: key1 -> {}", removed);
        }
        
        println!("HashMap test completed successfully!");
    }

    fn test_basic_vec(&self) {
        println!("=== Basic Vec Test ===");
        let mut vec = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        
        // Test remove by index
        let removed = vec.remove(1);
        println!("Removed index 1: {}", removed);
        
        println!("Vec test completed successfully!");
    }
}

#[to_js]
pub fn test_simple_collections() -> bool {
    println!("=== Simple Collections Test ===");
    
    let test = SimpleTest::new();
    test.test_basic_hashmap();
    test.test_basic_vec();
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source, JsResult, JsValue};
    
    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        
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
    fn test_simple_collections_execution() {
        // Run Rust version first
        println!("=== Running Rust Version ===");
        let rust_result = test_simple_collections();
        assert!(rust_result);
        
        // Get generated JavaScript and reorder for proper execution
        println!("\n=== Generated JavaScript ===");
        let mut classes = String::new();
        let mut methods = String::new();
        let mut functions = String::new();
        
        for js_code in JS.iter() {
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
        println!("{}", full_js);
        
        // Test JavaScript execution
        println!("\n=== Testing JavaScript Execution ===");
        let test_js = format!(
            r#"
            {}
            
            console.log("Starting JavaScript test...");
            test_simple_collections();
            console.log("JavaScript test completed!");
            "#,
            full_js
        );
        
        match eval_js_with_context(&test_js) {
            Ok(_) => {
                println!("✅ JavaScript executed successfully!");
            }
            Err(e) => {
                println!("❌ JavaScript execution error: {:?}", e);
                println!("Generated JavaScript code:");
                println!("{}", test_js);
                panic!("JavaScript execution failed: {:?}", e);
            }
        }
    }
}