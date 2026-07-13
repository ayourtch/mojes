// Tests for unwrap_or, unwrap_or_else, unwrap_or_default transpilation.
// These now emit a Result-aware dispatcher instead of a bare `??`:
// null/undefined (None) and {error: ..} (Err) take the default,
// {ok: v} unwraps to v, and any plain value passes through.
use mojes_mojo::*;
use syn::{parse_quote, Expr, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_unwrap_or_transpilation() {
    // x.unwrap_or(42) should produce x ?? 42
    let expr: Expr = parse_quote! {
        x.unwrap_or(42)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS unwrap_or: {}", &js);
    assert!(js.contains("v && v.error !== undefined"), "Expected Result-aware dispatch, got: {}", &js);
    assert!(js.contains("42"), "Expected default value 42, got: {}", &js);
}

#[test]
fn test_unwrap_or_string_default() {
    // x.unwrap_or("default") should produce x ?? "default"
    let expr: Expr = parse_quote! {
        x.unwrap_or("default")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS unwrap_or string: {}", &js);
    assert!(js.contains("v && v.error !== undefined"), "Expected Result-aware dispatch, got: {}", &js);
    assert!(js.contains("default"), "Expected default string value, got: {}", &js);
}

#[test]
fn test_unwrap_or_else_transpilation() {
    // x.unwrap_or_else(|| compute()) should produce x ?? compute()
    let expr: Expr = parse_quote! {
        x.unwrap_or_else(|| compute())
    };
    let js = rust_expr_to_js(&expr);
    println!("JS unwrap_or_else: {}", &js);
    assert!(js.contains("v && v.error !== undefined"), "Expected Result-aware dispatch, got: {}", &js);
}

#[test]
fn test_unwrap_or_default_transpilation() {
    // x.unwrap_or_default() should produce x ?? null
    let expr: Expr = parse_quote! {
        x.unwrap_or_default()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS unwrap_or_default: {}", &js);
    assert!(js.contains("v && v.error !== undefined"), "Expected Result-aware dispatch, got: {}", &js);
    assert!(js.contains("null"), "Expected null as default, got: {}", &js);
}

#[test]
fn test_unwrap_or_execution_with_value() {
    // When receiver is not null, unwrap_or should return receiver value
    let block: Block = parse_quote! {
        {
            let x = 10;
            x.unwrap_or(42)
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS unwrap_or exec with value: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    // x is 10, not null/undefined, so result should be 10
    assert_eq!(result.as_number().unwrap(), 10.0);
}

#[test]
fn test_unwrap_or_execution_with_null() {
    // When receiver is null, unwrap_or should return default
    let block: Block = parse_quote! {
        {
            let x = None;
            x.unwrap_or(42)
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS unwrap_or exec with null: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    // x is None (null), so ?? should return 42
    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[test]
fn test_unwrap_or_default_execution_with_null() {
    // When receiver is null, unwrap_or_default should return null
    let block: Block = parse_quote! {
        {
            let x = None;
            x.unwrap_or_default()
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS unwrap_or_default exec with null: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    // x is None (null), ?? null should return null
    assert!(result.is_null(), "Expected null result");
}
