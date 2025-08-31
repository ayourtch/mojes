use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mock Promise type
#[derive(Debug, Clone)]
pub struct Promise<T> {
    _phantom: std::marker::PhantomData<T>,
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
pub struct PromiseReturnTest {
    pub id: String,
}

#[js_object]
impl PromiseReturnTest {
    pub fn make(id: String) -> Self {
        Self { id }
    }
    
    // PROBLEMATIC PATTERN: Implicit return in match
    // This might transpile to JavaScript without proper return statement
    pub fn create_connection_bad(&self, success: bool) -> Promise<()> {
        println!("{}",&format!("Creating connection (bad pattern) for {}", self.id));
        
        match success {
            true => {
                println!("Success case - no explicit return");
                Promise::resolve(())  // <-- Missing 'return' keyword
            },
            false => {
                println!("Error case - no explicit return");
                Promise::reject_unit("Connection failed".to_string())  // <-- Missing 'return'
            }
        }
    }
    
    // CORRECT PATTERN: Explicit return in match
    // This should transpile correctly to JavaScript with return statements
    pub fn create_connection_good(&self, success: bool) -> Promise<()> {
        println!("{}",&format!("Creating connection (good pattern) for {}", self.id));
        
        match success {
            true => {
                println!("Success case - with explicit return");
                return Promise::resolve(());  // <-- Explicit 'return'
            },
            false => {
                println!("Error case - with explicit return");
                return Promise::reject_unit("Connection failed".to_string());  // <-- Explicit 'return'
            }
        }
    }
    
    // ALSO PROBLEMATIC: Implicit return at end of function
    pub fn async_operation_bad(&self) -> Promise<()> {
        println!("Starting async operation (bad)");
        // ... some logic ...
        Promise::resolve(())  // <-- No 'return' keyword
    }
    
    // CORRECT: Explicit return at end of function
    pub fn async_operation_good(&self) -> Promise<()> {
        println!("Starting async operation (good)");
        // ... some logic ...
        return Promise::resolve(());  // <-- Explicit 'return'
    }
}

#[to_js]
pub fn demonstrate_promise_return_issue() {
    println!("{}","\n=== CRITICAL ISSUE: Promise Return in JavaScript ===");
    
    let test = PromiseReturnTest::make("test-1".to_string());
    
    println!("{}","\n--- Testing BAD pattern (no explicit return) ---");
    println!("{}","In JavaScript, this will likely return 'undefined' instead of a Promise:");
    let bad_result = test.create_connection_bad(true);
    println!("{}",&format!("Result: {:?}", bad_result));
    
    println!("{}","\n--- Testing GOOD pattern (with explicit return) ---");
    println!("{}","In JavaScript, this will correctly return a Promise:");
    let good_result = test.create_connection_good(true);
    println!("{}",&format!("Result: {:?}", good_result));
    
    println!("{}","\n=== Expected JavaScript Transpilation ===");
    
    println!("{}","\nBAD (without return):");
    println!("{}","create_connection_bad(success) {");
    println!("{}","  if (success) {");
    println!("{}","    console.log('Success case');");
    println!("{}","    Promise.resolve();  // <-- No return! Function returns undefined");
    println!("{}","  } else {");
    println!("{}","    console.log('Error case');");
    println!("{}","    Promise.reject('Connection failed');  // <-- No return!");
    println!("{}","  }");
    println!("{}","}");
    
    println!("{}","\nGOOD (with return):");
    println!("{}","create_connection_good(success) {");
    println!("{}","  if (success) {");
    println!("{}","    console.log('Success case');");
    println!("{}","    return Promise.resolve();  // <-- Correct! Returns the Promise");
    println!("{}","  } else {");
    println!("{}","    console.log('Error case');");
    println!("{}","    return Promise.reject('Connection failed');  // <-- Correct!");
    println!("{}","  }");
    println!("{}","}");
    
    println!("{}","\n=== Why This Matters ===");
    println!("{}","1. When calling promise_result.then(), if promise_result is undefined:");
    println!("{}","   - JavaScript throws: 'Cannot read property then of undefined'");
    println!("{}","2. The Rust compiler doesn't catch this because the types are correct in Rust");
    println!("{}","3. The issue only appears at JavaScript runtime");
    println!("{}","4. Solution: ALWAYS use explicit 'return' with Promise-returning functions");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_promise_return_patterns() {
        demonstrate_promise_return_issue();
    }
    
    #[test]
    fn test_show_actual_transpiled_javascript() {
        println!("=== ACTUAL TRANSPILED JAVASCRIPT FROM MOJES ===\n");
        
        println!("Generated JavaScript functions:");
        for (i, js_code) in JS.iter().enumerate() {
            println!("--- Function {} ---", i + 1);
            println!("{}", js_code);
            println!();
        }
        
        println!("=== ANALYSIS ===");
        println!("Look at the above JavaScript to see:");
        println!("1. How match expressions with implicit returns are transpiled");
        println!("2. Whether 'return' statements are properly generated");  
        println!("3. How Promise constructors are handled");
        println!("4. If there are any missing return statements that would cause undefined");
    }
}