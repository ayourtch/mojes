use mojes_mojo::*;
use syn::{parse_quote, Block};

#[test]
fn test_try_without_await_generates_iife_error_check() {
    // expr? (without await) should generate IIFE with .error property check
    let block: Block = parse_quote! {
        {
            let result = some_function()?;
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("Try without await JS: {}", &js);

    // Should contain the .error check pattern
    assert!(
        js.contains(".error"),
        "Expected '.error' check in output, got: {}",
        js
    );
    assert!(
        js.contains(".ok"),
        "Expected '.ok' unwrap in output, got: {}",
        js
    );
    // Should NOT contain try/catch keywords
    assert!(
        !js.contains("try {") && !js.contains("try{"),
        "Non-await try should NOT generate try/catch, got: {}",
        js
    );
}

#[test]
fn test_try_with_await_generates_try_catch() {
    // expr.await? should generate an async IIFE with try/catch
    let block: Block = parse_quote! {
        {
            let result = fetch(url).await?;
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("Try with await JS: {}", &js);

    // Should contain try and catch keywords
    assert!(
        js.contains("try"),
        "Expected 'try' keyword in output, got: {}",
        js
    );
    assert!(
        js.contains("catch"),
        "Expected 'catch' keyword in output, got: {}",
        js
    );
    // Should contain async since it wraps in an async IIFE
    assert!(
        js.contains("async"),
        "Expected 'async' in output for async IIFE, got: {}",
        js
    );
    // Should contain await
    assert!(
        js.contains("await"),
        "Expected 'await' in output, got: {}",
        js
    );
    // Should contain { error: e } pattern in catch block
    assert!(
        js.contains("error"),
        "Expected 'error' property in catch return, got: {}",
        js
    );
}

#[test]
fn test_try_with_await_in_async_fn() {
    // Test the pattern inside an async function, closer to real usage
    let block: Block = parse_quote! {
        {
            async fn fetch_data(url: String) -> String {
                let response = fetch(url).await?;
                let text = response.text().await?;
                text
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("Async fn with await? JS: {}", &js);

    // Should be an async function
    assert!(
        js.contains("async function"),
        "Expected 'async function' in output, got: {}",
        js
    );
    // Should contain try/catch for the .await? patterns
    assert!(
        js.contains("try"),
        "Expected 'try' keyword in output, got: {}",
        js
    );
    assert!(
        js.contains("catch"),
        "Expected 'catch' keyword in output, got: {}",
        js
    );
}
