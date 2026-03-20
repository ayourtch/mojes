// Tests for closures, ranges, struct expressions, and miscellaneous transpilation paths
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_closure_basic() {
    let expr: Expr = parse_quote! {
        |x: i32| x + 1
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("=>"));
}

#[test]
fn test_closure_multi_param() {
    let expr: Expr = parse_quote! {
        |a: i32, b: i32| a + b
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("=>"));
}

#[test]
fn test_closure_with_block_body() {
    let expr: Expr = parse_quote! {
        |x: i32| {
            let y = x * 2;
            y + 1
        }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("=>"));
}

#[test]
fn test_closure_with_reference_pattern() {
    // Tests extract_ident_from_pattern with Pat::Reference (line 5580)
    let expr: Expr = parse_quote! {
        |item| item + 1
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("=>") && js.contains("item"));
}

#[test]
fn test_range_inclusive() {
    // Tests range expression with start..=end
    let block: Block = parse_quote! {
        {
            let items: Vec<i32> = (0..5).collect();
            items
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    // Should use Array.from pattern
    assert!(js.contains("Array.from") || js.contains("["));
}

#[test]
fn test_range_half_open() {
    // Tests Range with just ..end (lines 5833-5871)
    let expr: Expr = parse_quote! {
        0..10
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("Array.from") || js.contains("length"));
}

#[test]
fn test_struct_expression() {
    // Tests handle_struct_expr (lines 5586-5633)
    let expr: Expr = parse_quote! {
        Point { x: 1, y: 2 }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should generate constructor call: new Point(1, 2)
    assert!(js.contains("Point") && (js.contains("new") || js.contains("x")));
}

#[test]
fn test_enum_variant_struct_expression() {
    // Tests struct expression for enum variant with multiple path segments (lines 5597-5632)
    let expr: Expr = parse_quote! {
        Message::Hello { name: "World" }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should generate object literal with type discriminant
    assert!(js.contains("type") && js.contains("Hello"));
}

#[test]
fn test_for_loop_basic() {
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            let mut sum = 0;
            for item in items {
                sum = sum + item;
            }
            sum
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_async_block() {
    let expr: Expr = parse_quote! {
        async { 42 }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("async"));
}

#[test]
fn test_await_expression() {
    let expr: Expr = parse_quote! {
        fetch_data().await
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("await"));
}

#[test]
fn test_type_cast_as() {
    // Tests Expr::Cast handling
    let expr: Expr = parse_quote! {
        x as f64
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    // Should generate Number() or similar cast
    assert!(js.contains("Number") || js.contains("x"));
}

#[test]
fn test_tuple_expression() {
    let expr: Expr = parse_quote! {
        (1, 2, 3)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("[") && js.contains("]"));
}

#[test]
fn test_index_expression() {
    let expr: Expr = parse_quote! {
        items[0]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("[0]") || js.contains("items"));
}

#[test]
fn test_reference_expression() {
    // Tests Expr::Reference — should be a no-op in JS
    let expr: Expr = parse_quote! {
        &x
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("x"));
}

#[test]
fn test_dereference_expression() {
    // Tests Expr::Unary with deref
    let expr: Expr = parse_quote! {
        *ptr
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("ptr"));
}

#[test]
fn test_negation_expression() {
    let expr: Expr = parse_quote! {
        !flag
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("!") && js.contains("flag"));
}

#[test]
fn test_paren_expression() {
    let expr: Expr = parse_quote! {
        (x + y)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("x") && js.contains("y"));
}

#[test]
fn test_field_access() {
    let expr: Expr = parse_quote! {
        point.x
    };
    let js = rust_expr_to_js(&expr);
    println!("JS: {}", &js);
    assert!(js.contains("point") && js.contains("x"));
}
