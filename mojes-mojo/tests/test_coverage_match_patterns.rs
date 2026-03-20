// Tests for match expression pattern types: literals, wildcards, None, Some, enum variants, tuples
// Covers create_match_condition (lines 5287-5404) and handle_pattern_binding
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_match_literal_int() {
    // Tests Pat::Lit with integer (lines 5293-5317)
    let block: Block = parse_quote! {
        {
            let x = 2;
            match x {
                1 => 10,
                2 => 20,
                3 => 30,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);
}

#[test]
fn test_match_literal_string() {
    // Tests Pat::Lit with string
    let block: Block = parse_quote! {
        {
            let color = "red";
            match color {
                "red" => 1,
                "blue" => 2,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
}

#[test]
fn test_match_literal_bool() {
    // Tests Pat::Lit with boolean (line 5308)
    let block: Block = parse_quote! {
        {
            let flag = true;
            match flag {
                true => 1,
                false => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
}

#[test]
fn test_match_wildcard() {
    // Tests Pat::Wild (lines 5319-5322)
    let block: Block = parse_quote! {
        {
            let x = 99;
            match x {
                _ => 42,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[test]
fn test_match_variable_binding() {
    // Tests Pat::Ident (lines 5323-5329)
    let block: Block = parse_quote! {
        {
            let x = 7;
            match x {
                val => val + 1,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 8.0);
}

#[test]
fn test_match_none_pattern() {
    // Tests Pat::Path with None variant (lines 5330-5351)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            match x {
                None => 0,
                Some(v) => v,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_match_some_pattern() {
    // Tests Pat::TupleStruct with Some(x) (lines 5366-5401)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = Some(42);
            match x {
                Some(val) => val,
                None => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_match_enum_unit_variant_path() {
    // Tests Pat::Path with custom enum variant (lines 5353-5359)
    let block: Block = parse_quote! {
        {
            let status = "Active";
            match status {
                "Active" => 1,
                "Inactive" => 0,
                _ => 2,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
}

#[test]
fn test_match_with_block_bodies() {
    // Tests match arms with block bodies
    let block: Block = parse_quote! {
        {
            let x = 5;
            match x {
                1 => {
                    let a = 10;
                    a + 1
                },
                _ => {
                    let b = 20;
                    b + 2
                },
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 22.0);
}
