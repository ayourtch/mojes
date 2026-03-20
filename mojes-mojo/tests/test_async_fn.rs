use mojes_mojo::*;
use syn::{parse_quote, Block};

#[test]
fn test_async_fn_produces_async_function() {
    // An async fn defined inside a block should transpile to "async function"
    let block: Block = parse_quote! {
        {
            async fn foo() -> i32 {
                42
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("Async fn JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in output, got: {}",
        js
    );
}

#[test]
fn test_regular_fn_not_async() {
    // A regular (non-async) fn should NOT contain "async"
    let block: Block = parse_quote! {
        {
            fn bar() -> i32 {
                42
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("Regular fn JS: {}", &js);
    assert!(
        !js.contains("async"),
        "Regular function should NOT contain 'async', got: {}",
        js
    );
}

#[test]
fn test_async_fn_with_await() {
    // An async fn with .await should generate valid JS with async function and await
    let block: Block = parse_quote! {
        {
            async fn fetch_data() -> String {
                let result = get_data().await;
                result
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("Async fn with await JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in output, got: {}",
        js
    );
    assert!(
        js.contains("await"),
        "Expected 'await' in output, got: {}",
        js
    );
}

#[test]
fn test_async_method_in_impl() {
    // An async method in an impl block should transpile to async function
    let impl_block: syn::ItemImpl = parse_quote! {
        impl MyStruct {
            async fn do_something(&self) -> i32 {
                let val = compute().await;
                val
            }
        }
    };
    let js = generate_js_methods_for_impl(&impl_block);
    println!("Async method JS: {}", &js);
    assert!(
        js.contains("async function"),
        "Expected 'async function' in impl method output, got: {}",
        js
    );
    assert!(
        js.contains("await"),
        "Expected 'await' in impl method output, got: {}",
        js
    );
}
