#[cfg(test)]
mod tests_dom {
    use boa_engine::{Context, JsResult, JsValue, Source};
    use mojes_mojo::*;
    use syn::{Block, Expr, parse_quote};

    // Helper function to create a JS context and evaluate code
    fn eval_js(code: &str) -> JsResult<JsValue> {
        let mut context = Context::default();
        context.eval(Source::from_bytes(code))
    }

    // Helper function to evaluate a block as a function and return the result
    fn eval_block_as_function(block_js: &str) -> JsResult<JsValue> {
        let function_code = format!("(function() {{\n{}}})();", block_js);
        eval_js(&function_code)
    }

    // Helper to check if JS code is syntactically valid
    fn is_valid_js(code: &str) -> bool {
        eval_js(code).is_ok()
    }

    // Helper to set up a mock DOM environment for testing
    fn setup_mock_dom() -> String {
        r#"
        // Mock DOM implementation for testing
        const document = {
            elements: [
                { tagName: 'DIV', id: 'div1' },
                { tagName: 'P', id: 'p1' },
                { tagName: 'P', id: 'p2' },
                { tagName: 'SPAN', id: 'span1' }
            ],
            createElement: function(tag) {
                return { tagName: tag.toUpperCase(), id: 'new_' + tag };
            },
            getElementsByTagName: function(tag) {
                return this.elements.filter(el => el.tagName === tag.toUpperCase());
            },
            getElementById: function(id) {
                return this.elements.find(el => el.id === id) || null;
            }
        };
        
        const console = {
            logs: [],
            log: function(msg) { this.logs.push(msg); }
        };
        "#
        .to_string()
    }

    #[test]
    fn test_closure_in_timeout_no_iife() {
        // Test that closures in setTimeout don't get wrapped in IIFE
        let expr: Expr = parse_quote! {
            setTimeout(|| {
                console.log("timeout fired");
            }, 1000)
        };

        let js_code = rust_expr_to_js(&expr);
        println!("DEBUG closure in timeout: {}", &js_code);

        // Should not contain IIFE pattern
        assert!(!js_code.contains("(function()"));
        assert!(!js_code.contains("})()"));

        // Should contain proper arrow function
        assert!(js_code.contains("() => {"));
        assert!(js_code.contains("console.log"));

        // Should be valid JS
        let test_code = format!(
            "{}\nconst console = {{log: () => {{}}}}; {};",
            "const setTimeout = (fn, delay) => fn;", js_code
        );
        assert!(is_valid_js(&test_code));
    }

    #[test]
    fn test_closure_in_interval_no_iife() {
        // Test that closures in setInterval don't get wrapped in IIFE
        let expr: Expr = parse_quote! {
            setInterval(|| {
                console.log("interval fired");
            }, 500)
        };

        let js_code = rust_expr_to_js(&expr);
        println!("DEBUG closure in interval: {}", &js_code);

        // Should not contain IIFE pattern
        assert!(!js_code.contains("(function()"));
        assert!(!js_code.contains("})()"));

        // Should contain proper arrow function
        assert!(js_code.contains("() => {"));
        assert!(js_code.contains("console.log"));
    }

    /* FIXME: fails
        #[test]
        fn test_for_loop_with_enumerate_entries() {
            // Test that for loops with enumerate use .entries() instead of .map()
            let block: Block = parse_quote! {
                {
                    let elements = document.getElementsByTagName("p");
                    for (i, element) in elements.iter().enumerate() {
                        console.log(&format!("Element {}: {}", i, element.tagName));
                    }
                }
            };

            let js_code = rust_block_to_js(&block);
            println!("DEBUG enumerate for loop: {}", &js_code);

            // Should use Array.from().entries() pattern
            assert!(js_code.contains("Array.from("));
            assert!(js_code.contains(".entries()"));
            assert!(!js_code.contains(".map("));

            // Should be valid JS
            assert!(is_valid_js(&format!("{}\n{}", setup_mock_dom(), js_code)));
        }
    */

    #[test]
    fn test_for_loop_with_enumerate_manual_counter() {
        // Test the manual counter approach for enumerate
        let block: Block = parse_quote! {
            {
                let elements = document.getElementsByTagName("p");
                for (i, element) in elements.iter().enumerate() {
                    console.log(&format!("Element {}: {}", i, element.tagName));
                }
            }
        };

        let js_code = rust_block_to_js(&block);
        println!("DEBUG manual counter for loop: {}", &js_code);

        // Should use manual counter approach
        assert!(js_code.contains("let i = 0;"));
        assert!(js_code.contains("i++;"));
        assert!(!js_code.contains("(function()")); // No IIFE wrapper

        let test_code = format!("{}\n{}", setup_mock_dom(), js_code);
        assert!(is_valid_js(&test_code));
    }
    /* FIXME: fails
        #[test]
        fn test_for_loop_no_iife_wrapper() {
            // Test that for loop bodies don't get wrapped in IIFE
            let block: Block = parse_quote! {
                {
                    let mut sum = 0;
                    for i in [1, 2, 3, 4, 5] {
                        if i == 3 {
                            sum = sum;
                        } else {
                            sum = sum + i;
                        }
                    }
                    sum
                }
            };

            let js_code = rust_block_to_js(&block);
            println!("DEBUG for loop no IIFE: {}", &js_code);

            // Should not contain IIFE in the loop body
            let lines: Vec<&str> = js_code.lines().collect();
            let for_line_index = lines.iter().position(|line| line.contains("for ("));

            if let Some(index) = for_line_index {
                let remaining_lines = &lines[index..];
                let for_section = remaining_lines.join("\n");

                // The for loop section should not contain IIFE pattern
                assert!(!for_section.contains("return (function()"));
                assert!(!for_section.contains("})();"));
            }

            // Should be valid JS and compute correct result
            let result = eval_block_as_function(&js_code).unwrap();
            assert_eq!(result.as_number().unwrap(), 12.0); // 1+2+4+5
        }
    */

    #[test]
    fn test_dom_methods_return_arrays() {
        // Test that DOM methods work with array methods
        let block: Block = parse_quote! {
            {
                let elements = document.getElementsByTagName("p");
                elements.len()
            }
        };

        let js_code = rust_block_to_js(&block);
        println!("DEBUG DOM array methods: {}", &js_code);

        // Should convert .len() to .length
        assert!(js_code.contains(".length"));

        let test_code = format!("{}\n{}", setup_mock_dom(), js_code);
        let result = eval_block_as_function(&test_code).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0); // Mock has 2 P elements
    }

    #[test]
    fn test_iter_method_removed() {
        // Test that .iter() is properly removed/handled
        let expr: Expr = parse_quote! {
            elements.iter()
        };

        let js_code = rust_expr_to_js(&expr);
        println!("DEBUG iter removal: {}", &js_code);

        // Should not contain .iter()
        assert!(!js_code.contains(".iter()"));
        // Should just be the elements
        assert_eq!(js_code, "elements");
    }

    /* FIXME - fails
        #[test]
        fn test_complex_dom_manipulation() {
            // Test a more complex DOM manipulation scenario
            let block: Block = parse_quote! {
                {
                    let new_element = document.createElement("div");
                    let elements = document.getElementsByTagName("p");
                    let mut count = 0;
                    for (i, element) in elements.iter().enumerate() {
                        if element.tagName == "P" {
                            count = count + 1;
                        }
                    }
                    count
                }
            };

            let js_code = rust_block_to_js(&block);
            println!("DEBUG complex DOM: {}", &js_code);

            let test_code = format!("{}\n{}", setup_mock_dom(), js_code);
            assert!(is_valid_js(&test_code));

            let result = eval_block_as_function(&test_code).unwrap();
            assert_eq!(result.as_number().unwrap(), 2.0); // Should count 2 P elements
        }
    */

    #[test]
    fn test_closure_with_parameters() {
        // Test closures with parameters don't get IIFE wrapped
        let expr: Expr = parse_quote! {
            elements.addEventListener("click", |event| {
                console.log("Clicked!");
                event.preventDefault();
            })
        };

        let js_code = rust_expr_to_js(&expr);
        println!("DEBUG closure with params: {}", &js_code);

        // Should not contain IIFE
        assert!(!js_code.contains("(function()"));
        assert!(!js_code.contains("})()"));

        // Should contain proper arrow function with parameter
        assert!(js_code.contains("event => {"));
        assert!(js_code.contains("console.log"));
        assert!(js_code.contains("event.preventDefault"));
    }

    #[test]
    fn test_nested_for_loops() {
        // Test nested for loops don't interfere with each other
        let block: Block = parse_quote! {
            {
                let outer = [1, 2];
                let inner = [3, 4];
                let mut sum = 0;
                for i in outer {
                    for j in inner {
                        sum = sum + i + j;
                    }
                }
                sum
            }
        };

        let js_code = rust_block_to_js(&block);
        println!("DEBUG nested loops: {}", &js_code);

        // Should not contain IIFE wrappers in loop bodies
        assert!(!js_code.contains("return (function()"));

        let result = eval_block_as_function(&js_code).unwrap();
        // (1+3) + (1+4) + (2+3) + (2+4) = 4 + 5 + 5 + 6 = 20
        assert_eq!(result.as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_let_mut_in_loops() {
        // Test that let mut statements work properly in loops
        let block: Block = parse_quote! {
            {
                for i in [1, 2, 3] {
                    let mut temp = i * 2;
                    temp = temp + 1;
                }
            }
        };

        let js_code = rust_block_to_js(&block);
        println!("DEBUG let mut in loops: {}", &js_code);

        // Should contain let declarations
        assert!(js_code.contains("let temp"));

        // Should be valid JS
        assert!(is_valid_js(&js_code));
    }

    #[test]
    fn test_format_macro_in_loop() {
        // Test that format! macro works properly in loop context
        let block: Block = parse_quote! {
            {
                let elements = ["div", "p", "span"];
                for (i, element) in elements.iter().enumerate() {
                    console.log(&format!("Index {}: {}", i, element));
                }
            }
        };

        let js_code = rust_block_to_js(&block);
        println!("DEBUG format in loop: {}", &js_code);

        // Should contain template literal
        assert!(js_code.contains("${"));
        assert!(js_code.contains("Index"));

        // Should not have IIFE wrapper
        assert!(!js_code.contains("return (function()"));

        let test_code = format!("{}\n{}", setup_mock_dom(), js_code);
        assert!(is_valid_js(&test_code));
    }
}
