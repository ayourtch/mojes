// additional_tests.rs - Suggested additional tests for better coverage

use boa_engine::{Context, JsResult, JsValue, Source};
use mojes_mojo::*;
use syn::{Block, Expr, ItemEnum, ItemStruct, parse_quote};

// Helper function to evaluate JS and get result
fn eval_js(code: &str) -> JsResult<JsValue> {
    let mut context = Context::default();
    context.eval(Source::from_bytes(code))
}

fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{\n{}}})();", block_js);
    eval_js(&code)
}

#[test]
fn test_current_string_escaping_behavior() {
    let expr: Expr = parse_quote!("Line 1\nLine 2\tTabbed\rCarriage\"Quote");
    let js_code = rust_expr_to_js(&expr);
    println!(
        "DEBUG test_current_string_escaping_behavior js code: {}",
        &js_code
    );

    // Currently escapes:
    assert!(js_code.contains("\\n")); // newlines
    assert!(js_code.contains("\\\"")); // quotes
    assert!(js_code.contains("\\t")); // tabs (literal)
    assert!(js_code.contains("\\r")); // carriage returns (literal)

    println!("Current escaping behavior: {}", js_code);
}

// ==================== UNARY OPERATIONS TESTS ====================
#[test]
fn test_unary_operations() {
    // Negation
    let expr: Expr = parse_quote!(-x);
    assert_eq!(rust_expr_to_js(&expr), "-x");

    // Logical NOT
    let expr: Expr = parse_quote!(!flag);
    assert_eq!(rust_expr_to_js(&expr), "!flag");

    // Dereference (should be handled)
    let expr: Expr = parse_quote!(*ptr);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "ptr"); // Should remove the dereference
}

// ==================== REFERENCE EXPRESSIONS TESTS ====================
#[test]
fn test_reference_expressions() {
    // Immutable reference to variable - your code specifically returns just the variable
    let expr: Expr = parse_quote!(&x);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "x"); // This is the actual behavior for Expr::Path

    // Mutable reference to variable - also returns just the variable
    let expr: Expr = parse_quote!(&mut y);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "y /* was &mut in Rust */"); // This should have the comment

    // Reference to string literal - should return just the string
    let expr: Expr = parse_quote!(&"hello");
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "\"hello\"");

    // Reference to a more complex expression (not a path) - should have comment
    let expr: Expr = parse_quote!(&(x + y));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "(x + y) /* was & in Rust */");
}
// Additional test to verify the path vs non-path reference behavior
#[test]
fn test_reference_expression_types() {
    // Path expressions (variables) - no comment
    let expr: Expr = parse_quote!(&variable_name);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "variable_name");

    // Literal expressions - no comment for string literals
    let expr: Expr = parse_quote!(&42);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "42 /* was & in Rust */");

    // Complex expressions - should have comment
    let expr: Expr = parse_quote!(&func_call());
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "func_call() /* was & in Rust */");

    // Field access - should have comment
    let expr: Expr = parse_quote!(&obj.field);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "obj.field /* was & in Rust */");
}

// Test mutable vs immutable references more thoroughly
#[test]
fn test_mutable_reference_behavior() {
    // Mutable reference to variable
    let expr: Expr = parse_quote!(&mut x);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "x /* was &mut in Rust */");

    // Mutable reference to complex expression
    let expr: Expr = parse_quote!(&mut (a + b));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "(a + b) /* was &mut in Rust */");
}

// ==================== BITWISE OPERATIONS TESTS ====================
#[test]
fn test_bitwise_operations() {
    let expr: Expr = parse_quote!(a ^ b);
    assert_eq!(rust_expr_to_js(&expr), "a ^ b");

    let expr: Expr = parse_quote!(x & y);
    assert_eq!(rust_expr_to_js(&expr), "x & y");

    let expr: Expr = parse_quote!(a | b);
    assert_eq!(rust_expr_to_js(&expr), "a | b");

    let expr: Expr = parse_quote!(x << 2);
    assert_eq!(rust_expr_to_js(&expr), "x << 2");

    let expr: Expr = parse_quote!(y >> 1);
    assert_eq!(rust_expr_to_js(&expr), "y >> 1");
}

// ==================== ASSIGNMENT OPERATIONS TESTS ====================
#[test]
fn test_assignment_operations() {
    // The code shows support for compound assignment in binary ops
    let expr: Expr = parse_quote!(x += 5);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "x += 5");

    let expr: Expr = parse_quote!(y -= 3);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "y -= 3");

    let expr: Expr = parse_quote!(z *= 2);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "z *= 2");

    let expr: Expr = parse_quote!(w /= 4);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "w /= 4");
}

// ==================== COMPLEX ENUM TESTS ====================
#[test]
fn test_enum_with_data() {
    let enum_def: ItemEnum = parse_quote! {
        enum Message {
            Quit,
            Move { x: i32, y: i32 },
            Write(String),
            ChangeColor(i32, i32, i32),
        }
    };

    let js_enum = generate_js_enum(&enum_def);
    assert!(js_enum.contains("const Message"));
    assert!(js_enum.contains("Quit: 'Quit'"));
    assert!(js_enum.contains("Move("));
    assert!(js_enum.contains("Write("));
    assert!(js_enum.contains("ChangeColor("));
}

// ==================== TUPLE STRUCT TESTS ====================
#[test]
fn test_tuple_struct() {
    let struct_def: ItemStruct = parse_quote! {
        struct Color(i32, i32, i32);
    };

    let js_class = generate_js_class_for_struct(&struct_def);
    assert!(js_class.contains("class Color"));
    assert!(js_class.contains("constructor(data)"));
}

// ==================== UNIT STRUCT TESTS ====================
#[test]
fn test_unit_struct() {
    let struct_def: ItemStruct = parse_quote! {
        struct Empty;
    };

    let js_class = generate_js_class_for_struct(&struct_def);
    assert!(js_class.contains("class Empty"));
    assert!(js_class.contains("constructor()"));
}

// ==================== COMPLEX MATCH EXPRESSIONS ====================
#[test]
fn test_match_with_variable_binding() {
    let expr: Expr = parse_quote! {
        match value {
            x => x + 1,
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("const x = _match_value"));
}

#[test]
fn test_match_with_mixed_patterns() {
    let expr: Expr = parse_quote! {
        match input {
            42 => "answer",
            x => format!("other: {}", x),
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("=== 42"));
    assert!(js_code.contains("const x = _match_value"));
}

// ==================== NESTED OPTION MATCHING ====================
#[test]
fn test_nested_option_match() {
    let expr: Expr = parse_quote! {
        match outer_opt {
            Some(inner_opt) => match inner_opt {
                Some(value) => value,
                None => 0,
            },
            None => -1,
        }
    };

    let js_code = rust_expr_to_js(&expr);
    // Should handle nested Option matching
    assert!(js_code.contains("!== null") || js_code.contains("!== undefined"));
}

// ==================== STRING CONCATENATION TESTS ====================
#[test]
fn test_string_concatenation_detection() {
    // String literal + variable
    let expr: Expr = parse_quote!("Hello " + name);
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("${"));

    // format! result + string
    let expr: Expr = parse_quote!(format!("Count: {}", n) + " items");
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("${"));
}

// ==================== METHOD CHAINING TESTS ====================
#[test]
fn test_method_chaining() {
    let expr: Expr = parse_quote!(text.trim().to_uppercase().len());
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("trim().toUpperCase().length()"));
}

#[test]
fn test_string_methods() {
    let expr: Expr = parse_quote!(s.starts_with("prefix"));
    assert!(rust_expr_to_js(&expr).contains("startsWith"));

    let expr: Expr = parse_quote!(s.ends_with("suffix"));
    assert!(rust_expr_to_js(&expr).contains("endsWith"));

    let expr: Expr = parse_quote!(s.trim_start());
    assert!(rust_expr_to_js(&expr).contains("trimStart"));

    let expr: Expr = parse_quote!(s.trim_end());
    assert!(rust_expr_to_js(&expr).contains("trimEnd"));
}

// ==================== ARRAY/VECTOR METHODS ====================
#[test]
fn test_vector_methods() {
    let expr: Expr = parse_quote!(vec.remove(index));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("splice"));

    let expr: Expr = parse_quote!(vec.insert(0, item));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("splice"));
}

// ==================== ITERATOR METHODS TESTS ====================
#[test]
fn test_iterator_methods() {
    let expr: Expr = parse_quote!(items.iter().map(process).collect());
    let js_code = rust_expr_to_js(&expr);
    // .iter() and .collect() should be removed, .map() should remain
    assert!(js_code.contains("map(process)"));
    assert!(!js_code.contains("iter()"));
    assert!(!js_code.contains("collect()"));

    let expr: Expr = parse_quote!(numbers.filter(is_even));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("filter(is_even)"));

    let expr: Expr = parse_quote!(items.find(predicate));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("find(predicate)"));
}

// ==================== NESTED BLOCKS TESTS ====================
#[test]
fn test_deeply_nested_blocks() {
    let expr: Expr = parse_quote! {
        {
            let x = {
                let y = 5;
                y * 2
            };
            x + 1
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("const y = 5"));
    assert!(js_code.contains("const x ="));
}

// ==================== COMPLEX LOOP TESTS ====================
#[test]
fn test_loop_with_break_continue() {
    // Note: break and continue aren't implemented yet, but we should test current behavior
    let block: Block = parse_quote! {
        {
            let mut sum = 0;
            for i in [1, 2, 3, 4, 5] {
                if i == 3 {
                    // This would be continue in real Rust
                    sum = sum;
                } else {
                    sum = sum + i;
                }
            }
            sum
        }
    };

    let js_code = rust_block_to_js(&block);
    let result = eval_block_as_function(&js_code).unwrap();
    assert_eq!(result.as_number().unwrap(), 12.0); // 1+2+4+5
}

// ==================== MIXED EXPRESSIONS IN BLOCKS ====================
#[test]
fn test_mixed_expressions_in_blocks() {
    let block: Block = parse_quote! {
        {
            let data = [1, 2, 3];
            let processed = data.map(|x| x * 2); // This won't work yet but let's see
            let result = if processed.len() > 0 {
                processed[0]
            } else {
                0
            };
            result
        }
    };

    let js_code = rust_block_to_js(&block);
    // Should at least generate valid structure
    assert!(js_code.contains("const data = [1, 2, 3]"));
}

// ==================== ERROR HANDLING TESTS ====================
#[test]
fn test_unsupported_expressions() {
    // Test that unsupported expressions generate comments
    let expr: Expr = parse_quote!(async { some_future.await });
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("Unsupported expression"));
}

// ==================== SPECIAL CHARACTER HANDLING ====================
#[test]
fn test_special_characters_in_strings() {
    let expr: Expr = parse_quote!("String with\nnewlines\tand\ttabs");
    let js_code = rust_expr_to_js(&expr);
    println!(
        "DEBUG test_special_characters_in_strings js code: {}",
        &js_code
    );
    assert!(js_code.contains("\\n"));
    assert!(js_code.contains("\\t"));

    let expr: Expr = parse_quote!("String with \"quotes\"");
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("\\\""));
}

// ==================== FORMAT MACRO EDGE CASES ====================
#[test]
fn test_format_macro_edge_cases() {
    // Empty format string
    let expr: Expr = parse_quote!(format!(""));
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "``");

    // Format with no placeholders but arguments (invalid Rust, but test behavior)
    let block: Block = parse_quote! {
        {
            println!("No placeholders", extra_arg);
        }
    };
    let js_code = rust_block_to_js(&block);
    // Should handle gracefully
    assert!(js_code.contains("console.log"));
}

// ==================== MACRO NESTING TESTS ====================
#[test]
fn test_nested_macros() {
    let block: Block = parse_quote! {
        {
            println!("{}", format!("Nested: {}", value));
        }
    };

    let js_code = rust_block_to_js(&block);
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("Nested"));
}

// ==================== STRUCT FIELD ACCESS TESTS ====================
#[test]
fn test_complex_field_access() {
    let expr: Expr = parse_quote!(person.address.street.number);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "person.address.street.number");

    // Tuple struct field access (by index)
    let expr: Expr = parse_quote!(point.0);
    let js_code = rust_expr_to_js(&expr);
    assert_eq!(js_code, "point.0");
}

// ==================== COMPLEX STRUCT INSTANTIATION ====================
#[test]
fn test_complex_struct_instantiation() {
    let expr: Expr = parse_quote! {
        Person {
            name: format!("User {}", id),
            age: calculate_age(birth_year),
            active: true,
        }
    };

    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("name:"));
    assert!(js_code.contains("age:"));
    assert!(js_code.contains("active: true"));
    assert!(js_code.contains("calculate_age"));
}

// ==================== EXECUTION FLOW TESTS ====================
#[test]
fn test_complex_execution_flow() {
    let block: Block = parse_quote! {
        {
            let mut result = 0;
            let numbers = [1, 2, 3, 4, 5];

            for num in numbers {
                let processed = match num % 2 {
                    0 => num * 2,
                    _ => num + 1,
                };
                result = result + processed;
            }

            if result > 20 {
                format!("Large result: {}", result)
            } else {
                format!("Small result: {}", result)
            }
        }
    };

    let js_code = rust_block_to_js(&block);
    let wrapped_code = format!(
        r#"
        const console = {{
            log: function(...args) {{ /* mock */ }},
            error: function(...args) {{ /* mock */ }}
        }};
        (function() {{
            {}
        }})();
    "#,
        js_code
    );

    let result = eval_js(&wrapped_code).unwrap();
    // Should return a string with the result
    assert!(result.is_string());
}

// ==================== TYPE CONVERSION TESTS ====================
#[test]
fn test_type_conversions() {
    // Test various Rust types to JS type mapping
    use syn::Type;

    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(format_rust_type(&ty), "Map");

    let ty: Type = parse_quote!(HashSet<String>);
    assert_eq!(format_rust_type(&ty), "Set");

    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(format_rust_type(&ty), "");

    let ty: Type = parse_quote!(&str);
    assert_eq!(format_rust_type(&ty), "string");
}

// ==================== SMART COMMA SPLIT TESTS ====================
#[test]
fn test_smart_comma_split_function() {
    // This tests the internal smart_comma_split function indirectly
    let expr: Expr = parse_quote!(format!("Hello {}, welcome to {}", "Alice", "Rust"));
    let js_code = rust_expr_to_js(&expr);
    assert!(js_code.contains("Alice"));
    assert!(js_code.contains("Rust"));
    assert!(js_code.contains("Hello"));
    assert!(js_code.contains("welcome"));
}

// ==================== COMPREHENSIVE INTEGRATION TEST ====================
#[test]
fn test_comprehensive_integration() {
    let block: Block = parse_quote! {
        {
            let person = Person { name: "Alice".to_string(), age: 30 };
            let greeting = format!("Hello, {}!", person.name);

            println!("{}", greeting);

            let numbers = [1, 2, 3, 4, 5];
            let mut sum = 0;

            for num in numbers {
                if num % 2 == 0 {
                    sum = sum + num;
                }
            }

            let result = match sum {
                0 => "No even numbers",
                x if x > 10 => "Many even numbers",
                _ => "Some even numbers",
            };

            format!("{} - Sum: {}", result, sum)
        }
    };

    let js_code = rust_block_to_js(&block);

    // Test that the generated code has all the expected parts
    assert!(js_code.contains("const person = { name:"));
    assert!(js_code.contains("console.log"));
    assert!(js_code.contains("for (const num of"));
    assert!(js_code.contains("if (num % 2 === 0)"));
    assert!(js_code.contains("_match_value"));

    // The complex match with guards won't work perfectly, but structure should be there
}
