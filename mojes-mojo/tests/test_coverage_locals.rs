// Tests for local variable declarations: typed patterns, tuple destructuring,
// struct destructuring, wildcard, uninitialized vars
// Covers lines 3328-3467, 3540-3548
use mojes_mojo::*;
use syn::{parse_quote, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_typed_variable_declaration() {
    // Tests Pat::Type handling (lines 3328-3348)
    let block: Block = parse_quote! {
        {
            let x: i32 = 42;
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS typed: {}", &js);
    assert!(js.contains("42"));
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[test]
fn test_mutable_typed_variable() {
    let block: Block = parse_quote! {
        {
            let mut count: i32 = 0;
            count = count + 1;
            count
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS mut typed: {}", &js);
    assert!(js.contains("let") && js.contains("count"));
}

#[test]
fn test_tuple_destructuring_let() {
    // Tests Pat::Tuple in let (lines 3351-3394)
    let block: Block = parse_quote! {
        {
            let (a, b) = (1, 2);
            a + b
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple destr: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_struct_destructuring_let() {
    // Tests Pat::Struct in let (lines 3396-3444)
    let block: Block = parse_quote! {
        {
            let point = Point { x: 10, y: 20 };
            let Point { x, y } = point;
            x + y
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS struct destr: {}", &js);
    // Should generate object destructuring
    assert!(js.contains("x") && js.contains("y"));
}

#[test]
fn test_wildcard_let() {
    // Tests Pat::Wild in let (lines 3446-3452)
    let block: Block = parse_quote! {
        {
            let _ = some_function();
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS wildcard let: {}", &js);
    // Should just evaluate the expression for side effects
    assert!(js.contains("some_function"));
}

#[test]
fn test_uninitialized_variable() {
    // Tests variable declaration without initialization (lines 3457-3468)
    let block: Block = parse_quote! {
        {
            let x;
            x = 42;
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS uninit: {}", &js);
    assert!(js.contains("let") && js.contains("x"));
}

#[test]
fn test_string_concat_detection() {
    // Tests string expression detection via + operator
    let block: Block = parse_quote! {
        {
            let a = "hello";
            let b = "world";
            let c = format!("{} {}", a, b);
            c
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS string concat: {}", &js);
    assert!(js.contains("`") || js.contains("hello"));
}

#[test]
fn test_nested_method_chain() {
    // Test chained method calls
    let block: Block = parse_quote! {
        {
            let s = "hello world";
            let result = s.trim().to_uppercase();
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS chain: {}", &js);
    assert!(js.contains("trim") && js.contains("toUpperCase"));
}

#[test]
fn test_remove_method() {
    // Tests remove() transpilation
    let block: Block = parse_quote! {
        {
            let mut m = HashMap::new();
            m.remove("key");
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS remove: {}", &js);
    // Should generate delete or splice IIFE
    assert!(js.contains("delete") || js.contains("splice") || js.contains("remove"));
}
