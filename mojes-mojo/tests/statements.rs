// tests/statements.rs
use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Stmt, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

// Helper to test a block wrapped in a function
fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{\n{}}})();", block_js);
    eval_js(&code)
}

#[test]
fn test_variable_declarations() {
    // Simple let binding
    let block: Block = parse_quote! {
        {
            let x = 5;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const x = 5"));

    // Mutable let binding
    let block: Block = parse_quote! {
        {
            let mut y = 10;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("let y = 10"));
}

#[test]
fn test_variable_declarations_with_types() {
    // Variable with different types
    let block: Block = parse_quote! {
        {
            let name = "Alice";
            let age = 30;
            let active = true;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const name = \"Alice\""));
    assert!(js_code.contains("const age = 30"));
    assert!(js_code.contains("const active = true"));
}

#[test]
fn test_variable_without_initialization() {
    // Uninitialized variables
    let block: Block = parse_quote! {
        {
            let x;
            let mut y;
        }
    };

    let js_code = rust_block_to_js(&block);
    // Both should become 'let' in JS since they're uninitialized
    assert!(js_code.contains("let x;"));
    assert!(js_code.contains("let y;"));
}

#[test]
fn test_expression_statements() {
    // Expression with semicolon
    let block: Block = parse_quote! {
        {
            x + y;
            func_call();
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("x + y;"));
    assert!(js_code.contains("func_call();"));
}

#[test]
fn test_return_statements() {
    // Return with value
    let block: Block = parse_quote! {
        {
            return 42;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("return 42;"));

    // Return without semicolon (implicit return)
    let block: Block = parse_quote! {
        {
            42
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("return 42;"));
}

#[test]
fn test_destructuring_patterns() {
    // Tuple destructuring
    let block: Block = parse_quote! {
        {
            let (x, y) = point;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const [x, y] = point"));

    // Struct destructuring
    let block: Block = parse_quote! {
        {
            let Person { name, age } = person;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const { name, age } = person"));
}

#[test]
fn test_block_execution_simple() {
    // Test that generated JS actually executes correctly
    let block: Block = parse_quote! {
        {
            let x = 5;
            let y = 10;
            x + y
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);
}

#[test]
fn test_block_execution_with_mutations() {
    let block: Block = parse_quote! {
        {
            let mut counter = 0;
            counter = counter + 1;
            counter = counter * 2;
            counter
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);
}

#[test]
fn test_nested_blocks() {
    let block: Block = parse_quote! {
        {
            let x = 1;
            {
                let y = 2;
                x + y
            }
        }
    };

    let js_code = rust_block_to_js(&block);
    // Should contain the nested structure
    assert!(js_code.contains("const x = 1"));
    assert!(js_code.contains("const y = 2"));
}

#[test]
fn test_multiple_statements() {
    let block: Block = parse_quote! {
        {
            let a = 1;
            let b = 2;
            let c = a + b;
            let result = c * 2;
            result
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);
}

#[test]
fn test_function_calls_in_statements() {
    let block: Block = parse_quote! {
        {
            println!("Starting");
            let value = calculate(5, 3);
            value
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("const value = calculate(5, 3)"));
    assert!(js_code.contains("return value;"));
}

#[test]
fn test_method_calls_in_statements() {
    let block: Block = parse_quote! {
        {
            let mut vec = create_vec();
            vec.push(42);
            vec.len()
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("let vec = create_vec()"));
    assert!(js_code.contains("vec.push(42)"));
    assert!(js_code.contains("return vec.length()"));
}

#[test]
fn test_assignment_statements() {
    let block: Block = parse_quote! {
        {
            let mut x = 0;
            x = 5;
            x = x + 1;
            x
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("let x = 0"));
    assert!(js_code.contains("x = 5"));
    assert!(js_code.contains("x = x + 1"));
}

#[test]
fn test_complex_expressions_in_statements() {
    let block: Block = parse_quote! {
        {
            let result = (a + b) * (c - d);
            let flag = x > 0 && y < 10;
            result
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const result = (a + b) * (c - d)"));
    assert!(js_code.contains("const flag = x > 0 && y < 10"));
    assert!(js_code.contains("return result;"));
}

#[test]
fn test_array_operations_in_statements() {
    let block: Block = parse_quote! {
        {
            let arr = [1, 2, 3];
            let first = arr[0];
            first
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const arr = [1, 2, 3]"));
    assert!(js_code.contains("const first = arr[0]"));
    assert!(js_code.contains("return first;"));
}

#[test]
fn test_struct_operations_in_statements() {
    let block: Block = parse_quote! {
        {
            let point = Point { x: 10, y: 20 };
            let x_coord = point.x;
            x_coord
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const point = { x: 10, y: 20 }"));
    assert!(js_code.contains("const x_coord = point.x"));
    assert!(js_code.contains("return x_coord;"));
}

#[test]
fn test_option_handling_in_statements() {
    let block: Block = parse_quote! {
        {
            let maybe_value = Some(42);
            let is_present = maybe_value.is_some();
            is_present
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const maybe_value = 42"));
    assert!(js_code.contains("!== null") || js_code.contains("!== undefined"));
}

#[test]
fn test_macro_calls_in_statements() {
    let block: Block = parse_quote! {
        {
            let greeting = format!("Hello, {}!", name);
            println!("Debug: {}", greeting);
            greeting
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("Hello"));
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("return greeting;"));
}

#[test]
fn test_empty_block() {
    let block: Block = parse_quote! { {} };

    let js_code = rust_block_to_js(&block);
    // Should produce valid but empty JS
    let result = eval_block_as_function(&js_code);
    assert!(result.is_ok());
}

#[test]
fn test_single_expression_block() {
    let block: Block = parse_quote! { { 42 } };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[test]
fn test_mixed_statement_types() {
    let block: Block = parse_quote! {
        {
            let x = 5;           // Variable declaration
            x + 1;               // Expression statement
            let y = x * 2;       // Variable declaration with expression
            println!("y is {}", y); // Macro call
            y                    // Final expression (return)
        }
    };

    let js_code = rust_block_to_js(&block);

    // Check all parts are present
    assert!(js_code.contains("const x = 5"));
    assert!(js_code.contains("x + 1;"));
    assert!(js_code.contains("const y = x * 2"));
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("return y;"));
}
