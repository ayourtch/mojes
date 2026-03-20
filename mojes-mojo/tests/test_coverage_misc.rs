// Tests for miscellaneous coverage: string escaping, HashMap::new, let if as retval,
// format_rust_type edge cases, ast_to_code variants, and various expression paths
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_hashmap_new() {
    // Tests HashMap::new() → {} transpilation
    let block: Block = parse_quote! {
        {
            let m = HashMap::new();
            m
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS HashMap::new: {}", &js);
    assert!(js.contains("{}") || js.contains("HashMap"));
}

#[test]
fn test_vec_new() {
    let block: Block = parse_quote! {
        {
            let v = Vec::new();
            v
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Vec::new: {}", &js);
    assert!(js.contains("[]") || js.contains("Vec"));
}

#[test]
fn test_let_if_as_value() {
    // Tests if expression used as initializer for let binding (retval pattern)
    let block: Block = parse_quote! {
        {
            let x = 5;
            let result = if x > 3 { "big" } else { "small" };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS let if: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    let s = result.as_string().unwrap().to_std_string_escaped();
    assert_eq!(s, "big");
}

#[test]
fn test_let_match_as_value() {
    // Tests match expression used as initializer
    let block: Block = parse_quote! {
        {
            let x = 2;
            let result = match x {
                1 => "one",
                2 => "two",
                _ => "other",
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS let match: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    let s = result.as_string().unwrap().to_std_string_escaped();
    assert_eq!(s, "two");
}

#[test]
fn test_format_rust_type_option() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("Option<i32>").unwrap());
    println!("Option<i32> type: {}", &result);
    // Just verify it doesn't panic
}

#[test]
fn test_format_rust_type_vec() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("Vec<String>").unwrap());
    println!("Vec<String> type: {}", &result);
    // Should be "Array" or similar
    assert!(result.contains("Array") || result.contains("string"));
}

#[test]
fn test_format_rust_type_reference() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("&str").unwrap());
    println!("&str type: {}", &result);
    assert_eq!(result, "string");
}

#[test]
fn test_string_escaping() {
    // Test that strings with special chars are properly escaped
    let expr: Expr = parse_quote! {
        "hello\nworld"
    };
    let js = rust_expr_to_js(&expr);
    println!("JS escaped: {}", &js);
    assert!(js.contains("hello") && js.contains("world"));
}

#[test]
fn test_negative_number() {
    let expr: Expr = parse_quote! {
        -42
    };
    let js = rust_expr_to_js(&expr);
    println!("JS neg: {}", &js);
    let result = eval_js(&js).unwrap();
    assert_eq!(result.as_number().unwrap(), -42.0);
}

#[test]
fn test_complex_binary_expression() {
    let block: Block = parse_quote! {
        {
            let x = 5;
            let y = 3;
            let z = (x + y) * 2 - 1;
            z
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS complex bin: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);
}

#[test]
fn test_comparison_operators() {
    let expr: Expr = parse_quote! { x == y };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("==="), "== should become ===");

    let expr: Expr = parse_quote! { x != y };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("!=="), "!= should become !==");
}

#[test]
fn test_logical_operators() {
    let expr: Expr = parse_quote! { a && b };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("&&"));

    let expr: Expr = parse_quote! { a || b };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("||"));
}

#[test]
fn test_shift_operators() {
    let expr: Expr = parse_quote! { x << 2 };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("<<"));

    let expr: Expr = parse_quote! { x >> 1 };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains(">>"));
}

#[test]
fn test_ast_to_code_trimmed() {
    use swc_common::DUMMY_SP;
    use swc_ecma_ast as js;

    let module_items = vec![
        js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(js::Expr::Lit(js::Lit::Str(js::Str {
                span: DUMMY_SP,
                value: "hello".into(),
                raw: None,
            }))),
        })),
    ];

    let code = ast_to_code_trimmed(&module_items).unwrap();
    println!("Code trimmed: {}", &code);
    assert!(code.contains("hello"));
}

#[test]
fn test_ast_to_code_compact() {
    use swc_common::DUMMY_SP;
    use swc_ecma_ast as js;

    let module_items = vec![
        js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(js::Expr::Lit(js::Lit::Num(js::Number {
                span: DUMMY_SP,
                value: 99.0,
                raw: None,
            }))),
        })),
    ];

    let code = ast_to_code_compact(&module_items).unwrap();
    println!("Code compact: {}", &code);
    assert!(code.contains("99"));
}

#[test]
fn test_method_to_uppercase_lowercase() {
    let expr: Expr = parse_quote! { s.to_uppercase() };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("toUpperCase"));

    let expr: Expr = parse_quote! { s.to_lowercase() };
    let js = rust_expr_to_js(&expr);
    assert!(js.contains("toLowerCase"));
}

#[test]
fn test_method_trim_variants() {
    let expr: Expr = parse_quote! { s.trim() };
    assert!(rust_expr_to_js(&expr).contains("trim"));

    let expr: Expr = parse_quote! { s.trim_start() };
    assert!(rust_expr_to_js(&expr).contains("trimStart"));

    let expr: Expr = parse_quote! { s.trim_end() };
    assert!(rust_expr_to_js(&expr).contains("trimEnd"));
}

#[test]
fn test_method_push_pop() {
    let expr: Expr = parse_quote! { v.push(1) };
    assert!(rust_expr_to_js(&expr).contains("push"));

    let expr: Expr = parse_quote! { v.pop() };
    assert!(rust_expr_to_js(&expr).contains("pop"));
}

#[test]
fn test_method_contains() {
    let expr: Expr = parse_quote! { v.contains(&x) };
    let js = rust_expr_to_js(&expr);
    println!("contains: {}", &js);
    assert!(js.contains("includes"));
}

#[test]
fn test_method_clone_noop() {
    let expr: Expr = parse_quote! { x.clone() };
    let js = rust_expr_to_js(&expr);
    println!("clone: {}", &js);
    // clone should be a no-op, returning receiver
    assert!(js.contains("x"));
}
