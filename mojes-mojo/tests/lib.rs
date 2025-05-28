#[cfg(test)]
mod expressions;

#[cfg(test)]
mod tests {
    use boa_engine::{Context, JsResult, JsValue, Source};
    use mojes_mojo::*;
    use syn::{Expr, ItemEnum, ItemStruct, Type, parse_quote};

    // Helper function to create a JS context and evaluate code
    fn eval_js(code: &str) -> JsResult<JsValue> {
        let mut context = Context::default();
        context.eval(Source::from_bytes(code))
    }

    // Helper to check if JS code is syntactically valid
    fn is_valid_js(code: &str) -> bool {
        eval_js(code).is_ok()
    }

    // Helper to check if a JsValue represents an array
    fn is_js_array(value: &JsValue) -> bool {
        value.is_object()
            && value.as_object().map_or(false, |obj| {
                obj.get("length", &mut Context::default()).is_ok()
            })
    }

    #[test]
    fn test_format_rust_type_basic() {
        let ty: Type = parse_quote!(i32);
        assert_eq!(format_rust_type(&ty), "number");

        let ty: Type = parse_quote!(String);
        assert_eq!(format_rust_type(&ty), "string");

        let ty: Type = parse_quote!(bool);
        assert_eq!(format_rust_type(&ty), "boolean");

        let ty: Type = parse_quote!(Vec<i32>);
        assert_eq!(format_rust_type(&ty), "Array");
    }

    #[test]
    fn test_rust_expr_to_js_literals() {
        let expr: Expr = parse_quote!(42);
        assert_eq!(rust_expr_to_js(&expr), "42");

        let expr: Expr = parse_quote!("hello");
        assert_eq!(rust_expr_to_js(&expr), "\"hello\"");

        let expr: Expr = parse_quote!(true);
        assert_eq!(rust_expr_to_js(&expr), "true");
    }

    #[test]
    fn test_rust_expr_to_js_binary_ops() {
        let expr: Expr = parse_quote!(a + b);
        assert_eq!(rust_expr_to_js(&expr), "a + b");

        let expr: Expr = parse_quote!(x == y);
        assert_eq!(rust_expr_to_js(&expr), "x === y");

        let expr: Expr = parse_quote!(x != y);
        assert_eq!(rust_expr_to_js(&expr), "x !== y");
    }

    #[test]
    fn test_rust_expr_to_js_function_calls() {
        let expr: Expr = parse_quote!(println!("test"));
        let result = rust_expr_to_js(&expr);
        eprintln!("Result: {:?}", &result);
        assert!(result.contains("console.log"));

        let expr: Expr = parse_quote!(format!("Hello {}", name));
        let result = rust_expr_to_js(&expr);
        assert!(result.starts_with("`Hello ${"));
    }

    #[test]
    fn test_rust_expr_to_js_with_boa() {
        // Test that generated JS actually runs
        let expr: Expr = parse_quote!(2 + 3);
        let js_code = rust_expr_to_js(&expr);

        let result = eval_js(&js_code).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_array_handling_with_boa() {
        let expr: Expr = parse_quote!([1, 2, 3]);
        let js_code = rust_expr_to_js(&expr);

        // Wrap in a function to return the array
        let test_code = format!("(function() {{ return {}; }})()", js_code);
        let result = eval_js(&test_code).unwrap();

        assert!(is_js_array(&result));
    }

    #[test]
    fn test_generate_js_class_for_struct() {
        let struct_def: ItemStruct = parse_quote! {
            struct Person {
                name: String,
                age: i32,
            }
        };

        let js_class = generate_js_class_for_struct(&struct_def);

        // Test that generated class is valid JS
        assert!(is_valid_js(&js_class));
        assert!(js_class.contains("class Person"));
        assert!(js_class.contains("constructor(name, age)"));
        assert!(js_class.contains("toJSON()"));
        assert!(js_class.contains("fromJSON"));
    }

    #[test]
    fn test_js_class_functionality_with_boa() {
        let struct_def: ItemStruct = parse_quote! {
            struct Point { x: i32, y: i32 }
        };

        let js_class = generate_js_class_for_struct(&struct_def);

        let test_code = format!(
            r#"
            {}
            const p = new Point(10, 20);
            const json = p.toJSON();
            const p2 = Point.fromJSON(json);
            [p.x, p.y, p2.x, p2.y]
        "#,
            js_class
        );

        println!(
            "DEBUG test_js_class_functionality_with_boa js code: {}",
            &test_code
        );
        let result = eval_js(&test_code).unwrap();
        // Should return [10, 20, 10, 20]
        assert!(is_js_array(&result));
    }

    #[test]
    fn test_generate_js_enum() {
        let enum_def: ItemEnum = parse_quote! {
            enum Status {
                Active,
                Inactive,
                Pending,
            }
        };

        let js_enum = generate_js_enum(&enum_def);

        // Test that generated enum is valid JS
        assert!(is_valid_js(&js_enum));
        assert!(js_enum.contains("const Status"));
        assert!(js_enum.contains("Active: 'Active'"));
    }

    #[test]
    fn test_js_enum_functionality_with_boa() {
        let enum_def: ItemEnum = parse_quote! {
            enum Color { Red, Green, Blue }
        };

        let js_enum = generate_js_enum(&enum_def);

        let test_code = format!(
            r#"
            {}
            [Color.Red, isColor(Color.Red), isColor("invalid")]
        "#,
            js_enum
        );

        let result = eval_js(&test_code);
        // Should work without throwing
        assert!(result.is_ok());
    }

    #[test]
    fn test_rust_block_to_js_basic() {
        use syn::Block;

        let block: Block = parse_quote! {
            {
                let x = 5;
                let y = 10;
                x + y
            }
        };

        let js_code = rust_block_to_js(&block);

        // Should be valid JS
        let test_code = format!("(function() {{\n{}}})();", js_code);
        let result = eval_js(&test_code).unwrap();
        assert_eq!(result.as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_option_handling() {
        let expr: Expr = parse_quote!(Some(42));
        let js_code = rust_expr_to_js(&expr);
        assert_eq!(js_code, "42");

        let expr: Expr = parse_quote!(None);
        let js_code = rust_expr_to_js(&expr);
        assert_eq!(js_code, "null");
    }

    #[test]
    fn test_complex_expression_with_boa() {
        // Test a more complex Rust-like expression
        let expr: Expr = parse_quote! {
            if x > 0 { x * 2 } else { 0 }
        };

        let js_code = rust_expr_to_js(&expr);

        // Create a test function with the expression
        let test_code = format!(
            r#"
            function test(x) {{
                return {};
            }}
            [test(5), test(-3), test(0)]
        "#,
            js_code
        );

        let result = eval_js(&test_code).unwrap();
        assert!(is_js_array(&result));
    }
}
