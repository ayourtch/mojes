// Tests for panic!, todo!, assert!, and dbg! macro transpilation
use mojes_mojo::*;
use syn::{parse_quote, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_panic_with_string_message() {
    let expr: Expr = parse_quote! {
        panic!("something went wrong")
    };
    let js = rust_expr_to_js(&expr);
    println!("panic! JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    assert!(js.contains("something went wrong"));

    // Verify it actually throws when executed
    let result = eval_js(&js);
    assert!(result.is_err(), "panic! should throw an error");
}

#[test]
fn test_panic_with_format_args() {
    let expr: Expr = parse_quote! {
        panic!("value is {}", x)
    };
    let js = rust_expr_to_js(&expr);
    println!("panic! with format JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    // Should contain template literal with interpolation
    assert!(js.contains("x"));
}

#[test]
fn test_panic_no_args() {
    let expr: Expr = parse_quote! {
        panic!()
    };
    let js = rust_expr_to_js(&expr);
    println!("panic!() JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    assert!(js.contains("explicit panic"));
}

#[test]
fn test_todo_no_args() {
    let expr: Expr = parse_quote! {
        todo!()
    };
    let js = rust_expr_to_js(&expr);
    println!("todo!() JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    assert!(js.contains("not yet implemented"));

    // Verify it actually throws when executed
    let result = eval_js(&js);
    assert!(result.is_err(), "todo! should throw an error");
}

#[test]
fn test_todo_with_message() {
    let expr: Expr = parse_quote! {
        todo!("implement this later")
    };
    let js = rust_expr_to_js(&expr);
    println!("todo! with msg JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    assert!(js.contains("implement this later"));
}

#[test]
fn test_assert_simple_condition() {
    let expr: Expr = parse_quote! {
        assert!(x > 0)
    };
    let js = rust_expr_to_js(&expr);
    println!("assert! JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    // Should negate the condition
    assert!(js.contains("!"));

    // Test that it does NOT throw when condition is true
    let test_code = format!("var x = 5; {}", js);
    let result = eval_js(&test_code);
    assert!(result.is_ok(), "assert! should not throw when condition is true");

    // Test that it DOES throw when condition is false
    let test_code_fail = format!("var x = -1; {}", js);
    let result_fail = eval_js(&test_code_fail);
    assert!(result_fail.is_err(), "assert! should throw when condition is false");
}

#[test]
fn test_assert_with_custom_message() {
    let expr: Expr = parse_quote! {
        assert!(x > 0, "x must be positive")
    };
    let js = rust_expr_to_js(&expr);
    println!("assert! with msg JS: {}", &js);
    assert!(js.contains("throw"));
    assert!(js.contains("new Error"));
    assert!(js.contains("x must be positive"));
}

#[test]
fn test_dbg_expression() {
    let expr: Expr = parse_quote! {
        dbg!(value)
    };
    let js = rust_expr_to_js(&expr);
    println!("dbg! JS: {}", &js);
    assert!(js.contains("console.log"));
    assert!(js.contains("value"));

    // dbg! should return the value
    // Provide a console mock since Boa doesn't have console built-in
    let test_code = format!("var console = {{ log: function() {{}} }}; var value = 42; {}", js);
    let result = eval_js(&test_code);
    assert!(result.is_ok(), "dbg! execution failed: {:?}", result.err());
    let val = result.unwrap();
    assert_eq!(val.as_number().unwrap(), 42.0, "dbg! should return the value");
}

#[test]
fn test_dbg_empty() {
    let expr: Expr = parse_quote! {
        dbg!()
    };
    let js = rust_expr_to_js(&expr);
    println!("dbg!() JS: {}", &js);
    assert!(js.contains("console.log"));
}
