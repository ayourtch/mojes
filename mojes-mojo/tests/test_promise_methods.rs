use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();

    let promise_setup = r#"
        function Promise() {}
        Promise.resolve = function(value) {
            return {
                type: 'Promise',
                state: 'resolved',
                value: value,
                then: function(callback) {
                    return Promise.resolve(callback ? callback(this.value) : this.value);
                },
                finally: function(callback) {
                    if (callback) callback();
                    return this;
                }
            };
        };
        Promise.reject = function(reason) {
            return {
                type: 'Promise',
                state: 'rejected',
                reason: reason
            };
        };
        Promise.all = function(promises) {
            return { type: 'Promise', method: 'all', args: promises };
        };
        Promise.race = function(promises) {
            return { type: 'Promise', method: 'race', args: promises };
        };
        Promise.any = function(promises) {
            return { type: 'Promise', method: 'any', args: promises };
        };
        Promise.allSettled = function(promises) {
            return { type: 'Promise', method: 'allSettled', args: promises };
        };

        const console = {
            log: function(...args) {}
        };
    "#;

    context.eval(Source::from_bytes(promise_setup))?;
    context.eval(Source::from_bytes(code))
}

#[test]
fn test_promise_all_transpilation() {
    let expr: Expr = parse_quote! {
        Promise::all(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Promise::all => {}", js_code);
    assert!(js_code.contains("Promise.all(promises)"), "Expected Promise.all(promises), got: {}", js_code);
}

#[test]
fn test_promise_race_transpilation() {
    let expr: Expr = parse_quote! {
        Promise::race(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Promise::race => {}", js_code);
    assert!(js_code.contains("Promise.race(promises)"), "Expected Promise.race(promises), got: {}", js_code);
}

#[test]
fn test_promise_any_transpilation() {
    let expr: Expr = parse_quote! {
        Promise::any(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Promise::any => {}", js_code);
    assert!(js_code.contains("Promise.any(promises)"), "Expected Promise.any(promises), got: {}", js_code);
}

#[test]
fn test_promise_all_settled_transpilation() {
    let expr: Expr = parse_quote! {
        Promise::all_settled(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Promise::all_settled => {}", js_code);
    assert!(js_code.contains("Promise.allSettled(promises)"), "Expected Promise.allSettled(promises), got: {}", js_code);
}

#[test]
fn test_promise_resolve_transpilation() {
    let expr: Expr = parse_quote! {
        Promise::resolve(42)
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Promise::resolve => {}", js_code);
    assert!(js_code.contains("Promise.resolve(42)"), "Expected Promise.resolve(42), got: {}", js_code);
}

#[test]
fn test_promise_finally_transpilation() {
    let expr: Expr = parse_quote! {
        promise.finally(|| {})
    };

    let js_code = rust_expr_to_js(&expr);
    println!("promise.finally => {}", js_code);
    assert!(js_code.contains(".finally("), "Expected .finally( in output, got: {}", js_code);
}

#[test]
fn test_promise_all_execution() {
    let expr: Expr = parse_quote! {
        Promise::all(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    let test_code = format!(r#"
        const promises = [Promise.resolve(1), Promise.resolve(2)];
        const result = {};
        JSON.stringify(result);
    "#, js_code);

    let result = eval_js(&test_code);
    assert!(result.is_ok(), "Promise.all execution failed: {:?}", result.err());
    let val = result.unwrap();
    let output = val.as_string().expect("Expected string result");
    let output_str = output.to_std_string_escaped();
    println!("Promise.all execution result: {}", output_str);
    assert!(output_str.contains("\"method\":\"all\""), "Expected method:'all' in result, got: {}", output_str);
}

#[test]
fn test_promise_all_settled_execution() {
    let expr: Expr = parse_quote! {
        Promise::all_settled(promises)
    };

    let js_code = rust_expr_to_js(&expr);
    let test_code = format!(r#"
        const promises = [Promise.resolve(1), Promise.reject("err")];
        const result = {};
        JSON.stringify(result);
    "#, js_code);

    let result = eval_js(&test_code);
    assert!(result.is_ok(), "Promise.allSettled execution failed: {:?}", result.err());
    let val = result.unwrap();
    let output = val.as_string().expect("Expected string result");
    let output_str = output.to_std_string_escaped();
    println!("Promise.allSettled execution result: {}", output_str);
    assert!(output_str.contains("\"method\":\"allSettled\""), "Expected method:'allSettled' in result, got: {}", output_str);
}
