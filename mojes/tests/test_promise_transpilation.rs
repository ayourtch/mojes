use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mock Promise type for testing (simulating the DOM API Promise)
#[derive(Debug, Clone)]
pub struct Promise<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Promise<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Promise<()> {
    pub fn resolve(_value: ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn reject_unit(_reason: String) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[js_type]
#[derive(Debug, Clone)]
pub struct TestPromiseManager {
    pub value: i32,
}

#[js_object]
impl TestPromiseManager {
    pub fn make() -> Self {
        Self { value: 42 }
    }
    
    // Test 1: Promise return WITHOUT explicit return keyword (may not transpile correctly)
    pub fn promise_without_return(&self, should_succeed: bool) -> Promise<()> {
        println!("Testing promise_without_return");
        
        if should_succeed {
            println!("Success path - no explicit return");
            Promise::resolve(())
        } else {
            println!("Error path - no explicit return");
            Promise::reject_unit("error".to_string())
        }
    }
    
    // Test 2: Promise return WITH explicit return keyword (should transpile correctly)
    pub fn promise_with_return(&self, should_succeed: bool) -> Promise<()> {
        println!("Testing promise_with_return");
        
        if should_succeed {
            println!("Success path - with explicit return");
            return Promise::resolve(());
        } else {
            println!("Error path - with explicit return");
            return Promise::reject_unit("error".to_string());
        }
    }
    
    // Test 3: Promise return in match expression WITHOUT return
    pub fn promise_match_without_return(&self, value: i32) -> Promise<()> {
        println!("Testing promise_match_without_return");
        
        match value {
            0 => {
                println!("Match 0 - no return");
                Promise::resolve(())
            },
            1 => {
                println!("Match 1 - no return");
                Promise::reject_unit("matched 1".to_string())
            },
            _ => {
                println!("Match default - no return");
                Promise::resolve(())
            }
        }
    }
    
    // Test 4: Promise return in match expression WITH return
    pub fn promise_match_with_return(&self, value: i32) -> Promise<()> {
        println!("Testing promise_match_with_return");
        
        match value {
            0 => {
                println!("Match 0 - with return");
                return Promise::resolve(());
            },
            1 => {
                println!("Match 1 - with return");
                return Promise::reject_unit("matched 1".to_string());
            },
            _ => {
                println!("Match default - with return");
                return Promise::resolve(());
            }
        }
    }
    
    // Test 5: Promise with early return pattern
    pub fn promise_early_return(&self, check: bool) -> Promise<()> {
        println!("Testing promise_early_return");
        
        if !check {
            println!("Early return path");
            return Promise::reject_unit("check failed".to_string());
        }
        
        println!("Normal flow after early return");
        // Some processing here...
        
        println!("Final return");
        return Promise::resolve(());
    }
    
    // Test 6: Promise without early return pattern (implicit return)
    pub fn promise_implicit_return(&self, check: bool) -> Promise<()> {
        println!("Testing promise_implicit_return");
        
        if check {
            println!("Check passed - implicit");
            Promise::resolve(())
        } else {
            println!("Check failed - implicit");
            Promise::reject_unit("check failed".to_string())
        }
    }
}

// Test function to demonstrate the patterns
#[to_js]
pub fn test_promise_patterns() {
    println!("{}","=== Testing Promise Return Transpilation Patterns ===");
    
    let manager = TestPromiseManager::make();
    
    println!("{}","\n--- Test 1: Without explicit return keyword ---");
    let promise1_success = manager.promise_without_return(true);
    println!("{}",&format!("Promise1 success: {:?}", promise1_success));
    let promise1_error = manager.promise_without_return(false);
    println!("{}",&format!("Promise1 error: {:?}", promise1_error));
    
    println!("{}","\n--- Test 2: With explicit return keyword ---");
    let promise2_success = manager.promise_with_return(true);
    println!("{}",&format!("Promise2 success: {:?}", promise2_success));
    let promise2_error = manager.promise_with_return(false);
    println!("{}",&format!("Promise2 error: {:?}", promise2_error));
    
    println!("{}","\n--- Test 3: Match without return ---");
    let promise3_0 = manager.promise_match_without_return(0);
    println!("{}",&format!("Promise3 (0): {:?}", promise3_0));
    let promise3_1 = manager.promise_match_without_return(1);
    println!("{}",&format!("Promise3 (1): {:?}", promise3_1));
    let promise3_default = manager.promise_match_without_return(99);
    println!("{}",&format!("Promise3 (default): {:?}", promise3_default));
    
    println!("{}","\n--- Test 4: Match with return ---");
    let promise4_0 = manager.promise_match_with_return(0);
    println!("{}",&format!("Promise4 (0): {:?}", promise4_0));
    let promise4_1 = manager.promise_match_with_return(1);
    println!("{}",&format!("Promise4 (1): {:?}", promise4_1));
    let promise4_default = manager.promise_match_with_return(99);
    println!("{}",&format!("Promise4 (default): {:?}", promise4_default));
    
    println!("{}","\n--- Test 5: Early return pattern ---");
    let promise5_pass = manager.promise_early_return(true);
    println!("{}",&format!("Promise5 pass: {:?}", promise5_pass));
    let promise5_fail = manager.promise_early_return(false);
    println!("{}",&format!("Promise5 fail: {:?}", promise5_fail));
    
    println!("{}","\n--- Test 6: Implicit return pattern ---");
    let promise6_pass = manager.promise_implicit_return(true);
    println!("{}",&format!("Promise6 pass: {:?}", promise6_pass));
    let promise6_fail = manager.promise_implicit_return(false);
    println!("{}",&format!("Promise6 fail: {:?}", promise6_fail));
    
    println!("{}","\n=== Expected Transpilation Behavior ===");
    println!("{}","1. WITHOUT 'return': May transpile to just 'Promise.resolve()' without return statement");
    println!("{}","2. WITH 'return': Should transpile to 'return Promise.resolve()'");
    println!("{}","3. This affects whether the JavaScript function actually returns the Promise");
    println!("{}","4. Without proper return, JavaScript may return 'undefined' instead of the Promise");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_promise_transpilation() {
        // This test just ensures the code compiles
        // The actual transpilation behavior needs to be verified in the generated JavaScript
        test_promise_patterns();
    }
}