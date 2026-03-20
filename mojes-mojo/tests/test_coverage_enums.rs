// Tests for enum generation: generate_js_enum with unit, tuple, and struct variants
// Covers the big uncovered blocks around lines 6081-6216, 6383-6511
use mojes_mojo::*;
use syn::{parse_quote, ItemEnum};

#[test]
fn test_enum_unit_variants() {
    let input: ItemEnum = parse_quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    };
    let js = generate_js_enum(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Red"));
    assert!(js.contains("Green"));
    assert!(js.contains("Blue"));
}

#[test]
fn test_enum_tuple_variants() {
    // Tests tuple variant handling in create_enum_to_json_method (lines 6101-6133)
    // and create_enum_to_json_standalone_function (lines 6403-6435)
    let input: ItemEnum = parse_quote! {
        enum Message {
            Text(String),
            Number(i32),
        }
    };
    let js = generate_js_enum(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Text"));
    assert!(js.contains("Number"));
    assert!(js.contains("value0") || js.contains("type"));
}

#[test]
fn test_enum_struct_variants() {
    // Tests struct variant handling in create_enum_to_json_method (lines 6135-6170)
    // and create_enum_to_json_standalone_function (lines 6437-6472)
    let input: ItemEnum = parse_quote! {
        enum Event {
            Click { x: i32, y: i32 },
            Scroll { delta: f64 },
        }
    };
    let js = generate_js_enum(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Click"));
    assert!(js.contains("Scroll"));
    // Should have field names
    assert!(js.contains("x") || js.contains("y") || js.contains("delta"));
}

#[test]
fn test_enum_mixed_variants() {
    // Tests combination of unit, tuple, and struct variants
    let input: ItemEnum = parse_quote! {
        enum Shape {
            Circle,
            Rectangle(f64, f64),
            Triangle { base: f64, height: f64 },
        }
    };
    let js = generate_js_enum(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Circle"));
    assert!(js.contains("Rectangle"));
    assert!(js.contains("Triangle"));
}

#[test]
fn test_enum_with_single_variant() {
    let input: ItemEnum = parse_quote! {
        enum SingleVariant {
            Only,
        }
    };
    let js = generate_js_enum(&input);
    println!("JS: {}", &js);
    assert!(js.contains("Only"));
}
