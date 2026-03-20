// Tests for impl block transpilation with various method types
// Covers generate_js_methods_for_impl_with_state, generate_js_method, handle_function_definition
use mojes_mojo::*;
use syn::{parse_quote, ItemImpl, ItemStruct};

#[test]
fn test_impl_self_constructor() {
    // Tests Self {} expression in static method context
    let input: ItemImpl = parse_quote! {
        impl MyStruct {
            fn new(x: i32, y: i32) -> Self {
                Self { x, y }
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS constructor: {}", &js);
    assert!(js.contains("MyStruct"));
}

#[test]
fn test_impl_instance_method_with_self() {
    // Tests &self method transpilation
    let input: ItemImpl = parse_quote! {
        impl Counter {
            fn increment(&mut self) {
                self.count = self.count + 1;
            }

            fn get_count(&self) -> i32 {
                self.count
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS self methods: {}", &js);
    assert!(js.contains("Counter") && js.contains("prototype"));
}

#[test]
fn test_impl_with_multiple_params() {
    let input: ItemImpl = parse_quote! {
        impl Calculator {
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            fn multiply(&self, a: i32, b: i32) -> i32 {
                a * b
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS multi params: {}", &js);
    assert!(js.contains("Calculator"));
}

#[test]
fn test_struct_with_many_fields() {
    let input: ItemStruct = parse_quote! {
        struct Config {
            debug: bool,
            verbose: bool,
            max_retries: i32,
            timeout: f64,
            name: String,
        }
    };
    let js = generate_js_class_for_struct(&input);
    println!("JS multi-field struct: {}", &js);
    assert!(js.contains("class Config"));
    assert!(js.contains("constructor"));
    assert!(js.contains("debug") && js.contains("verbose"));
}

#[test]
fn test_transpile_impl_to_js() {
    // Tests the transpile_impl_to_js convenience function
    let input: ItemImpl = parse_quote! {
        impl Adder {
            fn add(&self, n: i32) -> i32 {
                self.value + n
            }
        }
    };
    let js = transpile_impl_to_js(&input).unwrap();
    println!("JS transpile_impl: {}", &js);
    assert!(js.contains("Adder") && js.contains("add"));
}

#[test]
fn test_transpile_struct_to_js() {
    // Tests the transpile_struct_to_js convenience function
    let input: ItemStruct = parse_quote! {
        struct SimpleStruct {
            value: i32,
        }
    };
    let js = transpile_struct_to_js(&input).unwrap();
    println!("JS transpile_struct: {}", &js);
    assert!(js.contains("class SimpleStruct"));
}

#[test]
fn test_transpile_enum_to_js() {
    // Tests the transpile_enum_to_js convenience function
    let input: syn::ItemEnum = parse_quote! {
        enum Direction {
            North,
            South,
            East,
            West,
        }
    };
    let js = transpile_enum_to_js(&input).unwrap();
    println!("JS transpile_enum: {}", &js);
    assert!(js.contains("North") && js.contains("South"));
}
