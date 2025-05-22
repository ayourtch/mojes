// precedence_analysis.rs - When parentheses matter in transpiled code

use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Expr, parse_quote};

fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// Test cases where precedence could matter
#[test]
fn test_precedence_safety() {
    // Case 1: Basic arithmetic - precedence is preserved
    let expr: Expr = parse_quote!(a + b * c);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "a + b * c");

    // Test execution to verify precedence
    let test_code = "const a = 2, b = 3, c = 4; a + b * c";
    let result = eval_js(test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 14.0); // 2 + (3 * 4), not (2 + 3) * 4
}

#[test]
fn test_comparison_precedence() {
    // Case 2: Comparison operators
    let expr: Expr = parse_quote!(a + b > c);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "a + b > c");

    // Verify precedence: addition before comparison
    let test_code = "const a = 2, b = 3, c = 4; a + b > c";
    let result = eval_js(test_code).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true); // (2 + 3) > 4 = 5 > 4 = true
}

#[test]
fn test_logical_operator_precedence() {
    // Case 3: Logical operators
    let expr: Expr = parse_quote!(a && b || c);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "a && b || c");

    // && has higher precedence than ||
    let test_code = "const a = false, b = true, c = true; a && b || c";
    let result = eval_js(test_code).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true); // (false && true) || true = false || true = true
}

#[test]
fn test_parenthesized_expressions() {
    // Case 4: Explicit parentheses should be preserved
    let expr: Expr = parse_quote!((a + b) * c);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "(a + b) * c");

    // Verify different result from a + b * c
    let test1 = "const a = 2, b = 3, c = 4; (a + b) * c";
    let test2 = "const a = 2, b = 3, c = 4; a + b * c";

    let result1 = eval_js(test1).unwrap();
    let result2 = eval_js(test2).unwrap();

    assert_eq!(result1.as_number().unwrap(), 20.0); // (2 + 3) * 4 = 20
    assert_eq!(result2.as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
}

#[test]
fn test_return_statement_precedence() {
    // The specific case: does return need parentheses?

    // Simple case - no parentheses needed
    let rust_code = "{ x + 1 }";
    let block: syn::Block = syn::parse_str(rust_code).unwrap();
    let js_code = rust_block_to_js(&block);

    assert!(js_code.contains("return x + 1;"));

    // Complex case - precedence still preserved without extra parentheses
    let rust_code = "{ a + b * c - d }";
    let block: syn::Block = syn::parse_str(rust_code).unwrap();
    let js_code = rust_block_to_js(&block);

    assert!(js_code.contains("return a + b * c - d;"));

    // Test that it executes correctly
    let wrapped_code = format!(
        "const a = 1, b = 2, c = 3, d = 4; (function() {{ {} }})()",
        js_code
    );
    let result = eval_js(&wrapped_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0); // 1 + (2 * 3) - 4 = 1 + 6 - 4 = 3
}

// Test edge cases where you MIGHT want parentheses for clarity
#[test]
fn test_complex_expression_clarity() {
    // Very complex expression - might benefit from parentheses for readability
    let expr: Expr = parse_quote!(a && b || c && d);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "a && b || c && d");

    // This is equivalent to: (a && b) || (c && d) due to precedence
    let test_code = "const a = true, b = false, c = false, d = true; a && b || c && d";
    let result = eval_js(test_code).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false); // (true && false) || (false && true) = false || false = false
}

// Conclusion test: return statement precedence
#[test]
fn test_return_precedence_conclusion() {
    use syn::Block;

    // Test various complexity levels in return statements
    let test_cases = vec![
        ("{ x }", "return x;"),
        ("{ x + 1 }", "return x + 1;"),
        ("{ a + b * c }", "return a + b * c;"),
        ("{ a && b || c }", "return a && b || c;"),
        ("{ (a + b) * c }", "return (a + b) * c;"), // Parentheses preserved
    ];

    for (rust_input, expected_return) in test_cases {
        let block: Block = syn::parse_str(rust_input).unwrap();
        let js_code = rust_block_to_js(&block);
        assert!(
            js_code.contains(expected_return),
            "Input: {} did not produce expected: {}\nActual: {}",
            rust_input,
            expected_return,
            js_code
        );
    }
}

// Performance consideration test
#[test]
fn test_unnecessary_parentheses_performance() {
    // Adding extra parentheses doesn't hurt performance, but isn't needed

    let expr1: Expr = parse_quote!(x + 1);
    let js1 = rust_expr_to_js(&expr1);

    // If we were to add unnecessary parentheses:
    let unnecessary_parens = format!("({})", js1);

    // Both should execute the same
    let test1 = format!("const x = 5; {}", js1);
    let test2 = format!("const x = 5; {}", unnecessary_parens);

    let result1 = eval_js(&test1).unwrap();
    let result2 = eval_js(&test2).unwrap();

    assert_eq!(result1.as_number().unwrap(), result2.as_number().unwrap());

    println!("Without parens: {}", js1);
    println!("With parens: {}", unnecessary_parens);
    println!("Both work identically, but without parens is cleaner");
}
