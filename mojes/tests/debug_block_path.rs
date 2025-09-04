use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// This should work (simple function body)
#[to_js]
fn simple_function_body() -> String {
    format!("simple")
}

// This should work (direct return from block)  
#[to_js]
fn direct_block() -> String {
    let result = {
        let temp = "nested";  
        format!("Result: {}", temp)
    };
    result
}

// This is the problematic case
#[to_js]
fn whole_block_as_body() -> String {
    {
        let temp = "nested";
        format!("Result: {}", temp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn debug_block_paths() {
        println!("=== Generated JavaScript ===");
        for js_code in JS.iter() {
            println!("JS: {}", js_code);
            println!("---");
        }
    }
}