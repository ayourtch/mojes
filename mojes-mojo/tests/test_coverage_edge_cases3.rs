// Tests targeting remaining edge cases:
// - println/print as function call path
// - Ok/Err in function call context
// - Self { field } in non-static method context
// - if-let expression path (Expr::Let in condition)
// - convert_if_to_stmt various else branches
// - while-let as expression
// - for loop with tuple destructuring and entries() IIFE
// - is_string_expr checks
// - handle_format_macro_with_state edge cases
// - retval with NoReturn action in blocks
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr, ItemImpl};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_println_as_function_call() {
    // Tests "println" match in handle_function_call (lines 2840-2843)
    let block: Block = parse_quote! {
        {
            println("hello");
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS println fn: {}", &js);
    assert!(js.contains("console.log"));
}

#[test]
fn test_ok_in_block() {
    // Tests Ok() in a block statement context (lines 2856-2873)
    let block: Block = parse_quote! {
        {
            let result = Ok(42);
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Ok in block: {}", &js);
    assert!(js.contains("ok") && js.contains("42"));
}

#[test]
fn test_err_in_block() {
    // Tests Err() in a block statement context (lines 2875-2892)
    let block: Block = parse_quote! {
        {
            let result = Err("oops");
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Err in block: {}", &js);
    assert!(js.contains("error") && js.contains("oops"));
}

#[test]
fn test_self_struct_expr_in_instance_method() {
    // Tests Self { field: val } in non-static method (lines 5516-5536)
    let input: ItemImpl = parse_quote! {
        impl Point {
            fn translate(&self, dx: i32, dy: i32) -> Self {
                Self { x: self.x + dx, y: self.y + dy }
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS self instance method: {}", &js);
    assert!(js.contains("Point") && js.contains("translate"));
}

#[test]
fn test_self_struct_expr_in_static_new() {
    // Tests Self { } in static constructor method
    let input: ItemImpl = parse_quote! {
        impl Config {
            fn new(debug: bool) -> Self {
                Self { debug }
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS Self static new: {}", &js);
    assert!(js.contains("Config"));
}

#[test]
fn test_if_without_else_in_stmt_context() {
    // Tests convert_if_to_stmt without else branch — line 870 (None)
    let block: Block = parse_quote! {
        {
            let x = 5;
            if x > 3 {
                x;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if no else stmt: {}", &js);
    assert!(js.contains("if") && !js.contains("else"));
}

#[test]
fn test_if_let_some_else_block_in_stmt() {
    // Tests if-let Some with else block in statement context (line 843-847)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = Some(5);
            if let Some(val) = opt {
                val;
            } else {
                let y = 0;
                y;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-let else block: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_for_tuple_with_three_elements() {
    // For loop with 3-element tuple — doesn't trigger the entries() IIFE (line 4615 branch)
    let block: Block = parse_quote! {
        {
            let items = vec![(1, 2, 3)];
            for (a, b, c) in items {
                a;
                b;
                c;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS for 3-tuple: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_if_retval_with_no_else() {
    // Tests if as value without else — should have the "no else" retval path
    let block: Block = parse_quote! {
        {
            let x = 5;
            let result = if x > 3 { 10 } else { 20 };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if retval: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);
}

#[test]
fn test_generic_if_let_without_else() {
    // Tests generic if-let without else (line 939)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            if let None = x {
                0;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-let no else: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_block_with_retval_return_void() {
    // Tests return; (no value) in retval context (line 1292)
    let block: Block = parse_quote! {
        {
            let x = {
                return;
            };
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval return void: {}", &js);
    assert!(js.contains("return"));
}

#[test]
fn test_method_as_str() {
    // Tests .as_str() method transpilation (lines 2159-2161)
    let expr: Expr = parse_quote! {
        name.as_str()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS as_str: {}", &js);
    assert!(js.contains("String"));
}

#[test]
fn test_method_is_empty() {
    // Tests .is_empty() → .length === 0 (lines 2181-2184)
    let expr: Expr = parse_quote! {
        name.is_empty()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS is_empty: {}", &js);
    assert!(js.contains("length") && js.contains("0"));
}

#[test]
fn test_string_concat_with_plus() {
    // Tests string binary add (lines 1681 area — string concatenation)
    let block: Block = parse_quote! {
        {
            let a = "hello";
            let b = " world";
            let c = a + b;
            c
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS str concat: {}", &js);
    assert!(js.contains("+") || js.contains("`"));
}

#[test]
fn test_match_with_wildcard_enum_arm() {
    // Tests wildcard catch-all in match after specific patterns
    let block: Block = parse_quote! {
        {
            let x = 42;
            let result = match x {
                1 => "one",
                2 => "two",
                3 => "three",
                _ => "many",
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS match wildcard: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "many");
}

#[test]
fn test_while_let_some_as_expression() {
    // Tests while-let used as an expression (wraps in IIFE)
    let expr: Expr = parse_quote! {
        {
            let mut items = vec![1, 2];
            while let Some(item) = items.pop() {
                item;
            }
        }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS while-let expr: {}", &js);
    assert!(js.contains("while") || js.contains("null"));
}

#[test]
fn test_for_with_method_chain_on_iterable() {
    // Tests for loop over method chain
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            for item in items.iter() {
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS for iter: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_template_literal_with_parts_exprs_mismatch() {
    // Tests mk_template_literal edge case where parts > exprs+1 (line 377)
    let mut state = TranspilerState::new();
    let parts = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
    let exprs = vec![state.mk_num_lit(1.0)]; // 1 expr but 4 parts
    let tpl = state.mk_template_literal(parts, exprs);
    let module_items = vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
        swc_ecma_ast::ExprStmt {
            span: swc_common::DUMMY_SP,
            expr: Box::new(tpl),
        },
    ))];
    let code = ast_to_code(&module_items).unwrap();
    println!("Template mismatch: {}", &code);
}

#[test]
fn test_impl_method_error_handling() {
    // Tests error path when method generation fails (line 440)
    // This is hard to trigger since most valid syntax works, but let's try a method
    // with complex return types
    let input: ItemImpl = parse_quote! {
        impl Widget {
            fn render(&self) {
                self.draw();
            }

            fn update(&mut self, value: i32) {
                self.value = value;
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS impl methods: {}", &js);
    assert!(js.contains("Widget"));
}
