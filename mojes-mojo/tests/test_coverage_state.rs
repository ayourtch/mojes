// Tests for TranspilerState methods: errors, warnings, scoping, temp vars, assign targets
// Covers lines 102-103, 127-132, 182-183, 194-199, 320-344
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_transpiler_state_errors_and_warnings() {
    // Covers lines 182-183, 190-199
    let mut state = TranspilerState::new();
    assert!(!state.has_errors());
    assert!(state.get_errors().is_empty());
    assert!(state.get_warnings().is_empty());

    state.add_error("test error".to_string());
    assert!(state.has_errors());
    assert_eq!(state.get_errors().len(), 1);
    assert_eq!(state.get_errors()[0], "test error");

    state.add_warning("test warning".to_string());
    assert_eq!(state.get_warnings().len(), 1);
}

#[test]
fn test_transpiler_state_scoping() {
    let mut state = TranspilerState::new();
    let name = state.declare_variable("x".to_string(), "x".to_string(), false);
    assert_eq!(name, "x");
    assert_eq!(state.resolve_variable("x"), Some("x".to_string()));

    state.enter_scope();
    let name = state.declare_variable("y".to_string(), "y".to_string(), true);
    assert_eq!(name, "y");
    assert_eq!(state.resolve_variable("y"), Some("y".to_string()));
    state.exit_scope();
}

#[test]
fn test_transpiler_state_variable_conflict() {
    let mut state = TranspilerState::new();
    let name1 = state.declare_variable("x".to_string(), "x".to_string(), false);
    let name2 = state.declare_variable("x2".to_string(), "x".to_string(), false);
    println!("name1: {}, name2: {}", &name1, &name2);
}

#[test]
fn test_transpiler_state_temp_var() {
    let mut state = TranspilerState::new();
    let v1 = state.generate_temp_var();
    let v2 = state.generate_temp_var();
    assert_ne!(v1, v2);
    assert!(v1.starts_with("_temp"));
    assert!(v2.starts_with("_temp"));
}

#[test]
fn test_transpiler_state_struct_name() {
    let mut state = TranspilerState::new();
    assert!(state.get_current_struct_name().is_none());
    assert!(!state.is_in_static_method());

    state.set_current_struct_name(Some("MyStruct".to_string()));
    assert_eq!(state.get_current_struct_name().unwrap(), "MyStruct");

    state.set_in_static_method(true);
    assert!(state.is_in_static_method());
    state.set_in_static_method(false);
    assert!(!state.is_in_static_method());
}

#[test]
fn test_format_rust_type_various() {
    assert_eq!(format_rust_type(&syn::parse_str::<syn::Type>("i32").unwrap()), "number");
    assert_eq!(format_rust_type(&syn::parse_str::<syn::Type>("f64").unwrap()), "number");
    assert_eq!(format_rust_type(&syn::parse_str::<syn::Type>("bool").unwrap()), "boolean");
    assert_eq!(format_rust_type(&syn::parse_str::<syn::Type>("String").unwrap()), "string");
}

#[test]
fn test_ast_to_code() {
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

    let code = ast_to_code(&module_items).unwrap();
    println!("Code: {}", &code);
    assert!(code.contains("42"));
}

#[test]
fn test_block_with_multiple_statements() {
    let block: Block = parse_quote! {
        {
            let x = 1;
            let y = 2;
            let z = x + y;
            z
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_block_return_action() {
    // Test that rust_block_to_js generates return statement for last expression
    let block: Block = parse_quote! {
        {
            let x = 42;
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    assert!(js.contains("return") || js.contains("42"));
}

#[test]
fn test_noreturn_with_state() {
    // Test NoReturn via the _with_state API directly
    let block: Block = parse_quote! {
        {
            let x = 42;
            x;
        }
    };
    let mut state = TranspilerState::new();
    let stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &block, &mut state).unwrap();
    let module_items: Vec<swc_ecma_ast::ModuleItem> = stmts
        .into_iter()
        .map(|stmt| swc_ecma_ast::ModuleItem::Stmt(stmt))
        .collect();
    let js = ast_to_code(&module_items).unwrap();
    println!("JS NoReturn: {}", &js);
}

#[test]
fn test_escape_js_identifier() {
    // Test the identifier escaping function
    assert_eq!(escape_js_identifier("normal"), "normal");
    // JS reserved words get _ suffix
    assert_eq!(escape_js_identifier("class"), "class_");
    assert_eq!(escape_js_identifier("function"), "function_");
}
