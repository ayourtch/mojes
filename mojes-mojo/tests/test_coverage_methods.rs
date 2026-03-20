// Tests for method transpilation: get(), contains_key(), is_some(), is_none() with function call receivers,
// keys(), insert(), remove(), and various string methods
use mojes_mojo::*;
use syn::{parse_quote, Expr};

#[test]
fn test_get_method_transpilation() {
    // Tests the "get" method IIFE pattern (lines 2482-2572)
    let expr: Expr = parse_quote! {
        my_map.get("key")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should generate IIFE with typeof check
    assert!(js.contains("typeof") || js.contains("function"));
}

#[test]
fn test_contains_key_transpilation() {
    // Tests the "contains_key" method IIFE pattern (lines 2714-2797)
    let expr: Expr = parse_quote! {
        my_map.contains_key("key")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("hasOwnProperty") || js.contains("has"));
}

#[test]
fn test_is_some_with_function_call_receiver() {
    // Tests is_some with function call receiver (IIFE branch, lines 2606-2650)
    let expr: Expr = parse_quote! {
        get_value().is_some()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should use IIFE to cache function call result
    assert!(js.contains("val") || js.contains("null"));
}

#[test]
fn test_is_none_with_function_call_receiver() {
    // Tests is_none with function call receiver (IIFE branch, lines 2660-2704)
    let expr: Expr = parse_quote! {
        get_value().is_none()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("val") || js.contains("null"));
}

#[test]
fn test_is_some_with_simple_receiver() {
    // Tests is_some without function call receiver (lines 2651-2658)
    let expr: Expr = parse_quote! {
        x.is_some()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("null") && js.contains("undefined"));
}

#[test]
fn test_is_none_with_simple_receiver() {
    // Tests is_none without function call receiver (lines 2706-2712)
    let expr: Expr = parse_quote! {
        x.is_none()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("null") && js.contains("undefined"));
}

#[test]
fn test_keys_method() {
    // Tests keys() transpilation (lines 2575-2585)
    let expr: Expr = parse_quote! {
        my_map.keys()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("Object.keys"));
}

#[test]
fn test_insert_method() {
    // Tests insert() transpilation (IIFE pattern)
    let expr: Expr = parse_quote! {
        my_map.insert("key", "value")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should generate universal insert IIFE
    assert!(js.contains("splice") || js.contains("obj"));
}

#[test]
fn test_string_methods() {
    let expr: Expr = parse_quote! { s.starts_with("hello") };
    assert!(rust_expr_to_js(&expr).contains("startsWith"));

    let expr: Expr = parse_quote! { s.ends_with("world") };
    assert!(rust_expr_to_js(&expr).contains("endsWith"));

    let expr: Expr = parse_quote! { s.replace("a", "b") };
    assert!(rust_expr_to_js(&expr).contains("replace"));

    let expr: Expr = parse_quote! { s.split(",") };
    assert!(rust_expr_to_js(&expr).contains("split"));

    let expr: Expr = parse_quote! { parts.join(",") };
    assert!(rust_expr_to_js(&expr).contains("join"));
}

#[test]
fn test_iter_and_collect_noop() {
    let expr: Expr = parse_quote! { items.iter() };
    let js = rust_expr_to_js(&expr);
    println!("iter JS: {}", &js);
    // iter() should be a no-op, just return receiver
    assert!(js.contains("items"));

    let expr: Expr = parse_quote! { items.collect() };
    let js = rust_expr_to_js(&expr);
    println!("collect JS: {}", &js);
    assert!(js.contains("items"));
}

#[test]
fn test_unwrap_noop() {
    let expr: Expr = parse_quote! { value.unwrap() };
    let js = rust_expr_to_js(&expr);
    println!("unwrap JS: {}", &js);
    // unwrap should just return the receiver
    assert!(js.contains("value"));
}
