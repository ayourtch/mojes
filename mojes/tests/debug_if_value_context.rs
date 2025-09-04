use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[to_js]
pub fn simple_if_test() {
    // This should generate an if expression that returns values
    let result = if true { "YES" } else { "NO" };
    println!("Result: {}", result);
}

#[to_js]
pub fn format_if_test() {
    // This tests if expressions in format! arguments
    let condition = true;
    let message = format!("Status: {}", if condition { "GOOD" } else { "BAD" });
    // Also test format! directly with a different if expression
    let status = format!("Result: {}", if false { "FAIL" } else { "PASS" });
    println!("Message: {}", message);
    println!("Status: {}", status);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_if_debug() {
        println!("=== Running simple if test ===");
        simple_if_test();
        
        println!("\n=== Running format if test ===");
        format_if_test();
        
        println!("\n=== Generated JavaScript ===");
        for js_code in JS.iter() {
            println!("JS: {}", js_code);
        }
    }
}