use mojes_mojo::*;
use syn::{parse_quote, Block};

#[test]
fn test_block_with_await_generates_async_iife() {
    // A block expression containing .await should generate an async IIFE
    let block: Block = parse_quote! {
        {
            let result = {
                let data = fetch_data().await;
                data
            };
        }
    };
    let js = rust_block_to_js(&block);
    println!("Block with await JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in IIFE output when block contains await, got: {}",
        js
    );
    assert!(
        js.contains("await"),
        "Expected 'await' in output, got: {}",
        js
    );
}

#[test]
fn test_block_without_await_generates_non_async_iife() {
    // A block expression WITHOUT .await should generate a non-async IIFE
    let block: Block = parse_quote! {
        {
            let result = {
                let x = 42;
                x + 1
            };
        }
    };
    let js = rust_block_to_js(&block);
    println!("Block without await JS: {}", &js);
    // The IIFE should NOT be async
    assert!(
        !js.contains("async"),
        "Expected no 'async' in IIFE output when block has no await, got: {}",
        js
    );
}

#[test]
fn test_match_with_await_generates_async_iife() {
    // A match expression with .await in an arm should generate an async IIFE
    let block: Block = parse_quote! {
        {
            let result = match value {
                1 => fetch_one().await,
                _ => fetch_default().await,
            };
        }
    };
    let js = rust_block_to_js(&block);
    println!("Match with await JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in IIFE output when match arm contains await, got: {}",
        js
    );
    assert!(
        js.contains("await"),
        "Expected 'await' in output, got: {}",
        js
    );
}

#[test]
fn test_if_expression_with_await_generates_async_iife() {
    // An if-as-expression with .await in a branch should generate an async IIFE
    let block: Block = parse_quote! {
        {
            let result = if condition {
                fetch_a().await
            } else {
                fetch_b().await
            };
        }
    };
    let js = rust_block_to_js(&block);
    println!("If-expr with await JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in IIFE output when if-expression contains await, got: {}",
        js
    );
    assert!(
        js.contains("await"),
        "Expected 'await' in output, got: {}",
        js
    );
}
