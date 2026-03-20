// Tests targeting specific edge cases in the transpiler:
// - Scope stack empty (global scope variable declaration)
// - If-let expression used directly (Expr::Let)
// - Block expression in NoReturn context
// - Ok()/Err() function calls
// - Self::new() call
// - Complex cast expressions
// - format_rust_type edge cases
// - ast_to_code_verbose
// - Template literal edge cases
// - Format fallback paths
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_global_scope_variable() {
    // Tests declare_variable when scope_stack is empty (lines 127-132)
    // This requires popping all scopes
    let mut state = TranspilerState::new();
    state.exit_scope(); // Remove the default scope
    let name = state.declare_variable("x".to_string(), "x".to_string(), true);
    assert_eq!(name, "x");
}

#[test]
fn test_ok_function_call() {
    // Tests Ok(value) → { ok: value } (lines 2856-2868)
    let expr: Expr = parse_quote! {
        Ok(42)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS Ok: {}", &js);
    assert!(js.contains("ok") && js.contains("42"));
}

#[test]
fn test_err_function_call() {
    // Tests Err(value) → { error: value } (lines 2870-2891)
    let expr: Expr = parse_quote! {
        Err("failed")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS Err: {}", &js);
    assert!(js.contains("error") && js.contains("failed"));
}

#[test]
fn test_hashmap_new_call() {
    // Tests HashMap::new() → {} (lines 2795)
    let expr: Expr = parse_quote! {
        HashMap::new()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS HashMap::new: {}", &js);
    // Should generate empty object
}

#[test]
fn test_vec_new_call() {
    // Tests Vec::new() → [] (lines 2793)
    let expr: Expr = parse_quote! {
        Vec::new()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS Vec::new: {}", &js);
    assert!(js.contains("[]") || js.contains("Array"));
}

#[test]
fn test_self_new_call_in_struct_context() {
    // Tests Self::new() when struct name is set (lines 2798-2805)
    let mut state = TranspilerState::new();
    state.set_current_struct_name(Some("MyStruct".to_string()));
    let expr: Expr = parse_quote! {
        Self::new()
    };
    let result = rust_expr_to_js_with_state(&expr, &mut state);
    let js_expr = result.unwrap();
    let module_items = vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
        swc_ecma_ast::ExprStmt {
            span: swc_common::DUMMY_SP,
            expr: Box::new(js_expr),
        },
    ))];
    let js = ast_to_code(&module_items).unwrap();
    println!("JS Self::new: {}", &js);
}

#[test]
fn test_complex_function_call_callee() {
    // Tests complex function expression call (line 2906-2907)
    let expr: Expr = parse_quote! {
        (get_fn())(42)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS complex call: {}", &js);
    assert!(js.contains("42"));
}

#[test]
fn test_cast_to_unknown_type() {
    // Tests cast to non-primitive type → "Object" (line 1910)
    let expr: Expr = parse_quote! {
        x as SomeCustomType
    };
    let js = rust_expr_to_js(&expr);
    println!("JS cast custom: {}", &js);
    // Should use Object() or similar
}

#[test]
fn test_if_expression_without_else() {
    // Tests if as expression without else — triggers the else { vec![] } path (line 1079)
    let expr: Expr = parse_quote! {
        if x > 0 { 1 }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS if no else: {}", &js);
    assert!(js.contains("if") && js.contains("x"));
}

#[test]
fn test_block_expression_noreturn() {
    // Tests block expression in NoReturn context (line 1682-1683)
    let block: Block = parse_quote! {
        {
            let result = {
                let x = 5;
                x + 1
            };
            result;
        }
    };
    let mut state = TranspilerState::new();
    let stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &block, &mut state).unwrap();
    let module_items: Vec<swc_ecma_ast::ModuleItem> = stmts
        .into_iter()
        .map(|stmt| swc_ecma_ast::ModuleItem::Stmt(stmt))
        .collect();
    let js = ast_to_code(&module_items).unwrap();
    println!("JS block noreturn: {}", &js);
}

#[test]
fn test_format_rust_type_tuple() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("(i32, String)").unwrap());
    println!("Tuple type: {}", &result);
    assert_eq!(result, "Array");
}

#[test]
fn test_format_rust_type_array() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("[i32; 5]").unwrap());
    println!("Array type: {}", &result);
    assert_eq!(result, "Array");
}

#[test]
fn test_format_rust_type_unknown_generic() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("SomeStruct").unwrap());
    println!("Unknown type: {}", &result);
    assert_eq!(result, "object");
}

#[test]
fn test_format_rust_type_result() {
    let result = format_rust_type(&syn::parse_str::<syn::Type>("Result<i32, String>").unwrap());
    println!("Result type: {}", &result);
    // Result is handled specially
}

#[test]
fn test_ast_to_code_verbose() {
    use swc_common::DUMMY_SP;
    use swc_ecma_ast as js;

    let module_items = vec![
        js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(js::Expr::Lit(js::Lit::Num(js::Number {
                span: DUMMY_SP,
                value: 42.0,
                raw: None,
            }))),
        })),
    ];

    let code = ast_to_code_verbose(&module_items).unwrap();
    println!("Code verbose: {}", &code);
    assert!(code.contains("42"));
}

#[test]
fn test_format_macro_non_string_first_arg() {
    // Tests format! with non-string first arg — triggers fallback (lines 3234-3253)
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!(x));
        p.push(parse_quote!(y));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS format non-string: {}", &js);
    // Should concatenate with +
}

#[test]
fn test_format_macro_single_non_string_arg() {
    // Tests format! with single non-string arg — hits args_vec.len() == 1 (line 3241)
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!(x));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS format single: {}", &js);
    assert!(js.contains("x"));
}

#[test]
fn test_format_macro_empty_args() {
    // Tests format! with empty args — hits lines 3176-3186
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = syn::punctuated::Punctuated::new();
    let js = handle_format_macro(&args);
    println!("JS format empty: {}", &js);
    assert!(js.contains("`") || js.is_empty() || js.contains("\"\""));
}

#[test]
fn test_remove_method_transpilation() {
    // Tests remove() IIFE pattern (lines 2187-2333)
    let expr: Expr = parse_quote! {
        items.remove(0)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS remove: {}", &js);
    assert!(js.contains("splice") || js.contains("delete"));
}

#[test]
fn test_insert_method_transpilation() {
    // Tests insert() IIFE pattern (lines 2336-2427)
    let expr: Expr = parse_quote! {
        items.insert(0, "value")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS insert: {}", &js);
    assert!(js.contains("splice") || js.contains("obj"));
}

#[test]
fn test_regular_function_call() {
    // Tests regular function call path (line 2894-2897)
    let expr: Expr = parse_quote! {
        my_function(1, 2, 3)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS regular call: {}", &js);
    assert!(js.contains("my_function") || js.contains("myFunction"));
}

#[test]
fn test_method_with_then() {
    // Tests promise .then() method
    let expr: Expr = parse_quote! {
        promise.then(|result| result)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS then: {}", &js);
    assert!(js.contains("then"));
}

#[test]
fn test_method_with_catch() {
    // Tests promise .catch() method
    let expr: Expr = parse_quote! {
        promise.catch(|err| err)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS catch: {}", &js);
    assert!(js.contains("catch"));
}

#[test]
fn test_multi_segment_path_call() {
    // Tests path with multiple segments like Foo::bar() (line 2760+)
    let expr: Expr = parse_quote! {
        console::log("hello")
    };
    let js = rust_expr_to_js(&expr);
    println!("JS multi segment: {}", &js);
}

#[test]
fn test_string_with_tab_escape() {
    // Tests string with tab escaping
    let expr: Expr = parse_quote! {
        "hello\tworld"
    };
    let js = rust_expr_to_js(&expr);
    println!("JS tab: {}", &js);
    assert!(js.contains("hello") && js.contains("world"));
}
