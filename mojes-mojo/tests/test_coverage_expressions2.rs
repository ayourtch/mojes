// Tests for additional expression types: try(?), repeat, loop, break/continue, return,
// function definitions in blocks, assign ops, and more
use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_try_operator() {
    // Tests Expr::Try handling (lines 2027-2065)
    let expr: Expr = parse_quote! {
        get_value()?
    };
    let js = rust_expr_to_js(&expr);
    println!("JS try: {}", &js);
    // Should generate IIFE with error check
    assert!(js.contains("error") || js.contains("ok"));
}

#[test]
fn test_repeat_expression() {
    // Tests Expr::Repeat handling (lines 1968-1999)
    let expr: Expr = parse_quote! {
        [0; 10]
    };
    let js = rust_expr_to_js(&expr);
    println!("JS repeat: {}", &js);
    assert!(js.contains("Array.from") && js.contains("length"));
}

#[test]
fn test_loop_expression() {
    // Tests handle_loop_expr (infinite loop)
    let block: Block = parse_quote! {
        {
            let mut x = 0;
            loop {
                x = x + 1;
                if x > 5 {
                    break;
                }
            }
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS loop: {}", &js);
    assert!(js.contains("while") && js.contains("true"));
}

#[test]
fn test_break_with_value() {
    // Tests Expr::Break with value (lines 1879-1883)
    let expr: Expr = parse_quote! {
        break 42
    };
    let js = rust_expr_to_js(&expr);
    println!("JS break: {}", &js);
    assert!(js.contains("42"));
}

#[test]
fn test_break_without_value() {
    // Tests Expr::Break without value — should generate a JS `break` statement
    let block: Block = parse_quote! {
        {
            loop {
                break;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS break: {}", &js);
    assert!(js.contains("break"));
}

#[test]
fn test_continue_in_loop() {
    // Tests that continue generates a proper JS continue statement
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            for item in items {
                if item == 2 {
                    continue;
                }
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS continue: {}", &js);
    assert!(js.contains("continue"));
}

#[test]
fn test_break_in_while_loop_executes() {
    // Verify that break in while loop generates correct, executable JS
    let block: Block = parse_quote! {
        {
            let mut count = 0;
            while true {
                count = count + 1;
                if count >= 3 {
                    break;
                }
            }
            count
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS break while: {}", &js);
    assert!(js.contains("break"));
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_continue_in_for_loop_executes() {
    // Verify that continue in for loop generates correct, executable JS
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3, 4, 5];
            let mut sum = 0;
            for item in items {
                if item == 3 {
                    continue;
                }
                sum = sum + item;
            }
            sum
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS continue for: {}", &js);
    assert!(js.contains("continue"));
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    // 1 + 2 + 4 + 5 = 12 (skipping 3)
    assert_eq!(result.as_number().unwrap(), 12.0);
}

#[test]
fn test_return_with_value() {
    // Tests return statement in block (lines 1330-1335)
    let block: Block = parse_quote! {
        {
            return 42;
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS return: {}", &js);
    assert!(js.contains("return") && js.contains("42"));
}

#[test]
fn test_return_without_value() {
    // Tests return without value (lines 1335-1337)
    let block: Block = parse_quote! {
        {
            return;
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS return void: {}", &js);
    assert!(js.contains("return"));
}

#[test]
fn test_function_definition_in_block() {
    // Tests Stmt::Item::Fn in block (lines 1311-1313)
    let block: Block = parse_quote! {
        {
            fn helper(x: i32) -> i32 {
                x + 1
            }
            helper(5)
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS fn in block: {}", &js);
    assert!(js.contains("function") && js.contains("helper"));
}

#[test]
fn test_compound_assignment_ops() {
    // Tests various compound assignment operators
    let block: Block = parse_quote! {
        {
            let mut x = 10;
            x += 5;
            x -= 3;
            x *= 2;
            x /= 4;
            x %= 3;
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS compound assign: {}", &js);
    assert!(js.contains("+=") && js.contains("-="));
}

#[test]
fn test_bitwise_assignment_ops() {
    // Tests bitwise compound assignment operators
    let block: Block = parse_quote! {
        {
            let mut x = 255;
            x &= 0xF0;
            x |= 0x01;
            x ^= 0xFF;
            x
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS bitwise assign: {}", &js);
    assert!(js.contains("&=") || js.contains("|=") || js.contains("^="));
}

#[test]
fn test_self_path_resolution() {
    // Tests Expr::Path with "self" → "this" (line 1561)
    let expr: Expr = parse_quote! {
        self.value
    };
    let js = rust_expr_to_js(&expr);
    println!("JS self: {}", &js);
    assert!(js.contains("this"));
}

#[test]
fn test_none_path() {
    // Tests Expr::Path with "None" → null (line 1571)
    let expr: Expr = parse_quote! {
        None
    };
    let js = rust_expr_to_js(&expr);
    println!("JS None: {}", &js);
    assert!(js.contains("null"));
}

#[test]
fn test_some_call() {
    // Tests Some(value) function call
    let expr: Expr = parse_quote! {
        Some(42)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS Some: {}", &js);
    assert!(js.contains("42"));
}

#[test]
fn test_macro_in_block() {
    // Tests Stmt::Macro in block (lines 1396-1398)
    let block: Block = parse_quote! {
        {
            vec![1, 2, 3];
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS macro in block: {}", &js);
    assert!(js.contains("[") && js.contains("]"));
}

#[test]
fn test_for_loop_in_block_statement_context() {
    // Tests Expr::ForLoop in statement context (lines 1339-1342)
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            for item in items {
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS for in block: {}", &js);
    assert!(js.contains("for"));
}

#[test]
fn test_while_in_block_statement_context() {
    // Tests Expr::While in statement context (lines 1344-1348)
    let block: Block = parse_quote! {
        {
            let mut i = 0;
            while i < 5 {
                i = i + 1;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS while in block: {}", &js);
    assert!(js.contains("while"));
}

#[test]
fn test_if_in_block_statement_context() {
    // Tests Expr::If in statement context (lines 1350-1358)
    let block: Block = parse_quote! {
        {
            let x = 5;
            if x > 3 {
                x + 1;
            } else {
                x - 1;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS if in block: {}", &js);
    assert!(js.contains("if"));
}

#[test]
fn test_expression_with_semicolon_vs_without() {
    // Tests the semi handling (lines 1367-1391)
    let block: Block = parse_quote! {
        {
            let x = 5;
            x + 1;
            x + 2
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS semi: {}", &js);
    // Last expression without semi should be returned
    assert!(js.contains("return") || js.contains("x + 2") || js.contains("x+2"));
}

#[test]
fn test_map_and_filter_methods() {
    let expr: Expr = parse_quote! {
        items.map(|x| x + 1)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS map: {}", &js);
    assert!(js.contains("map"));

    let expr: Expr = parse_quote! {
        items.filter(|x| x > 0)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS filter: {}", &js);
    assert!(js.contains("filter"));

    let expr: Expr = parse_quote! {
        items.find(|x| x > 5)
    };
    let js = rust_expr_to_js(&expr);
    println!("JS find: {}", &js);
    assert!(js.contains("find"));
}
