use mojes_mojo::*;
use syn::{Expr, parse_quote};

#[test]
fn test_closure_wildcard_parameters() {
    println!("Testing closure with wildcard parameters...");
    
    // Test single wildcard parameter
    let expr1: Expr = parse_quote! {
        some_method(|_| {
            println!("Ignoring parameter");
            42
        })
    };
    
    let js1 = rust_expr_to_js(&expr1);
    println!("Single wildcard: {}", js1);
    assert!(js1.contains("((_unused_0)=>{"), "Should make an unused parameter name for wildcard");
    
    // Test multiple parameters with wildcards
    let expr2: Expr = parse_quote! {
        some_method(|first, _, third| {
            println!("Using first: {}, ignoring middle, using third: {}", first, third);
            first + third
        })
    };
    
    let js2 = rust_expr_to_js(&expr2);
    println!("Mixed parameters: {}", js2);
    assert!(js2.contains("first"), "Should preserve named parameter");
    assert!(js2.contains("third"), "Should preserve third parameter");
    assert!(js2.contains("((first, _unused_1, third)=>"), "Should have placeholders for wildcard parameters");
    
    // Test wildcard with type annotation
    let expr3: Expr = parse_quote! {
        some_method(|_: i32| {
            println!("Ignoring typed parameter");
            "result"
        })
    };
    
    let js3 = rust_expr_to_js(&expr3);
    println!("Typed wildcard: {}", js3);
    assert!(js3.contains("((_unused_0)=>{"), "Should have placeholder for typed wildcard");
    
    println!("✅ All wildcard closure parameter tests passed!");
}

#[test]
fn test_various_closure_parameter_patterns() {
    println!("Testing various closure parameter patterns...");
    
    let test_cases = vec![
        ("|x| x + 1", "untyped named parameter"),
        ("|x: i32| x + 1", "typed named parameter"), 
        ("|_| 42", "wildcard parameter"),
        ("|_: String| 0", "typed wildcard parameter"),
        ("|a, _, c| a + c", "mixed named and wildcard"),
        ("|_, _| 0", "multiple wildcards"),
    ];
    
    for (rust_closure, description) in test_cases {
        let expr_str = format!("some_func({})", rust_closure);
        let expr: Expr = syn::parse_str(&expr_str)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", description, e));
        
        let js_code = rust_expr_to_js(&expr);
        println!("{}: {}", description, js_code);
        
        // All should generate valid arrow functions without panics
        assert!(js_code.contains("=>"), "Should generate arrow function for: {}", description);
        assert!(!js_code.is_empty(), "Should not be empty for: {}", description);
    }
    
    println!("✅ All closure parameter pattern tests passed!");
}

/*
This test verifies that:

1. Single wildcard parameters |_| are handled correctly
2. Mixed parameters with wildcards |a, _, c| work properly  
3. Typed wildcards |_: Type| are supported
4. Multiple wildcards |_, _| are handled
5. No panics occur for any wildcard pattern variants

The fix generates placeholder parameter names like _unused_0, _unused_1, etc.
for wildcard patterns, allowing the closure to compile to valid JavaScript
while preserving the "ignore this parameter" semantics.
*/
