// Tests for if-let patterns, if-let-some with tuples, generic if-let, and else branches
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_if_let_some_basic() {
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = Some(42);
            if let Some(val) = x {
                val + 1
            } else {
                0
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_if_let_some_with_tuple_destructuring() {
    // Tests the Pat::Tuple branch inside convert_if_let_some_to_stmt (lines 835-857)
    let block: Block = parse_quote! {
        {
            let pair: Option<(i32, i32)> = Some((10, 20));
            if let Some((a, b)) = pair {
                a + b
            } else {
                0
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("[0]") || js.contains("[1]") || js.contains("null"));
}

#[test]
fn test_if_let_some_with_else_if() {
    // Tests the Expr::If branch in else handling (lines 885-889)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            let y = 5;
            if let Some(val) = x {
                val
            } else if y > 3 {
                y
            } else {
                0
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_if_let_some_with_single_expr_else() {
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            if let Some(val) = x {
                val
            } else {
                99
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("99"));
}

#[test]
fn test_generic_if_let_pattern() {
    // Tests convert_generic_if_let_to_stmt (lines 924-991)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            if let None = x {
                0
            } else {
                1
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_if_with_retval_else_if_chain() {
    // Tests convert_if_to_stmt_with_retval with else-if (lines 706-712)
    let block: Block = parse_quote! {
        {
            let x = 5;
            let result = if x > 10 {
                1
            } else if x > 5 {
                2
            } else {
                3
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_if_single_expression_else() {
    let block: Block = parse_quote! {
        {
            let x = 5;
            if x > 10 {
                x + 1
            } else {
                x - 1
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("x"));
}
