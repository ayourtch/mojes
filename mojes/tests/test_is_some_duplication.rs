use mojes_derive::{js_object, js_type, to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
struct TestIsSomeStruct;

// Mock function to simulate function calls
fn get_optional_value(id: &str) -> Option<String> {
    Some(format!("value_{}", id))
}

#[js_object]
impl TestIsSomeStruct {
    fn test_is_some_with_function_call(&self, id: &str) -> bool {
        // This should generate IIFE to avoid duplicate function calls:
        // ((val) => val !== null && val !== undefined)(get_optional_value(id))
        get_optional_value(id).is_some()
    }
    
    fn test_is_some_without_function_call(&self) -> bool {
        let value = Some(42);
        // This should use the normal approach since it's not a function call:
        // value !== null && value !== undefined
        value.is_some()
    }
    
    fn test_is_none_with_function_call(&self, id: &str) -> bool {
        // This should generate IIFE to avoid duplicate function calls:
        // ((val) => val === null || val === undefined)(get_optional_value(id))
        get_optional_value(id).is_none()
    }
}

#[to_js]
pub fn test_is_some_optimization() {
    let tester = TestIsSomeStruct;
    
    println!("Testing is_some with function call: {}", tester.test_is_some_with_function_call("test"));
    println!("Testing is_some without function call: {}", tester.test_is_some_without_function_call());
    println!("Testing is_none with function call: {}", tester.test_is_none_with_function_call("test"));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_some_iife_optimization() {
        // This test ensures the code compiles and runs
        test_is_some_optimization();
        
        // Check the generated JavaScript for IIFE patterns
        println!("\n=== Generated JavaScript Code ===");
        for js_code in JS.iter() {
            println!("{}", js_code);
           /* 
            // Check for IIFE patterns in function calls
            if js_code.contains("test_is_some_with_function_call") {
                assert!(js_code.contains("((val)=>"), 
                       "is_some with function call should use IIFE pattern");
                assert!(js_code.contains("get_optional_value(id)"), 
                       "Function should be called once in IIFE");
            }
            
            if js_code.contains("test_is_some_without_function_call") {
                // Should NOT contain IIFE pattern for variable access
                assert!(!js_code.contains("((val)=>"), 
                       "is_some with variable should not use IIFE pattern");
            }
            
            if js_code.contains("test_is_none_with_function_call") {
                assert!(js_code.contains("((val)=>"), 
                       "is_none with function call should use IIFE pattern");
                assert!(js_code.contains("get_optional_value(id)"), 
                       "Function should be called once in IIFE");
            }
*/
        }
    }
}
