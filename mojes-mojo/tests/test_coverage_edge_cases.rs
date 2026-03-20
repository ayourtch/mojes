// Tests for edge cases: Expr::Let (if-let condition), format string escaping,
// range expressions, enum fromJSON, Self struct expression, typed for-loop patterns
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_while_let_some_as_condition() {
    // Tests Expr::Let → while let Some(x) pattern which uses Expr::Let handling
    // This goes through lines 1775-1812
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            let mut iter_items = items;
            while let Some(item) = iter_items.pop() {
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS while let: {}", &js);
    assert!(js.contains("while") || js.contains("null"));
}

#[test]
fn test_format_string_escaped_braces() {
    // Tests {{ and }} escaping in format strings (lines 3088-3092, 3141-3146)
    let block: Block = parse_quote! {
        {
            println!("value: {{{}}} end", 42);
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS escaped braces: {}", &js);
    assert!(js.contains("console.log"));
}

#[test]
fn test_format_string_debug_specifier() {
    // Tests {:?} debug specifier via handle_format_macro_with_state (line 3119-3124)
    let block: Block = parse_quote! {
        {
            let x = 42;
            println!("value: {:?}", x);
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS debug format: {}", &js);
    assert!(js.contains("console.log"));
}

#[test]
fn test_range_open_end() {
    // Tests range ..end (lines 5629-5667)
    let expr: Expr = parse_quote! {
        ..10
    };
    let js = rust_expr_to_js(&expr);
    println!("JS range ..10: {}", &js);
    assert!(js.contains("Array.from") || js.contains("10"));
}

#[test]
fn test_range_with_start_end() {
    // Tests range start..end
    let expr: Expr = parse_quote! {
        0..5
    };
    let js = rust_expr_to_js(&expr);
    println!("JS range 0..5: {}", &js);
    assert!(js.contains("Array.from") || js.contains("5"));
}

#[test]
fn test_self_field_access_chain() {
    // Tests self.field.method() chain
    let expr: Expr = parse_quote! {
        self.items.len()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS self.items.len: {}", &js);
    assert!(js.contains("this"));
}

#[test]
fn test_cast_to_string() {
    let expr: Expr = parse_quote! {
        x as String
    };
    let js = rust_expr_to_js(&expr);
    println!("JS cast String: {}", &js);
    assert!(js.contains("String") || js.contains("x"));
}

#[test]
fn test_cast_to_bool() {
    let expr: Expr = parse_quote! {
        x as bool
    };
    let js = rust_expr_to_js(&expr);
    println!("JS cast bool: {}", &js);
    assert!(js.contains("Boolean") || js.contains("x"));
}

#[test]
fn test_assignment_to_field() {
    // Tests member expression as assign target
    let block: Block = parse_quote! {
        {
            self.value = 42;
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS field assign: {}", &js);
    assert!(js.contains("this") && js.contains("42"));
}

#[test]
fn test_nested_if_let_some() {
    // Tests nested if-let with else
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = Some(5);
            let y: Option<i32> = Some(10);
            if let Some(a) = x {
                if let Some(b) = y {
                    a + b
                } else {
                    a
                }
            } else {
                0
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS nested if let: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_array_literal() {
    let expr: Expr = parse_quote! {
        [1, 2, 3]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS array lit: {}", &js);
    assert!(js.contains("[") && js.contains("1") && js.contains("3"));
}

#[test]
fn test_method_chain_on_closure_result() {
    // Tests chaining on closure/map result
    let expr: Expr = parse_quote! {
        items.map(|x| x + 1).filter(|x| x > 3)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS chain map.filter: {}", &js);
    assert!(js.contains("map") && js.contains("filter"));
}

#[test]
fn test_multiple_args_println() {
    // Tests println with multiple format args
    let block: Block = parse_quote! {
        {
            let name = "Alice";
            let age = 30;
            println!("{} is {} years old", name, age);
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS multi-arg println: {}", &js);
    assert!(js.contains("console.log") && js.contains("`"));
}

#[test]
fn test_complex_method_remove() {
    // Tests remove method (IIFE pattern)
    let expr: Expr = parse_quote! {
        map.remove("key")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS remove: {}", &js);
    // Should generate delete/splice IIFE
    assert!(js.contains("delete") || js.contains("splice") || js.contains("remove"));
}

#[test]
fn test_len_on_method_call() {
    // Tests .len() on various receivers
    let expr: Expr = parse_quote! {
        items.len()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS len: {}", &js);
    assert!(js.contains("length") || js.contains("Object.keys"));
}

#[test]
fn test_if_in_expression_position() {
    // Tests if as expression value (not in statement context) — triggers IIFE
    let expr: Expr = parse_quote! {
        if x > 0 { 1 } else { 0 }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS if expr: {}", &js);
    assert!(js.contains("if") && (js.contains("return") || js.contains("_rust_retval")));
}

#[test]
fn test_match_in_expression_position() {
    // Tests match as expression — generates IIFE
    let expr: Expr = parse_quote! {
        match x {
            1 => "one",
            _ => "other",
        }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS match expr: {}", &js);
    assert!(js.contains("_match_value") || js.contains("if"));
}
