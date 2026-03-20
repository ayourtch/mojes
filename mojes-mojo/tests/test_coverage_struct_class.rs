// Tests for struct-to-class generation and impl block transpilation
// Covers generate_js_class_for_struct, generate_js_methods_for_impl
use mojes_mojo::*;
use syn::{parse_quote, ItemImpl, ItemStruct};

#[test]
fn test_basic_struct_to_class() {
    let input: ItemStruct = parse_quote! {
        struct Point {
            x: f64,
            y: f64,
        }
    };
    let js = generate_js_class_for_struct(&input);
    println!("JS: {}", &js);
    assert!(js.contains("class Point"));
    assert!(js.contains("constructor"));
    assert!(js.contains("x") && js.contains("y"));
}

#[test]
fn test_struct_with_string_fields() {
    let input: ItemStruct = parse_quote! {
        struct Person {
            name: String,
            age: i32,
        }
    };
    let js = generate_js_class_for_struct(&input);
    println!("JS: {}", &js);
    assert!(js.contains("class Person"));
    assert!(js.contains("name") && js.contains("age"));
}

#[test]
fn test_impl_block_methods() {
    let input: ItemImpl = parse_quote! {
        impl Calculator {
            fn new(initial: i32) -> Self {
                Self { value: initial }
            }

            fn add(&self, n: i32) -> i32 {
                self.value + n
            }

            fn reset(&mut self) {
                self.value = 0;
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Calculator"));
    assert!(js.contains("add") || js.contains("reset"));
}

#[test]
fn test_impl_with_static_method() {
    let input: ItemImpl = parse_quote! {
        impl Config {
            fn default() -> Self {
                Self { debug: false }
            }
        }
    };
    let js = generate_js_methods_for_impl(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Config"));
}

#[test]
fn test_struct_single_field() {
    let input: ItemStruct = parse_quote! {
        struct Wrapper {
            value: i32,
        }
    };
    let js = generate_js_class_for_struct(&input);
    println!("JS: {}", &js);
    assert!(js.contains("class Wrapper"));
    assert!(js.contains("value"));
}
