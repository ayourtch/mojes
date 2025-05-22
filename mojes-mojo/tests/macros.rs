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

#[test]
fn test_unsupported_macro() {
    let expr: Expr = parse_quote!(unsupported_macro!("test"));
    let js_code = rust_expr_to_js(&expr);

    // Should generate a comment about unsupported macro
    assert!(js_code.contains("Unsupported macro"));
    assert!(js_code.contains("unsupported_macro"));
}
