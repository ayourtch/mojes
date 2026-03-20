// Tests for or-patterns (A | B) in match expressions
use mojes_mojo::*;
use syn::{parse_quote, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_or_pattern_integers() {
    // Test or-pattern with integers: 1 | 2 | 3 => "small"
    let block: Block = parse_quote! {
        {
            let x = 2;
            match x {
                1 | 2 | 3 => "small",
                4 | 5 | 6 => "medium",
                _ => "large",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (int or-pattern): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string().unwrap(), "small");
}

#[test]
fn test_or_pattern_integers_second_arm() {
    // Test that the second or-pattern arm matches correctly
    let block: Block = parse_quote! {
        {
            let x = 5;
            match x {
                1 | 2 | 3 => "small",
                4 | 5 | 6 => "medium",
                _ => "large",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (int or-pattern second arm): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string().unwrap(), "medium");
}

#[test]
fn test_or_pattern_integers_wildcard() {
    // Test that wildcard catches values not in any or-pattern
    let block: Block = parse_quote! {
        {
            let x = 10;
            match x {
                1 | 2 | 3 => "small",
                4 | 5 | 6 => "medium",
                _ => "large",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (int or-pattern wildcard): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string().unwrap(), "large");
}

#[test]
fn test_or_pattern_strings() {
    // Test or-pattern with string literals
    let block: Block = parse_quote! {
        {
            let color = "blue";
            match color {
                "red" | "blue" | "green" => "primary",
                _ => "other",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (string or-pattern): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string().unwrap(), "primary");
}

#[test]
fn test_or_pattern_strings_no_match() {
    // Test or-pattern with string that falls through to wildcard
    let block: Block = parse_quote! {
        {
            let color = "purple";
            match color {
                "red" | "blue" | "green" => "primary",
                _ => "other",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (string or-pattern no match): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string().unwrap(), "other");
}

#[test]
fn test_or_pattern_with_some_none() {
    // Test match with Some/None (not or-patterns, but ensuring they still work alongside)
    let block: Block = parse_quote! {
        {
            let opt = Some(42);
            match opt {
                Some(val) => val,
                None => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS (Some/None pattern): {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
}
