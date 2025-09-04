use mojes_derive::{to_js};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[to_js]
fn simple_implicit_return() -> String {
    "test".to_string()
}

#[to_js] 
fn complex_implicit_return() -> String {
    let prefix = "Hello";
    format!("{} World", prefix)
}

#[to_js]
fn conditional_implicit_return(flag: bool) -> String {
    if flag {
        "true_value".to_string()
    } else {
        "false_value".to_string()
    }
}

#[to_js]
fn block_implicit_return() -> String {
    {
        let temp = "nested";
        format!("Result: {}", temp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, JsResult, JsValue};

    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        Ok((result, context))
    }

    #[test]
    fn test_function_implicit_returns() {
        println!("=== Testing function implicit returns ===");
        
        // Get the generated JavaScript
        let mut js_code = String::new();
        for js_func in JS.iter() {
            js_code.push_str(js_func);
            js_code.push('\n');
        }
        
        println!("Generated JavaScript:");
        println!("{}", js_code);
        
        let (_, mut context) = eval_js_with_context(&js_code).expect("Failed to execute JavaScript");
        
        // Test each function
        let result1 = context.eval(boa_engine::Source::from_bytes("simple_implicit_return()")).unwrap();
        let result2 = context.eval(boa_engine::Source::from_bytes("complex_implicit_return()")).unwrap();
        let result3 = context.eval(boa_engine::Source::from_bytes("conditional_implicit_return(true)")).unwrap();
        let result4 = context.eval(boa_engine::Source::from_bytes("conditional_implicit_return(false)")).unwrap();
        let result5 = context.eval(boa_engine::Source::from_bytes("block_implicit_return()")).unwrap();
        
        println!("Function call results:");
        println!("  simple_implicit_return(): {:?}", result1);
        println!("  complex_implicit_return(): {:?}", result2);
        println!("  conditional_implicit_return(true): {:?}", result3);
        println!("  conditional_implicit_return(false): {:?}", result4);
        println!("  block_implicit_return(): {:?}", result5);
        
        // Convert results to strings for assertions
        let str1 = result1.to_string(&mut context).unwrap().to_std_string().unwrap();
        let str2 = result2.to_string(&mut context).unwrap().to_std_string().unwrap();
        let str3 = result3.to_string(&mut context).unwrap().to_std_string().unwrap();
        let str4 = result4.to_string(&mut context).unwrap().to_std_string().unwrap();
        let str5 = result5.to_string(&mut context).unwrap().to_std_string().unwrap();
        
        println!("String representations:");
        println!("  simple: '{}'", str1);
        println!("  complex: '{}'", str2);
        println!("  conditional(true): '{}'", str3);
        println!("  conditional(false): '{}'", str4);
        println!("  block: '{}'", str5);
        
        // Check if implicit returns work correctly
        assert_eq!(str1, "test", "Simple implicit return should work");
        assert_eq!(str2, "Hello World", "Complex implicit return should work");
        assert_eq!(str3, "true_value", "Conditional implicit return (true) should work");
        assert_eq!(str4, "false_value", "Conditional implicit return (false) should work");
        assert_eq!(str5, "Result: nested", "Block implicit return should work");
        
        println!("✅ All function implicit returns work correctly!");
    }
}