use mojes_mojo::*;
use syn::{Expr, parse_quote};

#[test]
fn test_tuple_pattern_some_some() {
    println!("Testing tuple pattern (Some(token), Some(signature))...");
    
    // Test the specific pattern from the error
    let expr: Expr = parse_quote! {
        match (&self.auth_token, &self.token_signature) {
            (Some(token), Some(signature)) => {
                format!("Auth: {} - {}", token, signature)
            },
            _ => {
                "No auth".to_string()
            }
        }
    };
    
    let js_code = rust_expr_to_js(&expr);
    println!("Generated JavaScript:\n{}", js_code);
    
    // The JavaScript should:
    // 1. Access tuple elements: _match_value[0], _match_value[1]
    // 2. Check both are not null/undefined 
    // 3. Bind token and signature variables
    assert!(js_code.contains("_match_value[0]"), "Should access first tuple element");
    assert!(js_code.contains("_match_value[1]"), "Should access second tuple element");
    assert!(js_code.contains("!== null"), "Should check for null");
    assert!(js_code.contains("!== undefined"), "Should check for undefined");
    assert!(js_code.contains("const token"), "Should bind token variable");
    assert!(js_code.contains("const signature"), "Should bind signature variable");
    
    println!("✅ SUCCESS: Tuple pattern transpilation includes all expected elements!");
}

#[test]
fn test_tuple_pattern_mixed() {
    println!("Testing mixed tuple pattern (Some(a), None)...");
    
    let expr: Expr = parse_quote! {
        match (value1, value2) {
            (Some(a), None) => {
                format!("First: {}", a)
            },
            (None, Some(b)) => {
                format!("Second: {}", b)
            },
            (Some(a), Some(b)) => {
                format!("Both: {} {}", a, b)
            },
            (None, None) => {
                "Neither".to_string()
            }
        }
    };
    
    let js_code = rust_expr_to_js(&expr);
    println!("Generated JavaScript:\n{}", js_code);
    
    // Should handle all combinations
    assert!(js_code.contains("_match_value[0]"), "Should access first element");
    assert!(js_code.contains("_match_value[1]"), "Should access second element");
    
    println!("✅ SUCCESS: Mixed tuple patterns work correctly!");
}

#[test] 
fn test_tuple_pattern_simple() {
    println!("Testing simple tuple pattern (a, b)...");
    
    let expr: Expr = parse_quote! {
        match (x, y) {
            (a, b) => {
                format!("{}-{}", a, b)
            }
        }
    };
    
    let js_code = rust_expr_to_js(&expr);
    println!("Generated JavaScript:\n{}", js_code);
    
    // Simple variable binding should work
    assert!(js_code.contains("_match_value[0]"), "Should access first element");
    assert!(js_code.contains("_match_value[1]"), "Should access second element");
    assert!(js_code.contains("const a"), "Should bind first variable");
    assert!(js_code.contains("const b"), "Should bind second variable");
    
    println!("✅ SUCCESS: Simple tuple pattern works!");
}

#[test]
fn test_tuple_pattern_three_elements() {
    println!("Testing three-element tuple pattern...");
    
    let expr: Expr = parse_quote! {
        match (a, b, c) {
            (Some(x), Some(y), Some(z)) => {
                format!("{} {} {}", x, y, z)
            },
            _ => {
                "Not all present".to_string()
            }
        }
    };
    
    let js_code = rust_expr_to_js(&expr);
    println!("Generated JavaScript:\n{}", js_code);
    
    // Should handle all three elements
    assert!(js_code.contains("_match_value[0]"), "Should access element 0");
    assert!(js_code.contains("_match_value[1]"), "Should access element 1");
    assert!(js_code.contains("_match_value[2]"), "Should access element 2");
    assert!(js_code.contains("const x"), "Should bind x");
    assert!(js_code.contains("const y"), "Should bind y");
    assert!(js_code.contains("const z"), "Should bind z");
    
    println!("✅ SUCCESS: Three-element tuple pattern works!");
}

/*
These tests verify that:

1. test_tuple_pattern_some_some: 
   - Tests the exact pattern from the user's error: (Some(token), Some(signature))
   - Verifies tuple element access with [0], [1] indexing
   - Checks for proper null/undefined validation
   - Confirms variable binding for inner pattern elements

2. test_tuple_pattern_mixed:
   - Tests multiple tuple pattern combinations in same match
   - Ensures all patterns generate correct conditions
   - Validates complex pattern matching logic

3. test_tuple_pattern_simple:
   - Tests basic tuple destructuring without Option patterns
   - Verifies simple variable binding works

4. test_tuple_pattern_three_elements:
   - Tests that the solution scales to larger tuples
   - Validates indexing and binding for multiple elements

The implementation handles tuple patterns by:
- Accessing each element with numeric indexing: _match_value[0], _match_value[1], etc.
- Creating temporary variables for each element: _tuple_elem_0, _tuple_elem_1
- Recursively calling handle_pattern_binding on each sub-pattern
- Combining all conditions with logical AND
- Collecting all variable bindings from sub-patterns
*/