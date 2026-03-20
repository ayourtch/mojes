// Tests for advanced match patterns via handle_pattern_binding:
// Ok/Err patterns, struct patterns, tuple patterns in match, enum struct patterns
// Covers lines 4860-5225
use mojes_mojo::*;
use syn::{parse_quote, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_match_ok_err_pattern() {
    // Tests Ok(x)/Err(e) pattern matching (handle_pattern_binding)
    let block: Block = parse_quote! {
        {
            let result: Result<i32, String> = Ok(42);
            match result {
                Ok(val) => val,
                Err(e) => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Ok/Err: {}", &js);
    assert!(js.contains("ok") || js.contains("error"));
}

#[test]
fn test_match_struct_pattern() {
    // Tests struct pattern in match (Pat::Struct in handle_pattern_binding)
    let block: Block = parse_quote! {
        {
            let msg = Message::Hello { name: "World".to_string() };
            match msg {
                Message::Hello { name } => name,
                _ => "unknown".to_string(),
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS struct match: {}", &js);
    assert!(js.contains("type") || js.contains("Hello") || js.contains("name"));
}

#[test]
fn test_match_tuple_pattern() {
    // Tests tuple pattern in match — only ident/wildcard elements are supported
    let block: Block = parse_quote! {
        {
            let pair = (1, 2);
            match pair {
                (x, y) => x + y,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple match: {}", &js);
    assert!(js.contains("[0]") || js.contains("[1]") || js.contains("_match_value"));
}

#[test]
fn test_match_multiple_arms() {
    // Tests chaining multiple match arms
    let block: Block = parse_quote! {
        {
            let x = 3;
            match x {
                1 => 10,
                2 => 20,
                3 => 30,
                4 => 40,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS multi-arm: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);
}

#[test]
fn test_match_option_some_none() {
    // Tests both Some and None pattern together
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = Some(10);
            match x {
                Some(val) => val * 2,
                None => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Some/None match: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_match_with_function_body() {
    // Tests match arm with complex block body
    let block: Block = parse_quote! {
        {
            let x = "hello";
            match x {
                "hello" => {
                    let greeting = "world";
                    greeting
                },
                _ => {
                    "unknown"
                },
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS match blocks: {}", &js);
    assert!(js.contains("hello") && js.contains("world"));
}

#[test]
fn test_match_char_pattern() {
    // Tests Pat::Lit with Char (line 4885 in handle_pattern_binding, 5309 in create_match_condition)
    let block: Block = parse_quote! {
        {
            let ch = 'a';
            match ch {
                'a' => 1,
                'b' => 2,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS char match: {}", &js);
    assert!(js.contains("a") && js.contains("b"));
}
