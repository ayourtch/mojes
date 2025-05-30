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
fn test_method_call_expressions_0() {
    // Basic method calls
    let expr: Expr = parse_quote!(arr.len());
    assert_eq!(rust_expr_to_js(&expr), "arr.length");

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

// Fix the test_method_call_expressions test in expressions.rs

#[test]
fn test_method_call_expressions() {
    // Basic method calls
    let expr: Expr = parse_quote!(arr.len());
    assert_eq!(rust_expr_to_js(&expr), "arr.length"); // Property, not method!

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

// Add additional test to verify the distinction between methods and properties
#[test]
fn test_method_vs_property_distinction() {
    // Properties (no parentheses in JavaScript)
    let expr: Expr = parse_quote!(arr.len());
    assert_eq!(rust_expr_to_js(&expr), "arr.length");

    let expr: Expr = parse_quote!(text.len());
    assert_eq!(rust_expr_to_js(&expr), "text.length");

    // Methods (keep parentheses in JavaScript)
    let expr: Expr = parse_quote!(arr.push(item));
    assert_eq!(rust_expr_to_js(&expr), "arr.push(item)");

    let expr: Expr = parse_quote!(text.trim());
    assert_eq!(rust_expr_to_js(&expr), "text.trim()");

    let expr: Expr = parse_quote!(text.to_uppercase());
    assert_eq!(rust_expr_to_js(&expr), "text.toUpperCase()");
}

// Test that the generated JavaScript actually works
#[test]
fn test_method_call_execution() {
    // Test array length
    let expr: Expr = parse_quote!(arr.len());
    let js_code = rust_expr_to_js(&expr);

    let test_code = format!("const arr = [1, 2, 3, 4, 5]; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Test string length
    let expr: Expr = parse_quote!(text.len());
    let js_code = rust_expr_to_js(&expr);

    let test_code = format!("const text = 'hello'; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Test method call
    let expr: Expr = parse_quote!(text.to_uppercase());
    let js_code = rust_expr_to_js(&expr);

    let test_code = format!("const text = 'hello'; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "HELLO");
}

// Test method chaining with proper property access
#[test]
fn test_corrected_method_chaining() {
    let expr: Expr = parse_quote!(text.trim().to_uppercase().len());
    let js_code = rust_expr_to_js(&expr);

    // Should end with .length (property), not .length() (method call)
    assert_eq!(js_code, "text.trim().toUpperCase().length");

    // Test execution
    let test_code = format!("const text = '  hello  '; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0); // "hello".length
}

// Test all the string methods we support
#[test]
fn test_all_string_methods() {
    let test_cases = vec![
        // Methods (keep parentheses)
        (parse_quote!(s.trim()), "s.trim()"),
        (parse_quote!(s.to_uppercase()), "s.toUpperCase()"),
        (parse_quote!(s.to_lowercase()), "s.toLowerCase()"),
        (parse_quote!(s.trim_start()), "s.trimStart()"),
        (parse_quote!(s.trim_end()), "s.trimEnd()"),
        (parse_quote!(s.starts_with("x")), "s.startsWith(\"x\")"),
        (parse_quote!(s.ends_with("x")), "s.endsWith(\"x\")"),
        (parse_quote!(s.replace("a", "b")), "s.replace(\"a\", \"b\")"),
        (parse_quote!(s.split(",")), "s.split(\",\")"),
        (parse_quote!(s.contains("test")), "s.includes(\"test\")"),
        // Properties (no parentheses)
        (parse_quote!(s.len()), "s.length"),
    ];

    for (expr, expected) in test_cases {
        let js_code = rust_expr_to_js(&expr);
        assert_eq!(js_code, expected, "Failed for expression: {:?}", expr);
    }
}

// Test all the array/vector methods we support
#[test]
fn test_all_array_methods() {
    let test_cases = vec![
        // Methods (keep parentheses)
        (parse_quote!(arr.push(item)), "arr.push(item)"),
        (parse_quote!(arr.pop()), "arr.pop()"),
        (parse_quote!(arr.remove(index)), "arr.splice(index, 1)[0]"),
        (parse_quote!(arr.insert(0, item)), "arr.splice(0, 0, item)"),
        (parse_quote!(arr.map(func)), "arr.map(func)"),
        (parse_quote!(arr.filter(pred)), "arr.filter(pred)"),
        (parse_quote!(arr.find(pred)), "arr.find(pred)"),
        (parse_quote!(arr.contains(item)), "arr.includes(item)"),
        // Properties (no parentheses)
        (parse_quote!(arr.len()), "arr.length"),
    ];

    for (expr, expected) in test_cases {
        let js_code = rust_expr_to_js(&expr);
        assert_eq!(js_code, expected, "Failed for expression: {:?}", expr);
    }
}

#[test]
fn test_array_expressions() {
    // Array literal - test with evaluation
    let expr: Expr = parse_quote!([1, 2, 3]);
    let js_code = rust_expr_to_js(&expr);

    // Test that the array is properly created and has correct length
    let test_code = format!("const arr = {}; arr.length", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Test individual elements
    let test_code = format!("const arr = {}; arr[0]", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let test_code = format!("const arr = {}; arr[1]", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);

    let test_code = format!("const arr = {}; arr[2]", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Empty array
    let expr: Expr = parse_quote!([]);
    let js_code = rust_expr_to_js(&expr);
    let test_code = format!("const arr = {}; arr.length", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    // Array with variables - test structure only since we can't evaluate variables
    let expr: Expr = parse_quote!([a, b, c]);
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("a") && js_code.contains("b") && js_code.contains("c"));

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
    println!("DEBUG test_struct_instantiation js code: {}", &result);
    assert!(result.contains("10"));
    assert!(result.contains("20"));
    assert!(result.starts_with("new Point(") && result.ends_with(")"));
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
    assert_eq!(rust_expr_to_js(&expr), "42");

    let expr: Expr = parse_quote!(return);
    assert_eq!(rust_expr_to_js(&expr), "undefined");
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
