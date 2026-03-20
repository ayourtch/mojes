// Tests for the retval (return value) paths in block processing:
// let x = if/match/block — these go through rust_block_to_js_with_retval
// and convert_if_to_stmt_with_retval
use mojes_mojo::*;
use syn::{parse_quote, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_let_if_retval_with_else_if() {
    // Triggers convert_if_to_stmt_with_retval with else-if chain (lines 676-683)
    let block: Block = parse_quote! {
        {
            let x = 7;
            let result = if x > 10 {
                "big"
            } else if x > 5 {
                "medium"
            } else {
                "small"
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval else-if: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "medium");
}

#[test]
fn test_let_if_retval_single_expr_else() {
    // Triggers the _ branch in convert_if_to_stmt_with_retval (lines 685-702)
    // with retval_var Some
    let block: Block = parse_quote! {
        {
            let x = 3;
            let val = if x > 5 { x * 2 } else { x + 1 };
            val
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval single else: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);
}

#[test]
fn test_let_if_let_some_retval() {
    // Triggers if-let Some path via retval (convert_if_to_stmt_with_retval calls handle_if_let_as_stmt)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = Some(42);
            let val = if let Some(x) = opt {
                x * 2
            } else {
                0
            };
            val
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval if-let: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 84.0);
}

#[test]
fn test_let_if_let_none_retval() {
    // Triggers generic if-let with retval (lines 924-939 via convert_generic_if_let_to_stmt)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = None;
            let val = if let None = opt {
                99
            } else {
                0
            };
            val
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval if-let none: {}", &js);
    assert!(js.contains("99"));
}

#[test]
fn test_retval_block_with_for_loop() {
    // Tests for-loop in retval block path (line 1295-1298)
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            let result = {
                let mut sum = 0;
                for item in items {
                    sum = sum + item;
                }
                sum
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval for: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_retval_block_with_while() {
    // Tests while in retval block path
    let block: Block = parse_quote! {
        {
            let result = {
                let mut i = 0;
                while i < 5 {
                    i = i + 1;
                }
                i
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval while: {}", &js);
    assert!(js.contains("while"));
}

#[test]
fn test_retval_block_with_return() {
    // Tests return in retval block path (line 1292)
    let block: Block = parse_quote! {
        {
            let result = {
                return 42;
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval return: {}", &js);
    assert!(js.contains("return") && js.contains("42"));
}

#[test]
fn test_retval_block_with_if() {
    // Tests if in retval block path (line 1308-1311)
    let block: Block = parse_quote! {
        {
            let x = 5;
            let result = {
                if x > 3 {
                    x + 1
                } else {
                    x - 1
                }
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval if: {}", &js);
    assert!(js.contains("if"));
}

#[test]
fn test_retval_fn_definition_in_block() {
    // Tests fn in retval block (line 1267-1269)
    let block: Block = parse_quote! {
        {
            let result = {
                fn double(x: i32) -> i32 {
                    x * 2
                }
                double(5)
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval fn: {}", &js);
    assert!(js.contains("function") && js.contains("double"));
}

#[test]
fn test_retval_expr_with_semicolon() {
    // Tests expression with semicolon in retval block (line 1325)
    let block: Block = parse_quote! {
        {
            let result = {
                let x = 5;
                x + 1;
                x + 2
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval semi: {}", &js);
    assert!(js.contains("x"));
}

#[test]
fn test_retval_macro_in_block() {
    // Tests macro statement in retval block
    let block: Block = parse_quote! {
        {
            let result = {
                println!("hello");
                42
            };
            result
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS retval macro: {}", &js);
    assert!(js.contains("console.log") && js.contains("42"));
}

#[test]
fn test_if_let_some_with_else_if_in_stmt() {
    // Triggers the if-let else handling with else-if chain (lines 855-858)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = None;
            let y = 5;
            if let Some(x) = opt {
                x;
            } else if y > 3 {
                y;
            } else {
                0;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-let else-if stmt: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_if_let_some_single_expr_else_in_stmt() {
    // Triggers the single-expression else branch for if-let (lines 860-865)
    let block: Block = parse_quote! {
        {
            let opt: Option<i32> = None;
            if let Some(x) = opt {
                x;
            } else {
                99;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if-let single else: {}", &js);
    assert!(js.contains("99"));
}

#[test]
fn test_generic_if_let_with_else_if() {
    // Tests generic if-let with else-if chain (lines 924-927)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            let y = 5;
            if let None = x {
                0;
            } else if y > 3 {
                y;
            } else {
                99;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS generic if-let else-if: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_generic_if_let_single_expr_else() {
    // Tests generic if-let with single expression else (lines 929-934)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            if let None = x {
                0;
            } else {
                99;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS generic if-let else: {}", &js);
    assert!(js.contains("99"));
}
