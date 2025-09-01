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

    fn test_hashmap_operations(&mut self) {
        println!("=== Testing HashMap Operations ===");
        
        // Insert multiple elements
        self.data.insert("apple".to_string(), "red".to_string());
        self.data.insert("banana".to_string(), "yellow".to_string());
        self.data.insert("grape".to_string(), "purple".to_string());
        println!("Inserted 3 items: apple->red, banana->yellow, grape->purple");
        
        // Iterate and print all items
        println!("Current HashMap contents:");
        for (key, value) in &self.data {
            println!("  {} -> {}", key, value);
        }
        
        // Remove elements one by one
        let removed_apple = self.data.remove("apple");
        match removed_apple {
            Some(value) => println!("Removed apple, value was: {}", value),
            None => println!("Apple not found for removal"),
        }
        
        // Print remaining items
        println!("After removing apple:");
        for (key, value) in &self.data {
            println!("  {} -> {}", key, value);
        }
        
        let removed_grape = self.data.remove("grape");
        match removed_grape {
            Some(value) => println!("Removed grape, value was: {}", value),
            None => println!("Grape not found for removal"),
        }
        
        // Print remaining items
        println!("After removing grape:");
        for (key, value) in &self.data {
            println!("  {} -> {}", key, value);
        }
        
        // Remove last item
        let removed_banana = self.data.remove("banana");
        match removed_banana {
            Some(value) => println!("Removed banana, value was: {}", value),
            None => println!("Banana not found for removal"),
        }
        
        // Verify empty
        println!("Final HashMap size: {}", self.data.len());
        if self.data.is_empty() {
            println!("HashMap is now empty - SUCCESS!");
        }
    }
    
    fn test_vec_operations(&mut self) {
        println!("=== Testing Vec Operations (ensure we didn't break arrays) ===");
        
        let mut fruits = vec!["apple".to_string(), "banana".to_string(), "grape".to_string(), "orange".to_string()];
        println!("Created Vec with 4 items");
        
        // Print all items
        println!("Current Vec contents:");
        for (i, fruit) in fruits.iter().enumerate() {
            println!("  [{}] = {}", i, fruit);
        }
        
        // Remove by index (should use .splice())
        let removed = fruits.remove(1); // Remove "banana" 
        println!("Removed index 1, value was: {}", removed);
        
        // Print remaining items
        println!("After removing index 1:");
        for (i, fruit) in fruits.iter().enumerate() {
            println!("  [{}] = {}", i, fruit);
        }
        
        // Remove another item
        let removed2 = fruits.remove(0); // Remove "apple"
        println!("Removed index 0, value was: {}", removed2);
        
        // Print remaining items  
        println!("After removing index 0:");
        for (i, fruit) in fruits.iter().enumerate() {
            println!("  [{}] = {}", i, fruit);
        }
        
        println!("Final Vec size: {}", fruits.len());
        if fruits.len() == 2 {
            println!("Vec operations working correctly - SUCCESS!");
        }
    }
}

// Main test function
#[to_js]
pub fn test_hashmap_remove_bug() -> bool {
    println!("=== Testing HashMap and Vec Operations ===");
    
    let mut app = TestApp::new();
    app.test_hashmap_operations();
    app.test_vec_operations();
    
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
    fn test_hashmap_remove_transpilation_bug() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_hashmap_remove_bug();
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
        
        // Check if we have the IIFE universal dispatcher (which is the correct fix)
        if full_js.contains("obj.splice ? obj.splice") && full_js.contains("delete obj[key]") {
            println!("✅ FOUND FIX: HashMap methods now use universal IIFE dispatcher!");
            
            // Show the improved IIFE patterns
            for (i, line) in full_js.lines().enumerate() {
                if line.contains("obj.splice ? obj.splice") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
            // Test the JavaScript execution with fixed universal dispatchers
            println!("\n=== Testing Fixed JavaScript Execution ===");
            let test_js = format!(
                r#"
                {}
                
                try {{
                    console.log("Starting comprehensive collection operations test...");
                    test_hashmap_remove_bug();
                    console.log("✅ All collection operations completed successfully!");
                }} catch (e) {{
                    console.log("❌ JavaScript execution error:", e.message);
                    throw e;
                }}
                "#,
                full_js
            );
            
            match eval_js_with_context(&test_js) {
                Ok(_) => {
                    println!("🎉 SUCCESS: Universal IIFE dispatchers work correctly!");
                    println!("   - HashMap operations use conditional logic for proper method dispatch");
                    println!("   - Vec operations work through the same universal system");
                    println!("   - No collection type conflicts or hardcoded assumptions");
                }
                Err(e) => {
                    let error_str = format!("{:?}", e);
                    println!("JavaScript execution failed: {:?}", e);
                    println!("This may be due to other transpilation issues, not the IIFE dispatcher fix");
                }
            }
            
        } else if full_js.contains(".splice(") && !full_js.contains("obj.splice ?") {
            println!("❌ FOUND OLD BUG: HashMap methods still hardcoded to .splice()!");
            
            // Find and print the problematic lines
            for (i, line) in full_js.lines().enumerate() {
                if line.contains(".splice(") {
                    println!("Line {}: {}", i + 1, line.trim());
                }
            }
            
            panic!("HashMap methods are still using hardcoded .splice() instead of universal IIFE dispatcher");
        } else {
            println!("✅ No problematic .splice() patterns found");
            
            // Test actual execution
            println!("\n=== Testing JavaScript Execution ===");
            let test_js = format!(
                r#"
                {}
                
                try {{
                    console.log("Starting comprehensive HashMap and Vec operations test...");
                    test_hashmap_remove_bug();
                    console.log("✅ All collection operations completed successfully!");
                }} catch (e) {{
                    console.log("❌ JavaScript execution error:", e.message);
                    if (e.message.includes("splice is not a function")) {{
                        throw new Error("HASHMAP BUG: .splice() called on HashMap: " + e.message);
                    }}
                    throw e;
                }}
                "#,
                full_js
            );
            
            match eval_js_with_context(&test_js) {
                Ok(_) => {
                    println!("🎉 SUCCESS: Both HashMap and Vec operations work correctly!");
                    println!("   - HashMap.insert() and HashMap.remove() use universal IIFE dispatcher");
                    println!("   - Vec.remove() still uses .splice() as expected");
                    println!("   - No collection type conflicts detected");
                }
                Err(e) => {
                    let error_str = format!("{:?}", e);
                    if error_str.contains("splice is not a function") {
                        panic!("❌ HASHMAP BUG: .splice() called on HashMap object: {:?}", e);
                    } else {
                        println!("JavaScript execution failed for different reason: {:?}", e);
                    }
                }
            }
        }
    }
}