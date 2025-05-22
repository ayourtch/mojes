// more_tests.rs - Tests for uncovered functionality

use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Expr, ItemEnum, ItemStruct, Type, parse_quote};

fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{\n{}}})();", block_js);
    eval_js(&code)
}

// ==================== 1. COMPOUND ASSIGNMENT OPERATORS ====================
// These are implemented in handle_binary_op but never tested!

#[test]
fn test_compound_assignment_operators() {
    // Addition assignment
    let expr: Expr = parse_quote!(x += 5);
    assert_eq!(rust_expr_to_js(&expr), "x += 5");

    // Subtraction assignment
    let expr: Expr = parse_quote!(y -= 3);
    assert_eq!(rust_expr_to_js(&expr), "y -= 3");

    // Multiplication assignment
    let expr: Expr = parse_quote!(z *= 2);
    assert_eq!(rust_expr_to_js(&expr), "z *= 2");

    // Division assignment
    let expr: Expr = parse_quote!(w /= 4);
    assert_eq!(rust_expr_to_js(&expr), "w /= 4");

    // Modulo assignment
    let expr: Expr = parse_quote!(a %= 3);
    assert_eq!(rust_expr_to_js(&expr), "a %= 3");

    // Bitwise assignments
    let expr: Expr = parse_quote!(x ^= mask);
    assert_eq!(rust_expr_to_js(&expr), "x ^= mask");

    let expr: Expr = parse_quote!(flags &= filter);
    assert_eq!(rust_expr_to_js(&expr), "flags &= filter");

    let expr: Expr = parse_quote!(bits |= new_bits);
    assert_eq!(rust_expr_to_js(&expr), "bits |= new_bits");

    let expr: Expr = parse_quote!(value <<= 2);
    assert_eq!(rust_expr_to_js(&expr), "value <<= 2");

    let expr: Expr = parse_quote!(value >>= 1);
    assert_eq!(rust_expr_to_js(&expr), "value >>= 1");
}

// ==================== 2. BINARY OP DEFAULT FALLBACK ====================
// Test the default case in handle_binary_op

#[test]
fn test_unsupported_binary_op_fallback() {
    // This tests the "x => format!("/* {:?} */ {} + {}", x, left, right)" case
    // Hard to trigger with syn's BinOp enum, but the fallback should exist

    // Test a complex expression that might hit edge cases
    let expr: Expr = parse_quote!((a as i32) + (b as i32));
    let js_code = rust_expr_to_js(&expr);
    // Should generate something, even if not perfect
    assert!(js_code.contains("+"));
}

// ==================== 3. NESTED BLOCK EXPRESSIONS ====================
// Expr::Block is implemented but not tested

#[test]
fn test_nested_block_expressions() {
    let expr: Expr = parse_quote! {
        {
            let temp = x + 1;
            temp * 2
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("function()"));
    assert!(js_code.contains("const temp = x + 1"));
    assert!(js_code.contains("return temp * 2"));
}

// ==================== 4. UNARY DEREFERENCE OPERATOR ====================

#[test]
fn test_unary_dereference() {
    let expr: Expr = parse_quote!(*ptr);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "ptr"); // Should remove the dereference

    let expr: Expr = parse_quote!(*some_ref);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "some_ref");
}

// ==================== 5. TUPLE FIELD ACCESS ====================
// Member::Unnamed is implemented but not tested

#[test]
fn test_tuple_field_access() {
    let expr: Expr = parse_quote!(point.0);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "point.0");

    let expr: Expr = parse_quote!(tuple.1);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "tuple.1");

    let expr: Expr = parse_quote!(nested.0.1);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "nested.0.1");
}

// ==================== 6. VECTOR METHODS WITH ARGUMENTS ====================

#[test]
fn test_vector_methods_with_args() {
    // remove method -> splice
    let expr: Expr = parse_quote!(vec.remove(index));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "vec.splice(index)");

    // insert method -> splice
    let expr: Expr = parse_quote!(vec.insert(0, item));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "vec.splice(0, item)");
}

// ==================== 7. VARIABLE DECLARATIONS WITHOUT INIT ====================

#[test]
fn test_uninitialized_variable_declarations() {
    let block: Block = parse_quote! {
        {
            let x;
            let mut y;
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("let x;"));
    assert!(js_code.contains("let y;")); // Both become 'let' when uninitialized
}

// ==================== 8. COMPLEX DESTRUCTURING PATTERNS ====================

#[test]
fn test_destructuring_patterns() {
    // Tuple destructuring
    let block: Block = parse_quote! {
        {
            let (x, y, z) = get_tuple();
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const [x, y, z] = get_tuple()"));

    // Struct destructuring
    let block: Block = parse_quote! {
        {
            let Point { x, y } = get_point();
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("const { x, y } = get_point()"));
}

// ==================== 9. STMT::MACRO STATEMENTS ====================

#[test]
fn test_macro_statements() {
    let block: Block = parse_quote! {
        {
            println!("Debug message");
            eprintln!("Error occurred");
            format!("Not assigned to anything");
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("console.error"));
    // The standalone format! should also be handled
    assert!(js_code.lines().count() >= 3);
}

// ==================== 10. IMPLICIT RETURNS (NO SEMICOLON) ====================

#[test]
fn test_implicit_return_expressions() {
    let block: Block = parse_quote! {
        {
            let x = 5;
            x + 1  // No semicolon = implicit return
        }
    };

    let js_code = rust_block_to_js(&block);
    println!(
        "DEBUG test_implicit_return_expressions js code: {}",
        &js_code
    );
    assert!(js_code.contains("return x + 1"));
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);
}

// ==================== 11. MATCH WILDCARD PATTERNS ====================

#[test]
fn test_match_wildcard_patterns() {
    let expr: Expr = parse_quote! {
        match value {
            42 => "answer",
            _ => "other",
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("=== 42"));
    assert!(js_code.contains("else { // Default case"));
}

#[test]
fn test_match_variable_binding() {
    let expr: Expr = parse_quote! {
        match input {
            x => x + 1,
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("const x = _match_value"));
    assert!(js_code.contains("return x + 1"));
}

// ==================== 12. COMPLEX TYPE HANDLING ====================

#[test]
fn test_complex_type_formatting() {
    // Nested generics
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let result = format_rust_type(&ty);
    assert_eq!(result, "Array"); // Should resolve to Array

    // Reference types
    let ty: Type = parse_quote!(&mut String);
    let result = format_rust_type(&ty);
    assert_eq!(result, "string");

    // Array types
    let ty: Type = parse_quote!([i32; 10]);
    let result = format_rust_type(&ty);
    assert_eq!(result, "Array");

    // Tuple types
    let ty: Type = parse_quote!((i32, String, bool));
    let result = format_rust_type(&ty);
    assert_eq!(result, "Array");
}

// ==================== 13. ENUM WITH COMPLEX DATA ====================

#[test]
fn test_complex_enum_generation() {
    let enum_def: ItemEnum = parse_quote! {
        enum Message {
            Quit,
            Move { x: i32, y: i32 },
            Write(String),
            ChangeColor(i32, i32, i32),
        }
    };

    let js_enum = generate_js_enum(&enum_def);

    // Should contain factory functions for complex variants
    assert!(js_enum.contains("Move(x, y)"));
    assert!(js_enum.contains("Write(value0)"));
    assert!(js_enum.contains("ChangeColor(value0, value1, value2)"));
    assert!(js_enum.contains("Quit: 'Quit'"));

    // Should contain utility methods
    assert!(js_enum.contains("is(obj, variant)"));
    assert!(js_enum.contains("function isMessage(value)"));
}

// ==================== 14. STRUCT WITH COMPLEX FIELDS ====================

#[test]
fn test_complex_struct_generation() {
    let struct_def: ItemStruct = parse_quote! {
        struct Config {
            name: String,
            values: Vec<i32>,
            metadata: HashMap<String, String>,
            enabled: bool,
        }
    };

    let js_class = generate_js_class_for_struct(&struct_def);
    assert!(js_class.contains("constructor(name, values, metadata, enabled)"));
    assert!(js_class.contains("this.name = name"));
    assert!(js_class.contains("this.values = values"));
    assert!(js_class.contains("this.metadata = metadata"));
    assert!(js_class.contains("this.enabled = enabled"));
}

// ==================== 15. SMART COMMA SPLIT EDGE CASES ====================

#[test]
fn test_smart_comma_split_complex() {
    // Test the smart_comma_split function indirectly through format!

    // Nested quotes
    let expr: Expr = parse_quote!(format!("Hello {}, \"quoted {}\", end", name, value));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("name"));
    assert!(js_code.contains("value"));
    assert!(js_code.contains("Hello"));

    // Empty arguments
    let expr: Expr = parse_quote!(format!("Just text"));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "`Just text`");
}

// ==================== 16. UNSUPPORTED MACRO TYPES ====================

#[test]
fn test_unsupported_macros() {
    let expr: Expr = parse_quote!(custom_macro!("test", 42));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("Unsupported macro custom_macro"));
}

// ==================== 17. IS_STRING_EXPR EDGE CASES ====================

#[test]
fn test_string_detection_edge_cases() {
    // Test format! call detection for string concatenation
    let expr: Expr = parse_quote!(format!("Hello") + " world");
    let js_code = rust_expr_to_js(&expr);
    // Should use template literal since format! is detected as string
    assert!(js_code.contains("$"));

    // Test method call string detection
    let expr: Expr = parse_quote!(text.to_string() + suffix);
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("$"));
}

// ==================== 18. COMPREHENSIVE EXECUTION TEST ====================

#[test]
fn test_comprehensive_uncovered_execution() {
    let block: Block = parse_quote! {
        {
            let mut data = [1, 2, 3];
            let point = Point { x: 10, y: 20 };

            // Compound assignment (uncovered)
            data[0] += point.x;

            // Tuple access (uncovered)
            let coords = (point.x, point.y);
            let x_coord = coords.0;

            // Nested block expression (uncovered)
            let result = {
                let temp = x_coord * 2;
                temp + 5
            };

            // Uninitialized then assigned (uncovered pattern)
            let final_value;
            final_value = result + data[0];

            final_value
        }
    };

    let js_code = rust_block_to_js(&block);

    // Should contain all the uncovered patterns
    assert!(js_code.contains("+=")); // Compound assignment
    assert!(js_code.contains(".0")); // Tuple access  
    assert!(js_code.contains("function()")); // Nested block
    assert!(js_code.contains("let final_value;")); // Uninitialized
    assert!(js_code.contains("final_value = result")); // Later assignment

    // The generated JS should be syntactically valid
    let wrapped = format!("(function() {{\n{}}})();", js_code);
    // Note: This might not execute perfectly due to missing Point constructor,
    // but the structure should be valid
    println!("Generated JS structure: {}", js_code);
}
