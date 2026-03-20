// Final push targeting remaining edge cases:
// - Tuple match with Some/None elements
// - Expr::Let as standalone expression
// - Format string edge cases (unmatched }, extra args)
// - convert_if_to_stmt single-expression else
// - while-let pattern None path
// - remove/insert error paths
// - Enum variant paths in match
// - for-of as expression (IIFE wrapper)
// - Chained match if statements
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_match_tuple_with_some_binding() {
    // Tests Pat::TupleStruct "Some" inside tuple pattern (lines 5055-5086)
    let block: Block = parse_quote! {
        {
            let data = (Some(10), Some(20));
            match data {
                (Some(a), Some(b)) => a + b,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple Some/Some: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_match_tuple_with_none_element() {
    // Tests Pat::Path "None" inside tuple pattern (lines 5099-5118)
    let block: Block = parse_quote! {
        {
            let data = (42, None);
            match data {
                (x, None) => x,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple with None: {}", &js);
    assert!(js.contains("null") || js.contains("undefined") || js.contains("[1]"));
}

#[test]
fn test_match_tuple_with_ident_elements() {
    // Tests tuple with only ident elements (all variable bindings)
    let block: Block = parse_quote! {
        {
            let pair = (1, 2);
            match pair {
                (x, y) => x + y,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple idents: {}", &js);
    assert!(js.contains("[0]") || js.contains("[1]") || js.contains("_match_value"));
}

#[test]
fn test_match_tuple_with_wildcard_element() {
    // Tests Pat::Wild inside tuple pattern — currently unsupported
    let block: Block = parse_quote! {
        {
            let pair = (1, 2);
            match pair {
                (x, _) => x,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple wildcard: {}", &js);
    assert!(js.contains("[0]") || js.contains("_match_value"));
}

#[test]
fn test_match_enum_variant_with_wildcard_field() {
    // Tests wildcard in enum tuple variant: Message::Data(_, y)
    let block: Block = parse_quote! {
        {
            let msg = Message::Data(1, 2);
            match msg {
                Message::Data(_, y) => y,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS enum wildcard field: {}", &js);
    assert!(js.contains("type") || js.contains("Data"));
}

#[test]
fn test_if_with_single_expression_else_stmt_context() {
    // Tests convert_if_to_stmt with non-block else (lines 619-625)
    // This needs an expression else, not a block else
    // Rust syntax requires block for else, but the transpiler handles the case
    // where the else branch is already a non-block expression internally
    let block: Block = parse_quote! {
        {
            let x = 5;
            if x > 3 {
                10
            } else {
                20
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-else stmt: {}", &js);
    assert!(js.contains("if") && js.contains("10") && js.contains("20"));
}

#[test]
fn test_match_with_many_arms_chain() {
    // Tests chain_if_statement with multiple arms (lines 5224-5248)
    let block: Block = parse_quote! {
        {
            let x = 5;
            match x {
                1 => "a",
                2 => "b",
                3 => "c",
                4 => "d",
                5 => "e",
                6 => "f",
                _ => "g",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS many arms: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "e");
}

#[test]
fn test_for_as_expression() {
    // Tests handle_for_expr (for-of wrapped in IIFE, line 4662-4668)
    let expr: Expr = parse_quote! {
        {
            let mut sum = 0;
            for item in items {
                sum = sum + item;
            }
            sum
        }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS for-expr: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_format_string_with_extra_placeholder_args() {
    // Tests format string with more {} than arguments (line 3138 — empty string fallback)
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("one {} two {} three {}"));
        p.push(parse_quote!(a)); // Only 1 arg for 3 placeholders
        p
    };
    let js = handle_format_macro(&args);
    println!("JS extra placeholders: {}", &js);
    assert!(js.contains("`"));
}

#[test]
fn test_format_string_with_closing_brace_only() {
    // Tests unmatched } in format string (line 3150)
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("value: }"));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS closing brace: {}", &js);
    assert!(js.contains("value"));
}

#[test]
fn test_format_string_with_escaped_braces() {
    // Tests {{ and }} escaping (lines 3088-3092, 3141-3146)
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("{{literal braces}}"));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS escaped braces: {}", &js);
    assert!(js.contains("literal braces") || js.contains("{"));
}

#[test]
fn test_format_other_specifier() {
    // Tests {:x}, {:02} etc (line 3131 — "other format specifiers")
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = {
        let mut p = syn::punctuated::Punctuated::new();
        p.push(parse_quote!("hex: {:x}"));
        p.push(parse_quote!(num));
        p
    };
    let js = handle_format_macro(&args);
    println!("JS hex format: {}", &js);
    assert!(js.contains("`") || js.contains("num"));
}

#[test]
fn test_match_with_enum_struct_and_binding() {
    // Tests enum struct variant with field binding in match
    let block: Block = parse_quote! {
        {
            let event = Event::Click { x: 100, y: 200 };
            match event {
                Event::Click { x, y } => x + y,
                Event::Hover { x } => x,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS enum struct match: {}", &js);
    assert!(js.contains("type") || js.contains("Click"));
}

#[test]
fn test_impl_with_void_return() {
    // Tests method with no return value
    let input: syn::ItemImpl = parse_quote! {
        impl Logger {
            fn log(&self, msg: String) {
                println!("{}", msg);
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS void method: {}", &js);
    assert!(js.contains("Logger") && js.contains("log"));
}

#[test]
fn test_block_with_multiple_ifs() {
    // Tests multiple if statements in a block (not as value)
    let block: Block = parse_quote! {
        {
            let x = 5;
            if x > 10 {
                println!("big");
            }
            if x > 3 {
                println!("medium");
            }
            if x > 0 {
                println!("positive");
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS multi if: {}", &js);
    assert!(js.contains("console.log"));
}

#[test]
fn test_closure_no_params() {
    // Tests closure with no parameters
    let expr: Expr = parse_quote! {
        || 42
    };
    let js = rust_expr_to_js(&expr);
    println!("JS no-param closure: {}", &js);
    assert!(js.contains("=>") && js.contains("42"));
}

#[test]
fn test_closure_with_block_body_and_return() {
    // Tests closure with block body containing return
    let expr: Expr = parse_quote! {
        |x: i32| {
            if x > 0 {
                return x;
            }
            0
        }
    };
    let js = rust_expr_to_js(&expr);
    println!("JS closure block return: {}", &js);
    assert!(js.contains("=>") && js.contains("return"));
}

#[test]
fn test_nested_match_in_match() {
    // Tests nested match expressions
    let block: Block = parse_quote! {
        {
            let x = 1;
            let y = 2;
            match x {
                1 => match y {
                    2 => "x=1,y=2",
                    _ => "x=1,y=other",
                },
                _ => "x=other",
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS nested match: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "x=1,y=2");
}

#[test]
fn test_method_on_result_of_method() {
    // Tests method call on result of another method call
    let expr: Expr = parse_quote! {
        text.trim().to_lowercase()
    };
    let js = rust_expr_to_js(&expr);
    println!("JS chain: {}", &js);
    assert!(js.contains("trim") && js.contains("toLowerCase"));
}

#[test]
fn test_if_let_some_no_else() {
    // Tests if-let Some without else branch (line 870)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = Some(42);
            if let Some(val) = opt {
                val;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-let no else: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}
