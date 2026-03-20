// Tests for format!, println!, vec! macros and format string parsing
// Covers lines 3006-3082 (vec!), 3142-3307 (format), and the debug format specifier
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_vec_with_repeat() {
    // Tests vec![value; count] pattern (lines 3023-3059)
    let expr: Expr = parse_quote! {
        vec![0; 5]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("Array.from"));
    assert!(js.contains("length"));
}

#[test]
fn test_vec_with_elements() {
    // Tests vec![a, b, c] pattern (lines 3061-3078)
    let expr: Expr = parse_quote! {
        vec![1, 2, 3]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("[") && js.contains("]"));
    assert!(js.contains("1") && js.contains("2") && js.contains("3"));
}

#[test]
fn test_vec_empty() {
    let expr: Expr = parse_quote! {
        vec![]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("[]"));
}

#[test]
fn test_format_with_placeholders() {
    // Tests format! macro with {} placeholders via handle_format_macro
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("Hello, {}!"));
        p.push(parse_quote!(name));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS: {}", &js);
    assert!(js.contains("`") || js.contains("Hello"));
}

#[test]
fn test_format_without_placeholders() {
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("static string"));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS: {}", &js);
    assert!(js.contains("static string"));
}

#[test]
fn test_println_transpilation() {
    let block: Block = parse_quote! {
        {
            println!("Hello, {}!", "world");
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("console.log"));
}

#[test]
fn test_eprintln_transpilation() {
    let block: Block = parse_quote! {
        {
            eprintln!("Error: {}", "something went wrong");
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("console.error"));
}

#[test]
fn test_format_debug_specifier() {
    // Tests {:?} debug format specifier
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("debug: {:?}"));
        p.push(parse_quote!(x));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS: {}", &js);
    assert!(js.contains("debug") || js.contains("`"));
}
