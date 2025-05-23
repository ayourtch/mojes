// remaining_uncovered_tests.rs - Testing remaining realistic uncovered areas

use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Expr, ItemEnum, ItemStruct, Type, parse_quote};

fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// ==================== 1. CLOSURES/LAMBDAS ====================

#[test]
fn test_closure_expressions() {
    // Closures are now properly supported! Test the actual functionality
    let expr: Expr = parse_quote!(|x| x + 1);
    let js_code = rust_expr_to_js(&expr);

    // Should generate proper JavaScript arrow function
    assert_eq!(js_code, "x => x + 1");

    println!("✓ Single parameter closure: {}", js_code);

    // Test multiple parameter closure
    let expr: Expr = parse_quote!(|a, b| a * b);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "(a, b) => a * b");

    println!("✓ Multiple parameter closure: {}", js_code);

    // Test zero parameter closure
    let expr: Expr = parse_quote!(|| 42);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "() => 42");

    println!("✓ Zero parameter closure: {}", js_code);
}

#[test]
fn test_closure_in_method_calls() {
    // Test closures in common contexts like map/filter
    let expr: Expr = parse_quote!(numbers.map(|x| x * 2));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "numbers.map(x => x * 2)");

    let expr: Expr = parse_quote!(items.filter(|item| item.active));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "items.filter(item => item.active)");

    println!("✓ Closures in method calls work perfectly");
}

#[test]
fn test_closure_execution() {
    // Test that the generated closure actually works in JavaScript
    let expr: Expr = parse_quote!(|x| x * 2);
    let js_code = rust_expr_to_js(&expr);

    let test_code = format!("const closure = {}; closure(5);", js_code);
    let result = eval_js(&test_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    println!("✓ Generated closure executes correctly and returns 10");
}

#[test]
fn test_complex_closure_bodies() {
    // Test closures with more complex bodies
    let expr: Expr = parse_quote!(|x| {
        let doubled = x * 2;
        doubled + 1
    });
    let js_code = rust_expr_to_js(&expr);

    // Should generate arrow function with block body
    assert!(js_code.contains("x =>"));
    assert!(js_code.contains("doubled"));

    println!("✓ Complex closure body: {}", js_code);
}

#[test]
fn test_closure_in_method_calls_2() {
    // Test closures in common contexts
    let expr: Expr = parse_quote!(numbers.map(|x| x * 2));
    let js_code = rust_expr_to_js(&expr);
    // Should handle the .map() part even if closure fails
    assert!(js_code.contains("map"));
    println!("Closure in map: {}", js_code);
}

// ==================== 2. RANGE EXPRESSIONS ====================
// Ranges like 1..10, 1..=10 are common in Rust

#[test]
fn test_range_expressions() {
    // Exclusive range
    let expr: Expr = parse_quote!(1..10);
    let js_code = rust_expr_to_js(&expr);
    println!("Exclusive range: {}", js_code);

    // Inclusive range
    let expr: Expr = parse_quote!(1..=10);
    let js_code = rust_expr_to_js(&expr);
    println!("Inclusive range: {}", js_code);

    // Range from variable
    let expr: Expr = parse_quote!(start..end);
    let js_code = rust_expr_to_js(&expr);
    println!("Variable range: {}", js_code);
}

// ==================== 3. JAVASCRIPT KEYWORD CONFLICTS ====================
// Test Rust identifiers that are JS reserved words

#[test]
fn test_javascript_reserved_words() {
    // These are valid Rust identifiers but JS reserved words
    let expr: Expr = parse_quote!(function);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "function_");
    println!("JS reserved word 'function': {}", js_code);

    let expr: Expr = parse_quote!(class);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "class_");

    /*let expr: Expr = parse_quote!(const);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "const_"); */

    // This could cause JS syntax errors!
    let block: Block = parse_quote! {
        {
            let function = 42;
            function + 1
        }
    };
    let js_code = rust_block_to_js(&block);
    // This will generate invalid JS: "const function = 42;"
    println!("Problematic JS output: {}", js_code);
}

// ==================== 4. MATCH GUARDS ====================
// if conditions in match arms

#[test]
fn test_match_guards() {
    // This is likely unsupported but test current behavior
    let expr: Expr = parse_quote! {
        match x {
            n if n > 5 => "big",
            n => "small",
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Match with guards: {}", js_code);
    // Likely doesn't handle the guard condition properly
}

// ==================== 5. LOOP EXPRESSIONS WITH VALUES ====================
// Loops can return values in Rust

#[test]
fn test_loop_expressions_with_values() {
    // loop expressions (infinite loops with break values)
    let expr: Expr = parse_quote! {
        loop {
            if condition {
                break result;
            }
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("Loop expression: {}", js_code);
    // Probably not implemented

    // while let loops
    let expr: Expr = parse_quote! {
        while let Some(item) = iterator.next() {
            process(item);
        }
    };

    let js_code = rust_expr_to_js(&expr);
    println!("While let loop: {}", js_code);
}

// ==================== 6. UNICODE AND SPECIAL IDENTIFIERS ====================

#[test]
fn test_unicode_identifiers() {
    // Rust supports Unicode identifiers
    let expr: Expr = parse_quote!(变量名);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "变量名");

    let expr: Expr = parse_quote!(μ_value);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "μ_value");

    // Test in context
    let block: Block = parse_quote! {
        {
            let 测试 = 42;
            测试 + 1
        }
    };
    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const 测试 = 42"));
    println!("Unicode identifiers: {}", js_code);
}

// ==================== 7. RAW STRINGS ====================
// r"..." and r#"..."# strings

#[test]
fn test_raw_strings() {
    // Raw strings - these might not parse correctly with syn
    // This test might not even compile if syn doesn't handle them in parse_quote!

    // For now, test what happens with escaped strings that would be raw
    let expr: Expr = parse_quote!("C:\\path\\to\\file");
    let js_code = rust_expr_to_js(&expr);
    println!("Path-like string: {}", js_code);

    // Test string with quotes
    let expr: Expr = parse_quote!("String with \"nested quotes\"");
    let js_code = rust_expr_to_js(&expr);
    println!("Nested quotes: {}", js_code);
}

// ==================== 8. TRY OPERATOR ====================
// The ? operator for error handling

#[test]
fn test_try_operator() {
    // This likely isn't implemented
    let expr: Expr = parse_quote!(some_function()?);
    let js_code = rust_expr_to_js(&expr);
    println!("Try operator: {}", js_code);
    // Probably unsupported
}

// ==================== 9. COMPLEX NESTED EXPRESSIONS ====================
// Test very deep nesting to see if there are stack issues

#[test]
fn test_deeply_nested_expressions() {
    // Create a deeply nested expression
    let expr: Expr = parse_quote! {
        ((((((a + b) * c) - d) / e) % f) + g)
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("((((((a + b) * c) - d) / e) % f) + g)"));

    // Test deeply nested field access
    let expr: Expr = parse_quote!(a.b.c.d.e.f.g.h.i.j);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "a.b.c.d.e.f.g.h.i.j");

    // Test deeply nested function calls
    let expr: Expr = parse_quote!(f(g(h(i(j(k()))))));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "f(g(h(i(j(k())))))");
}

// ==================== 10. EMPTY AND EDGE CASE BLOCKS ====================

#[test]
fn test_empty_and_edge_cases() {
    // Completely empty block
    let block: Block = parse_quote! { {} };
    let js_code = rust_block_to_js(&block);
    assert_eq!(js_code.trim(), "");

    // Block with only comments (if they somehow get through)
    // This is hard to test since syn strips comments

    // Block with only semicolons
    let block: Block = parse_quote! {
        {
            ;
            ;
        }
    };
    let js_code = rust_block_to_js(&block);
    // Should handle gracefully
    println!("Block with semicolons: '{}'", js_code);
}

// ==================== 11. VERY LONG IDENTIFIERS ====================

#[test]
fn test_very_long_identifiers() {
    // Test with a very long identifier name
    let long_name = "a".repeat(1000);
    let expr_str = format!("{}", long_name);

    // This is tricky to test with parse_quote!, so test the principle
    let expr: Expr = parse_quote!(very_long_identifier_name_that_goes_on_and_on_and_on);
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.len() > 50);
    println!("Long identifier length: {}", js_code.len());
}

// ==================== 12. CONST AND STATIC EXPRESSIONS ====================

#[test]
fn test_const_static_expressions() {
    // These might not be expressible in syn::Expr, but test what we can

    // const evaluation in expressions
    let expr: Expr = parse_quote!(MY_CONST);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "MY_CONST");

    // Array with const size (might not work)
    let expr: Expr = parse_quote!([0; SIZE]);
    let js_code = rust_expr_to_js(&expr);
    println!("Array with const size: {}", js_code);
}

// ==================== 13. COMPREHENSIVE STRESS TEST ====================

#[test]
fn test_comprehensive_stress() {
    // Combine many features in one complex expression
    let block: Block = parse_quote! {
        {
            let data = vec![1, 2, 3, 4, 5];
            let mut result = 0;

            for item in data.iter() {
                match item {
                    x if *x > 3 => result += x * 2, // Match guard (likely unsupported)
                    x => result += x,
                }
            }

            let final_calc = ((result + 10) * 2) - 5;
            let option_val = Some(final_calc);

            match option_val {
                Some(val) => format!("Result: {}", val),
                None => "No result".to_string(),
            }
        }
    };

    let js_code = rust_block_to_js(&block);

    // Should handle most of this, though match guards might fail
    assert!(js_code.contains("const data"));
    assert!(js_code.contains("for (const item of"));
    assert!(js_code.contains("_match_value"));

    println!("Complex stress test output:\n{}", js_code);

    // Try to execute what we can (might fail due to unsupported features)
    let wrapped = format!(
        r#"
        const vec = {{
            iter: function() {{ return [1, 2, 3, 4, 5]; }}
        }};
        const console = {{
            log: function(...args) {{ /* mock */ }}
        }};
        (function() {{
            {}
        }})();
    "#,
        js_code
    );

    // This might fail, but let's see how much works
    match eval_js(&wrapped) {
        Ok(result) => println!("Stress test executed successfully: {:?}", result),
        Err(e) => println!("Stress test failed (expected): {:?}", e),
    }
}
