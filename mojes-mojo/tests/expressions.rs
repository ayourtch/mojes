// tests/expressions.rs
use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// Helper to test expression evaluation with variables
fn eval_expr_with_vars(expr_js: &str, vars: &[(&str, &str)]) -> JsResult<JsValue> {
    let var_declarations: String = vars
        .iter()
        .map(|(name, value)| format!("const {} = {};", name, value))
        .collect::<Vec<_>>()
        .join("\n");

    let code = format!("{}\n{}", var_declarations, expr_js);
    eval_js(&code)
}

#[test]
fn test_literal_expressions() {
    // Integer literals
    let expr: Expr = parse_quote!(42);
    assert_eq!(rust_expr_to_js(&expr), "42");

    let expr: Expr = parse_quote!(-17);
    let result = rust_expr_to_js(&expr);
    let js_result = eval_js(&result).unwrap();
    assert_eq!(js_result.as_number().unwrap(), -17.0);

    // Float literals
    let expr: Expr = parse_quote!(3.14);
    assert_eq!(rust_expr_to_js(&expr), "3.14");

    // String literals
    let expr: Expr = parse_quote!("hello world");
    assert_eq!(rust_expr_to_js(&expr), "\"hello world\"");

    // Boolean literals
    let expr: Expr = parse_quote!(true);
    assert_eq!(rust_expr_to_js(&expr), "true");

    let expr: Expr = parse_quote!(false);
    assert_eq!(rust_expr_to_js(&expr), "false");

    // Character literals (should become strings in JS)
    let expr: Expr = parse_quote!('x');
    assert_eq!(rust_expr_to_js(&expr), "\"x\"");
}

#[test]
fn test_binary_expressions() {
    // Arithmetic operations
    let expr: Expr = parse_quote!(a + b);
    assert_eq!(rust_expr_to_js(&expr), "a + b");

    let expr: Expr = parse_quote!(x - y);
    assert_eq!(rust_expr_to_js(&expr), "x - y");

    let expr: Expr = parse_quote!(a * b);
    assert_eq!(rust_expr_to_js(&expr), "a * b");

    let expr: Expr = parse_quote!(x / y);
    assert_eq!(rust_expr_to_js(&expr), "x / y");

    let expr: Expr = parse_quote!(a % b);
    assert_eq!(rust_expr_to_js(&expr), "a % b");

    // Comparison operations
    let expr: Expr = parse_quote!(a == b);
    assert_eq!(rust_expr_to_js(&expr), "a === b");

    let expr: Expr = parse_quote!(x != y);
    assert_eq!(rust_expr_to_js(&expr), "x !== y");

    let expr: Expr = parse_quote!(a < b);
    assert_eq!(rust_expr_to_js(&expr), "a < b");

    let expr: Expr = parse_quote!(x > y);
    assert_eq!(rust_expr_to_js(&expr), "x > y");

    let expr: Expr = parse_quote!(a <= b);
    assert_eq!(rust_expr_to_js(&expr), "a <= b");

    let expr: Expr = parse_quote!(x >= y);
    assert_eq!(rust_expr_to_js(&expr), "x >= y");

    // Logical operations
    let expr: Expr = parse_quote!(a && b);
    assert_eq!(rust_expr_to_js(&expr), "a && b");

    let expr: Expr = parse_quote!(x || y);
    assert_eq!(rust_expr_to_js(&expr), "x || y");
}

#[test]
fn test_binary_expressions_with_evaluation() {
    // Test arithmetic with actual values
    let expr: Expr = parse_quote!(5 + 3);
    let js_code = rust_expr_to_js(&expr);
    let result = eval_js(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 8.0);

    // Test string concatenation detection
    let expr: Expr = parse_quote!("hello" + "world");
    let js_code = rust_expr_to_js(&expr);
    // Should use template literal for string concat
    assert!(js_code.contains("$"));

    // Test mixed operations
    let expr: Expr = parse_quote!(10 - 3 * 2);
    let js_code = rust_expr_to_js(&expr);
    let result = eval_js(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);
}

#[test]
fn test_variable_expressions() {
    let expr: Expr = parse_quote!(my_variable);
    assert_eq!(rust_expr_to_js(&expr), "my_variable");

    let expr: Expr = parse_quote!(snake_case_var);
    assert_eq!(rust_expr_to_js(&expr), "snake_case_var");

    // Test special constants
    let expr: Expr = parse_quote!(None);
    assert_eq!(rust_expr_to_js(&expr), "null");
}

#[test]
fn test_function_call_expressions() {
    // Simple function call
    let expr: Expr = parse_quote!(my_function());
    assert_eq!(rust_expr_to_js(&expr), "my_function()");

    // Function call with arguments
    let expr: Expr = parse_quote!(add(1, 2));
    assert_eq!(rust_expr_to_js(&expr), "add(1, 2)");

    // Nested function calls
    let expr: Expr = parse_quote!(outer(inner(x)));
    assert_eq!(rust_expr_to_js(&expr), "outer(inner(x))");

    // Option::Some handling
    let expr: Expr = parse_quote!(Some(42));
    assert_eq!(rust_expr_to_js(&expr), "42");
}

#[test]
fn test_method_call_expressions() {
    // Basic method calls
    let expr: Expr = parse_quote!(arr.len());
    assert_eq!(rust_expr_to_js(&expr), "arr.length()");

    let expr: Expr = parse_quote!(vec.push(item));
    assert_eq!(rust_expr_to_js(&expr), "vec.push(item)");

    let expr: Expr = parse_quote!(text.contains("test"));
    assert_eq!(rust_expr_to_js(&expr), "text.includes(\"test\")");

    // Option method calls
    let expr: Expr = parse_quote!(opt.is_some());
    let result = rust_expr_to_js(&expr);
    assert!(result.contains("!== null") && result.contains("!== undefined"));

    let expr: Expr = parse_quote!(opt.is_none());
    let result = rust_expr_to_js(&expr);
    assert!(result.contains("=== null") || result.contains("=== undefined"));

    let expr: Expr = parse_quote!(opt.unwrap());
    assert_eq!(rust_expr_to_js(&expr), "opt");
}

#[test]
fn test_array_expressions() {
    // Array literal
    let expr: Expr = parse_quote!([1, 2, 3]);
    assert_eq!(rust_expr_to_js(&expr), "[1, 2, 3]");

    // Empty array
    let expr: Expr = parse_quote!([]);
    assert_eq!(rust_expr_to_js(&expr), "[]");

    // Array with variables
    let expr: Expr = parse_quote!([a, b, c]);
    assert_eq!(rust_expr_to_js(&expr), "[a, b, c]");

    // Array indexing
    let expr: Expr = parse_quote!(arr[0]);
    assert_eq!(rust_expr_to_js(&expr), "arr[0]");

    let expr: Expr = parse_quote!(matrix[i][j]);
    assert_eq!(rust_expr_to_js(&expr), "matrix[i][j]");
}

#[test]
fn test_array_expressions_with_evaluation() {
    // Test array creation and access
    let expr: Expr = parse_quote!([10, 20, 30]);
    let js_code = rust_expr_to_js(&expr);

    let test_code = format!("const arr = {}; arr[1];", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);
}

#[test]
fn test_field_access_expressions() {
    let expr: Expr = parse_quote!(person.name);
    assert_eq!(rust_expr_to_js(&expr), "person.name");

    let expr: Expr = parse_quote!(obj.field.subfield);
    assert_eq!(rust_expr_to_js(&expr), "obj.field.subfield");
}

#[test]
fn test_struct_instantiation() {
    let expr: Expr = parse_quote!(Point { x: 10, y: 20 });
    let result = rust_expr_to_js(&expr);
    assert!(result.contains("x: 10"));
    assert!(result.contains("y: 20"));
    assert!(result.starts_with("{") && result.ends_with("}"));
}

#[test]
fn test_assignment_expressions() {
    let expr: Expr = parse_quote!(x = 42);
    assert_eq!(rust_expr_to_js(&expr), "x = 42");

    let expr: Expr = parse_quote!(obj.field = value);
    assert_eq!(rust_expr_to_js(&expr), "obj.field = value");
}

#[test]
fn test_parenthesized_expressions() {
    let expr: Expr = parse_quote!((a + b));
    assert_eq!(rust_expr_to_js(&expr), "(a + b)");

    let expr: Expr = parse_quote! {((nested))};
    assert_eq!(rust_expr_to_js(&expr), "((nested))");
}

#[test]
fn test_return_expressions() {
    let expr: Expr = parse_quote!(return 42);
    assert_eq!(rust_expr_to_js(&expr), "return 42");

    let expr: Expr = parse_quote!(return);
    assert_eq!(rust_expr_to_js(&expr), "return");
}

#[test]
fn test_complex_nested_expressions() {
    // Complex arithmetic with parentheses
    let expr: Expr = parse_quote!((a + b) * (c - d));
    assert_eq!(rust_expr_to_js(&expr), "(a + b) * (c - d)");

    // Method chaining
    let expr: Expr = parse_quote!(text.trim().to_uppercase());
    let result = rust_expr_to_js(&expr);
    assert!(result.contains("trim") && result.contains("toUpperCase"));

    // Function call with complex arguments
    let expr: Expr = parse_quote!(calculate(x * 2, y + 1));
    assert_eq!(rust_expr_to_js(&expr), "calculate(x * 2, y + 1)");
}
