use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

#[test]
fn test_wildcard_in_tuple_struct_execution() {
    // Test simple wildcard pattern with execution
    let expr: Expr = parse_quote! {
        match result {
            Ok(value) => value,
            Err(_) => "error occurred",  // This should not panic anymore
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Simple wildcard pattern: {}", js_code);
    
    // Test with Ok variant - using correct Result format
    let test_code_ok = format!(r#"
        const result = {{ ok: "success" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_ok).unwrap();
    println!("Result test_wildcard_in_tuple_struct_execution: {:?}", &result);
    assert_eq!(result.as_string().unwrap(), "success");
    
    // Test with Err variant (wildcard should be ignored) - using correct Result format
    let test_code_err = format!(r#"
        const result = {{ error: "some error details" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_err).unwrap();
    assert_eq!(result.as_string().unwrap(), "error occurred");
}

#[test]
fn test_mixed_wildcard_and_ident_patterns_execution() {
    // Test mixed patterns with execution
    let expr: Expr = parse_quote! {
        match data {
            First(value, _) => value,      // Use value, ignore second
            Second(_, backup) => backup,   // Ignore first, use backup
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Mixed wildcard pattern: {}", js_code);
    
    // Test First variant (use first, ignore second)
    let test_code_first = format!(r#"
        const data = {{ type: "First", value0: "important", value1: "ignored" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_first).unwrap();
    assert_eq!(result.as_string().unwrap(), "important");
    
    // Test Second variant (ignore first, use second)
    let test_code_second = format!(r#"
        const data = {{ type: "Second", value0: "ignored", value1: "backup" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_second).unwrap();
    assert_eq!(result.as_string().unwrap(), "backup");
}

#[test]
fn test_multiple_wildcards_execution() {
    // Test multiple wildcards with execution
    let expr: Expr = parse_quote! {
        match complex {
            Triple(_, _, important) => important,  // Two wildcards, one binding
            Single(_) => "single matched",         // Single wildcard
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Multiple wildcards pattern: {}", js_code);
    
    // Test Triple variant (ignore first two, use third)
    let test_code_triple = format!(r#"
        const complex = {{ type: "Triple", value0: "ignore1", value1: "ignore2", value2: "keep this" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_triple).unwrap();
    assert_eq!(result.as_string().unwrap(), "keep this");
    
    // Test Single variant (ignore the value)
    let test_code_single = format!(r#"
        const complex = {{ type: "Single", value0: "ignored value" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_single).unwrap();
    assert_eq!(result.as_string().unwrap(), "single matched");
}

#[test]
fn test_all_wildcards_execution() {
    // Test pattern where all elements are wildcards
    let expr: Expr = parse_quote! {
        match ignored {
            Pair(_, _) => 42,           // All wildcards, return number
            Triple(_, _, _) => 100,     // All wildcards, return different number
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("All wildcards pattern: {}", js_code);
    
    // Test Pair variant (ignore both values)
    let test_code_pair = format!(r#"
        const ignored = {{ type: "Pair", value0: "ignore1", value1: "ignore2" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_pair).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
    
    // Test Triple variant (ignore all three values)
    let test_code_triple = format!(r#"
        const ignored = {{ type: "Triple", value0: "ignore1", value1: "ignore2", value2: "ignore3" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code_triple).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);
}

#[test]
fn test_wildcard_pattern_does_not_panic() {
    // Ensure the fix prevents the original panic
    let expr: Expr = parse_quote! {
        match status {
            Success(message, _) => message,  // This used to panic
            Error(_) => "failed",            // This used to panic  
        }
    };

    // Should not panic during transpilation
    let js_code = rust_expr_to_js(&expr);
    println!("No panic test: {}", js_code);
    
    // Verify it generates working JavaScript
    let test_code = format!(r#"
        const status = {{ type: "Success", value0: "all good", value1: "ignored details" }};
        const output = {};
        output;
    "#, js_code);
    
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "all good");
}
