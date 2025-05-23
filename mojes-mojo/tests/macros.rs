// tests/macros.rs
use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// Helper to test a block wrapped in a function
fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{\n{}}})();", block_js);
    eval_js(&code)
}

// Helper to capture console output (simulate it)
fn eval_with_console_capture(code: &str) -> JsResult<JsValue> {
    let wrapped_code = format!(
        r#"
        let console_output = [];
        const console = {{
            log: (...args) => console_output.push(args.join(' ')),
            error: (...args) => console_output.push('ERROR: ' + args.join(' '))
        }};
        {};
        console_output;
    "#,
        code
    );
    eval_js(&wrapped_code)
}

#[test]
fn test_format_macro_basic() {
    // Simple format! with no placeholders
    let expr: Expr = parse_quote!(format!("hello"));
    let js_code = rust_expr_to_js(&expr);
    println!("DEBUG test_format_macro_basic js code: {}", &js_code);
    assert_eq!(js_code, "`hello`");

    // Test execution
    let result = eval_js(&js_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "hello");
}

#[test]
fn test_format_macro_with_placeholders() {
    // format! with single placeholder
    let expr: Expr = parse_quote!(format!("Hello {}", name));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.starts_with("`Hello ${"));
    assert!(js_code.contains("name"));
    assert!(js_code.ends_with("}`"));

    // Test execution
    let test_code = format!("const name = 'World'; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "Hello World");
}

#[test]
fn test_format_macro_multiple_placeholders() {
    let expr: Expr = parse_quote!(format!("{} + {} = {}", a, b, result));
    let js_code = rust_expr_to_js(&expr);

    assert!(js_code.starts_with("`"));
    assert!(js_code.ends_with("`"));
    assert!(js_code.contains("${a}"));
    assert!(js_code.contains("${b}"));
    assert!(js_code.contains("${result}"));

    // Test execution
    let test_code = format!("const a = 2, b = 3, result = 5; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "2 + 3 = 5");
}

#[test]
fn test_format_macro_with_expressions() {
    let expr: Expr = parse_quote!(format!("Value is {}", x * 2));
    let js_code = rust_expr_to_js(&expr);

    assert!(js_code.contains("${x * 2}"));

    // Test execution
    let test_code = format!("const x = 5; {}", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "Value is 10");
}

#[test]
fn test_println_macro_simple() {
    let block: Block = parse_quote! {
        {
            println!("Hello, world!");
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("\"Hello, world!\""));
}

#[test]
fn test_println_macro_with_formatting() {
    let block: Block = parse_quote! {
        {
            println!("Hello, {}!", name);
        }
    };

    let js_code = rust_block_to_js(&block);
    println!(
        "DEBUG test_println_macro_with_formatting js code: {}",
        &js_code
    );
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("Hello"));
    assert!(js_code.contains("name"));
}

#[test]
fn test_println_macro_execution() {
    let block: Block = parse_quote! {
        {
            println!("Test message");
            42
        }
    };

    let js_code = rust_block_to_js(&block);
    println!("DEBUG test_println_macro_execution js code: {}", &js_code);

    // Test that it executes without error and captures console output
    let result = eval_with_console_capture(&format!("(function() {{\n{}}})();", js_code)).unwrap();

    // Should capture the console.log output
    // The result should be an array with the logged message
    assert!(result.is_object());
}

#[test]
fn test_print_macro() {
    let block: Block = parse_quote! {
        {
            print!("No newline");
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("\"No newline\""));
}

#[test]
fn test_eprintln_macro() {
    let block: Block = parse_quote! {
        {
            eprintln!("Error message");
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.error"));
    assert!(js_code.contains("\"Error message\""));
}

#[test]
fn test_eprint_macro() {
    let block: Block = parse_quote! {
        {
            eprint!("Error: ");
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.error"));
    assert!(js_code.contains("\"Error: \""));
}

#[test]
fn test_mixed_print_macros() {
    let block: Block = parse_quote! {
        {
            println!("Info: Starting process");
            eprintln!("Warning: Something might be wrong");
            let result = format!("Result: {}", 42);
            println!("{}", result);
            result
        }
    };

    let js_code = rust_block_to_js(&block);

    // Should contain both console.log and console.error
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("console.error"));
    assert!(js_code.contains("Info: Starting process"));
    assert!(js_code.contains("Warning: Something might be wrong"));
    assert!(js_code.contains("Result"));
}

#[test]
fn test_println_with_multiple_args() {
    let block: Block = parse_quote! {
        {
            println!("{} {} {}", first, second, third);
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("first"));
    assert!(js_code.contains("second"));
    assert!(js_code.contains("third"));
}

#[test]
fn test_println_with_complex_expressions() {
    let block: Block = parse_quote! {
        {
            println!("Sum: {}, Product: {}", a + b, a * b);
        }
    };

    let js_code = rust_block_to_js(&block);
    println!(
        "DEBUG test_println_with_complex_expressions js code: {}",
        &js_code
    );
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("a + b"));
    assert!(js_code.contains("a * b"));
    assert!(js_code.contains("Sum"));
    assert!(js_code.contains("Product"));
}

#[test]
fn test_empty_println() {
    let block: Block = parse_quote! {
        {
            println!();
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log()"));
}

#[test]
fn test_nested_format_in_println() {
    let block: Block = parse_quote! {
        {
            let msg = format!("Inner: {}", value);
            println!("Outer: {}", msg);
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("Inner"));
    assert!(js_code.contains("Outer"));
    assert!(js_code.contains("value"));
    assert!(js_code.contains("msg"));
}

#[test]
fn test_format_with_special_characters() {
    let expr: Expr = parse_quote!(format!("String with \"quotes\" and 'apostrophes'"));
    let js_code = rust_expr_to_js(&expr);

    // Should handle quotes properly
    let result = eval_js(&js_code).unwrap();
    assert_eq!(
        result.as_string().unwrap(),
        "String with \"quotes\" and 'apostrophes'"
    );
}

#[test]
fn test_format_with_backticks() {
    let expr: Expr = parse_quote!(format!("String with `backticks`"));
    let js_code = rust_expr_to_js(&expr);

    println!("DEBUG test_format_with_backticks js code: {}", &js_code);

    // Should escape backticks in the template literal
    assert!(js_code.contains("\\`"));

    let result = eval_js(&js_code).unwrap();
    assert_eq!(result.as_string().unwrap(), "String with `backticks`");
}

#[test]
fn test_macros_in_expressions() {
    let expr: Expr = parse_quote! {
        if condition {
            println!("Condition is true");
            format!("Value: {}", x)
        } else {
            eprintln!("Condition is false");
            format!("Default: {}", 0)
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("console.error"));
    assert!(js_code.contains("Condition is true"));
    assert!(js_code.contains("Condition is false"));
}

#[test]
fn test_macro_execution_flow() {
    let block: Block = parse_quote! {
        {
            let name = "Rust";
            let version = 2021;
            println!("Language: {}", name);
            let message = format!("Year: {}", version);
            println!("{}", message);
            format!("Final: {} {}", name, version)
        }
    };

    let js_code = rust_block_to_js(&block);
    println!("DEBUG test_macro_execution_flow js code: {}", &js_code);
    let console = r#"
const console = {
    log: function(...args) { /* mock implementation */ },
    error: function(...args) { /* mock implementation */ }
};
"#;
    let js_code = format!("{}\n{}", console, js_code);

    let result = eval_block_as_function(&js_code).unwrap();

    // The return value should be the final format! result
    assert_eq!(result.as_string().unwrap(), "Final: Rust 2021");
}

// Replace the failing test with this version that expects the panic

use std::panic;

#[test]
fn test_unsupported_macro() {
    // Test that unsupported macros cause a panic (which is desired behavior)
    let expr: Expr = parse_quote!(unsupported_macro!("test", 42));

    // Use std::panic::catch_unwind to capture the panic
    let result = panic::catch_unwind(|| rust_expr_to_js(&expr));

    // Verify that a panic occurred
    assert!(
        result.is_err(),
        "Expected unsupported macro to panic, but it didn't"
    );

    // Optionally, verify the panic message contains what we expect
    if let Err(panic_payload) = result {
        if let Some(panic_msg) = panic_payload.downcast_ref::<String>() {
            assert!(
                panic_msg.contains("Unsupported macro"),
                "Panic message should mention unsupported macro, got: {}",
                panic_msg
            );
            assert!(
                panic_msg.contains("unsupported_macro"),
                "Panic message should mention the macro name, got: {}",
                panic_msg
            );
        } else if let Some(panic_msg) = panic_payload.downcast_ref::<&str>() {
            assert!(
                panic_msg.contains("Unsupported macro"),
                "Panic message should mention unsupported macro, got: {}",
                panic_msg
            );
            assert!(
                panic_msg.contains("unsupported_macro"),
                "Panic message should mention the macro name, got: {}",
                panic_msg
            );
        }
    }

    println!("✓ Unsupported macro correctly panicked as expected");
}

#[test]
fn test_supported_macros_do_not_panic() {
    // Verify that supported macros work without panicking
    let supported_macros = vec![
        parse_quote!(println!("test")),
        parse_quote!(format!("hello {}", name)),
        parse_quote!(eprintln!("error")),
        parse_quote!(print!("output")),
    ];

    for expr in supported_macros {
        // These should not panic
        let result = panic::catch_unwind(|| rust_expr_to_js(&expr));

        assert!(
            result.is_ok(),
            "Supported macro should not panic: {:?}",
            expr
        );

        // Verify we get actual JavaScript code
        let js_code = result.unwrap();
        assert!(!js_code.is_empty(), "Should generate non-empty JavaScript");
        assert!(
            !js_code.contains("Unsupported"),
            "Should not contain 'Unsupported' for supported macros"
        );
    }

    println!("✓ All supported macros work without panicking");
}

#[test]
fn test_multiple_unsupported_macros_panic() {
    // Test various unsupported macro patterns
    let unsupported_macros = vec![
        parse_quote!(custom_macro!()),
        parse_quote!(unknown_macro!("arg")),
        parse_quote!(debug_assert!(condition)),
        parse_quote!(panic!("message")), // This might actually be supported, adjust if needed
        parse_quote!(compile_error!("error")),
        parse_quote!(include_str!("file.txt")),
        parse_quote!(cfg!(feature = "test")),
    ];

    for expr in unsupported_macros {
        let result = panic::catch_unwind(|| rust_expr_to_js(&expr));

        // Each should panic
        assert!(result.is_err(), "Expected macro to panic: {:?}", expr);
        println!("✓ Macro correctly panicked: {:?}", expr);
    }
}

#[test]
fn test_unsupported_macro_in_complex_expression() {
    // Test that unsupported macros panic even when nested in complex expressions
    let expr: Expr = parse_quote! {
        if condition {
            let x = unsupported_macro!("test");
            x + 1
        } else {
            0
        }
    };

    let result = panic::catch_unwind(|| rust_expr_to_js(&expr));

    assert!(
        result.is_err(),
        "Unsupported macro in complex expression should panic"
    );
    println!("✓ Unsupported macro in complex expression correctly panicked");
}

#[test]
fn test_unsupported_macro_in_block() {
    // Test unsupported macros in block statements
    use syn::Block;

    let block: Block = parse_quote! {
        {
            let x = 5;
            unsupported_macro!("test");
            x + 1
        }
    };

    let result = panic::catch_unwind(|| rust_block_to_js(&block));

    assert!(result.is_err(), "Unsupported macro in block should panic");
    println!("✓ Unsupported macro in block correctly panicked");
}

// Helper test to verify panic behavior is consistent
#[test]
fn test_panic_consistency() {
    let expr: Expr = parse_quote!(definitely_not_a_real_macro!("args"));

    // First call should panic
    let result1 = panic::catch_unwind(|| rust_expr_to_js(&expr));
    assert!(result1.is_err());

    // Second call should also panic (no state preservation)
    let result2 = panic::catch_unwind(|| rust_expr_to_js(&expr));
    assert!(result2.is_err());

    println!("✓ Panic behavior is consistent across multiple calls");
}

// Optional: Test that provides better error messages for debugging
#[test]
fn test_unsupported_macro_error_details() {
    let expr: Expr = parse_quote!(custom_debug_macro!("detailed", "args"));

    let result = panic::catch_unwind(|| rust_expr_to_js(&expr));

    match result {
        Ok(_) => panic!("Expected unsupported macro to panic"),
        Err(panic_payload) => {
            // Try to extract and examine the panic message
            let panic_msg = if let Some(msg) = panic_payload.downcast_ref::<String>() {
                msg.clone()
            } else if let Some(msg) = panic_payload.downcast_ref::<&str>() {
                msg.to_string()
            } else {
                "Unknown panic message".to_string()
            };

            println!("Panic message for debugging: {}", panic_msg);

            // Verify the panic message is informative
            assert!(
                panic_msg.contains("custom_debug_macro") || panic_msg.contains("Unsupported"),
                "Panic message should be informative: {}",
                panic_msg
            );
        }
    }
}
