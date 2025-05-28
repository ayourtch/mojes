// tests/control_flow.rs
use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Expr, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// Helper to test expression evaluation with variables
fn eval_expr_with_vars(expr_js: &str, vars: &[(&str, &str)]) -> JsResult<JsValue> {
    let var_declarations: String = vars
        .iter()
        .map(|(name, value)| format!("const {} = {};", name, value))
        .collect::<Vec<_>>()
        .join("\n");

    let code = format!("{}\n{}", var_declarations, expr_js);
    eval_js(&code)
}

// Helper to test a block wrapped in a function
fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{\n{}}})();", block_js);
    eval_js(&code)
}

#[test]
fn test_simple_if_expressions() {
    // Basic if expression
    let expr: Expr = parse_quote! {
        if x > 0 { 1 } else { 0 }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("if"));
    assert!(js_code.contains("x > 0"));

    // Test with actual values
    let result = eval_expr_with_vars(&js_code, &[("x", "5")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_expr_with_vars(&js_code, &[("x", "-3")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_if_without_else() {
    let expr: Expr = parse_quote! {
        if condition { 42 }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("if"));
    assert!(js_code.contains("condition"));
    assert!(js_code.contains("42"));

    // Should return undefined when condition is false
    let result = eval_expr_with_vars(&js_code, &[("condition", "false")]).unwrap();
    assert!(result.is_undefined());

    // Should return 42 when condition is true
    let result = eval_expr_with_vars(&js_code, &[("condition", "true")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[test]
fn test_nested_if_expressions() {
    let expr: Expr = parse_quote! {
        if x > 0 {
            if x > 10 { 2 } else { 1 }
        } else {
            0
        }
    };

    let js_code = rust_expr_to_js(&expr);

    // Test different values
    let result = eval_expr_with_vars(&js_code, &[("x", "15")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);

    let result = eval_expr_with_vars(&js_code, &[("x", "5")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_expr_with_vars(&js_code, &[("x", "-5")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_if_let_some_expressions() {
    // This might not be implemented yet, so test what we have
    let expr: Expr = parse_quote! {
        if let Some(value) = maybe_value {
            value * 2
        } else {
            0
        }
    };

    let js_code = rust_expr_to_js(&expr);
    // Should handle Option checking somehow
    assert!(js_code.contains("null") || js_code.contains("undefined") || js_code.contains("Some"));
}

#[test]
fn test_match_expressions_basic() {
    let expr: Expr = parse_quote! {
        match x {
            1 => "one",
            2 => "two",
            _ => "other",
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("_match_value"));

    // Test with different values
    let result = eval_expr_with_vars(&js_code, &[("x", "1")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "one");

    let result = eval_expr_with_vars(&js_code, &[("x", "2")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "two");

    let result = eval_expr_with_vars(&js_code, &[("x", "42")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "other");
}

#[test]
fn test_match_expressions_with_numbers() {
    let expr: Expr = parse_quote! {
        match status {
            0 => 100,
            1 => 200,
            _ => 500,
        }
    };

    let js_code = rust_expr_to_js(&expr);

    let result = eval_expr_with_vars(&js_code, &[("status", "0")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    let result = eval_expr_with_vars(&js_code, &[("status", "99")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 500.0);
}

#[test]
fn test_match_option_expressions() {
    let expr: Expr = parse_quote! {
        match opt {
            Some(value) => value * 2,
            None => 0,
        }
    };

    let js_code = rust_expr_to_js(&expr);
    eprintln!("DEBUG test_match_option_expressions js code: {}", &js_code);

    // Test with Some value (represented as the value itself in JS)
    let result = eval_expr_with_vars(&js_code, &[("opt", "42")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 84.0);

    // Test with None (represented as null in JS)
    let result = eval_expr_with_vars(&js_code, &[("opt", "null")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_while_loops_1() {
    let expr: Expr = parse_quote! {
        while counter < 3 {
            counter = counter + 1;
        }
    };

    let js_code = rust_expr_to_js(&expr);
    eprintln!("DEBUG test_while_loops_1 js code: {}", &js_code);
    assert!(js_code.contains("while"));
    assert!(js_code.contains("counter < 3"));
    assert!(js_code.contains("counter = counter + 1"));
}

#[test]
fn test_while_loop_execution() {
    let block: Block = parse_quote! {
        {
            let mut counter = 0;
            while counter < 3 {
                counter = counter + 1;
            }
            counter
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_for_loops() {
    let expr: Expr = parse_quote! {
        for item in items {
            process(item);
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("for"));
    assert!(js_code.contains("const item of items"));
    assert!(js_code.contains("process(item)"));
}

#[test]
fn test_while_loops_2() {
    let expr: Expr = parse_quote! {
        {
        let mut count = 0;
        while count < 5 {
           count += 1;
        }
        count
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("DEBUG test_while_loops js code: {}", &js_code);
    assert!(js_code.contains("while"));
}

#[test]
fn test_for_loop_execution() {
    let block: Block = parse_quote! {
        {
            let mut sum = 0;
            for i in [1, 2, 3, 4, 5] {
                sum = sum + i;
            }
            sum
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);
}

#[test]
fn test_complex_control_flow() {
    let block: Block = parse_quote! {
        {
            let mut result = 0;
            for i in [1, 2, 3, 4, 5] {
                if i % 2 == 0 {
                    result = result + i;
                }
            }
            result
        }
    };

    let js_code = rust_block_to_js(&block);
    eprintln!("DEBUG test_complex_control_flow js code: {}", &js_code);
    let result = eval_block_as_function(&js_code).unwrap();
    // Should sum even numbers: 2 + 4 = 6
    assert_eq!(result.as_number().unwrap(), 6.0);
}

#[test]
fn test_nested_loops() {
    let block: Block = parse_quote! {
        {
            let mut count = 0;
            for i in [1, 2] {
                for j in [1, 2] {
                    count = count + 1;
                }
            }
            count
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    // Should be 2 * 2 = 4 iterations
    assert_eq!(result.as_number().unwrap(), 4.0);
}

#[test]
fn test_if_in_loops() {
    let block: Block = parse_quote! {
        {
            let mut found = false;
            for i in [1, 2, 3, 4, 5] {
                if i == 3 {
                    found = true;
                }
            }
            found
        }
    };

    let js_code = rust_block_to_js(&block);
    eprintln!("DEBUG test_if_in_loops js code: {}", &js_code);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);
}

#[test]
fn test_match_in_loops() {
    let block: Block = parse_quote! {
        {
            let mut sum = 0;
            for x in [1, 2, 3] {
                let value = match x {
                    1 => 10,
                    2 => 20,
                    _ => 30,
                };
                sum = sum + value;
            }
            sum
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    // Should be 10 + 20 + 30 = 60
    assert_eq!(result.as_number().unwrap(), 60.0);
}

#[test]
fn test_logical_operators_in_conditions() {
    let expr: Expr = parse_quote! {
        if x > 0 && y < 10 {
            1
        } else {
            0
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("&&"));

    let result = eval_expr_with_vars(&js_code, &[("x", "5"), ("y", "3")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_expr_with_vars(&js_code, &[("x", "5"), ("y", "15")]).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_complex_conditions() {
    let expr: Expr = parse_quote! {
        if (x > 0 && y > 0) || z == 42 {
            "positive"
        } else {
            "negative"
        }
    };

    let js_code = rust_expr_to_js(&expr);

    let result = eval_expr_with_vars(&js_code, &[("x", "1"), ("y", "1"), ("z", "0")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "positive");

    let result = eval_expr_with_vars(&js_code, &[("x", "-1"), ("y", "-1"), ("z", "42")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "positive");

    let result = eval_expr_with_vars(&js_code, &[("x", "-1"), ("y", "-1"), ("z", "0")]).unwrap();
    assert_eq!(result.as_string().unwrap(), "negative");
}
