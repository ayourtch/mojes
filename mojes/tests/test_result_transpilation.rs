
use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
#[derive(Debug, Clone)]
pub struct TestResult {
    pub value: i32,
}

#[js_object]
impl TestResult {
    pub fn make() -> Self {
        Self { value: 42 }
    }
    
    // Test function that returns Result from different contexts
    pub fn test_method(&self, should_succeed: bool) -> Result<String, String> {
        if should_succeed {
            Ok("success".to_string())
        } else {
            Err("failure".to_string())
        }
    }
}

// Test various Result usage patterns
#[to_js]
pub fn test_result_patterns() {
    println!("{}","=== Testing Result Transpilation Patterns ===");
    
    let test_obj = TestResult::make();
    
    // Test 1: Early return with Ok
    println!("{}","Test 1: Early return Ok");
    let result1 = test_early_return_ok();
    println!("{}",&format!("Result1 type: {:?}", result1));
    
    // Test 2: Early return with Err  
    println!("{}","Test 2: Early return Err");
    let result2 = test_early_return_err();
    println!("{}",&format!("Result2 type: {:?}", result2));
    
    // Test 3: Match expression with Ok/Err
    println!("{}","Test 3: Match expression");
    let result3 = test_match_expression(true);
    println!("{}",&format!("Result3 type: {:?}", result3));
    
    let result4 = test_match_expression(false);
    println!("{}",&format!("Result4 type: {:?}", result4));
    
    // Test 4: Method call returning Result
    println!("{}","Test 4: Method call Result");
    let result5 = test_obj.test_method(true);
    println!("{}",&format!("Result5 type: {:?}", result5));
    
    let result6 = test_obj.test_method(false);
    println!("{}",&format!("Result6 type: {:?}", result6));
    
    // Test 5: Nested match on Result from method call
    println!("{}","Test 5: Nested match on method Result");
    match test_obj.test_method(true) {
        Ok(value) => {
            println!("{}",&format!("✓ Got Ok: {}", value));
        }
        Err(error) => {
            println!("{}",&format!("✗ Got Err: {}", error));
        }
    }
    
    match test_obj.test_method(false) {
        Ok(value) => {
            println!("{}",&format!("✓ Got Ok: {}", value));
        }
        Err(error) => {
            println!("{}",&format!("✗ Got Err: {}", error));
        }
    }
    
    // Test 6: Result propagation with ?
    println!("{}","Test 6: Result propagation");
    let result7 = test_result_propagation(true);
    println!("{}",&format!("Result7 type: {:?}", result7));
    
    let result8 = test_result_propagation(false);
    println!("{}",&format!("Result8 type: {:?}", result8));
}

#[to_js]
fn test_early_return_ok() -> Result<i32, String> {
    println!("{}","  Inside test_early_return_ok");
    
    if true {
        return Ok(123);
    }
    
    Err("should not reach".to_string())
}

#[to_js]
fn test_early_return_err() -> Result<i32, String> {
    println!("{}","  Inside test_early_return_err");
    
    if true {
        return Err("early error".to_string());
    }
    
    Ok(456)
}

#[to_js] 
fn test_match_expression(should_succeed: bool) -> Result<String, String> {
    println!("{}","  Inside test_match_expression");
    
    let inner_result = if should_succeed {
        Ok("inner_ok".to_string())
    } else {
        Err("inner_err".to_string())
    };
    
    match inner_result {
        Ok(value) => {
            println!("{}",&format!("  Match arm Ok: {}", value));
            Ok(format!("wrapped_{}", value))
        }
        Err(error) => {
            println!("{}",&format!("  Match arm Err: {}", error));
            Err(format!("wrapped_{}", error))
        }
    }
}

#[to_js]
fn test_result_propagation(should_succeed: bool) -> Result<String, String> {
    println!("{}","  Inside test_result_propagation");
    
    let test_obj = TestResult::make();
    
    // Use ? operator to propagate errors
    let value = test_obj.test_method(should_succeed)?;
    
    Ok(format!("propagated: {}", value))
}

// Test JavaScript-side Result handling
#[to_js]
pub fn test_javascript_result_handling() {
    println!("{}","=== Testing JavaScript Result Handling ===");
    
    // Test how JavaScript receives and processes Results
    let result = test_early_return_ok();
    
    // This is what should work consistently:
    println!("{}","Testing consistent Result format access:");
    
    // Try both possible formats to see which one works
    println!("{}",&format!("result.type: {:?}", "checking type field"));
    println!("{}",&format!("result.ok: {:?}", "checking ok field"));  
    println!("{}",&format!("result.error: {:?}", "checking error field"));
    
    // Test the actual access pattern that should work
    let test_obj = TestResult::make();
    let method_result = test_obj.test_method(true);
    
    println!("{}","Method result format test:");
    println!("{}",&format!("method_result type: {:?}", method_result));
}

/*
Expected Behavior Documentation:

All Result<T, E> values should transpile to consistent JavaScript objects:

Success case: { type: "Ok", value0: T }
Error case:   { type: "Err", value0: E }

This should work for:
1. Early returns: return Ok(value) / return Err(error)
2. Match expressions: Ok(value) => ... / Err(error) => ...
3. Method call results: obj.method_returning_result()
4. Result propagation: method()?

Current Issues Observed:
- Early returns transpile to { error: "message" } instead of { type: "Err", value0: "message" }
- Match expressions transpile to { ok: value } / { error: message } instead of consistent format
- Method calls correctly return { type: "Ok"/"Err", value0: value }

This creates inconsistency where JavaScript code expects .type === "Ok" but gets different object shapes.
*/

#[test]
fn test_generated_javascript() {
    // Test that we can access the generated JavaScript code
    println!("Generated JavaScript functions:");
    for js_code in JS.iter() {
        println!("JS: {}", js_code);
    }
    
    // Basic sanity check - should have some JavaScript generated
    assert!(!JS.is_empty(), "No JavaScript code was generated");
}
