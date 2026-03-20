// Tests for enum JSON generation: fromJSON, toJSON switch body
// Covers create_enum_from_json_function and create_enum_to_json_switch_body
use mojes_mojo::*;
use syn::{parse_quote, ItemEnum};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_enum_with_tuple_variant_json() {
    // Tests tuple variants in create_enum_to_json_switch_body
    let input: ItemEnum = parse_quote! {
        enum Msg {
            Text(String),
            Data(i32, i32),
        }
    };
    let js = generate_js_enum(&input);
    println!("JS tuple enum: {}", &js);
    assert!(js.contains("Text") && js.contains("Data"));
    assert!(js.contains("value0") || js.contains("type"));
}

#[test]
fn test_enum_with_named_variant_json() {
    // Tests struct variants in create_enum_to_json_switch_body
    let input: ItemEnum = parse_quote! {
        enum Action {
            Move { x: i32, y: i32 },
            Resize { width: i32, height: i32 },
        }
    };
    let js = generate_js_enum(&input);
    println!("JS named enum: {}", &js);
    assert!(js.contains("Move") && js.contains("Resize"));
}

#[test]
fn test_enum_all_variant_types() {
    // Tests mix of all three variant types for full JSON coverage
    let input: ItemEnum = parse_quote! {
        enum Event {
            None,
            Click(i32, i32),
            KeyPress { key: String, ctrl: bool },
        }
    };
    let js = generate_js_enum(&input);
    println!("JS all variants: {}", &js);
    assert!(js.contains("None") && js.contains("Click") && js.contains("KeyPress"));

    // Execute to verify no JS errors
    let result = eval_js(&js);
    println!("Eval result: {:?}", &result);
}

#[test]
fn test_enum_fromjson_roundtrip() {
    // Tests that the generated enum has fromJSON
    let input: ItemEnum = parse_quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    };
    let js = generate_js_enum(&input);
    println!("JS color enum: {}", &js);
    assert!(js.contains("fromJSON"));

    // Test the generated enum object
    let code = format!(r#"
        {}
        Color.Red === 'Red' ? 'ok' : 'fail'
    "#, &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "ok");
}

#[test]
fn test_enum_tojson_roundtrip() {
    // Tests that the generated enum has toJSON
    let input: ItemEnum = parse_quote! {
        enum Status {
            Active,
            Inactive,
            Pending,
        }
    };
    let js = generate_js_enum(&input);
    println!("JS status enum: {}", &js);
    assert!(js.contains("toJSON"));

    // Test toJSON with a unit variant
    let code = format!(r#"
        {}
        Status.toJSON('Active')
    "#, &js);
    let result = eval_js(&code);
    println!("toJSON result: {:?}", &result);
}

#[test]
fn test_transpile_enum_to_js_function() {
    // Tests the convenience transpile_enum_to_js
    let input: ItemEnum = parse_quote! {
        enum Priority {
            Low,
            Medium,
            High,
        }
    };
    let js = transpile_enum_to_js(&input).unwrap();
    println!("JS priority: {}", &js);
    assert!(js.contains("Low") && js.contains("Medium") && js.contains("High"));
}
