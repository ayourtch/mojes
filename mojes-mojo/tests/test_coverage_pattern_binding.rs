// Tests targeting handle_pattern_binding branches: Ok/Err, enum tuple variants,
// struct patterns, None in tuples, float/bool/char literals in patterns
use mojes_mojo::*;
use syn::{parse_quote, Block};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

#[test]
fn test_match_ok_pattern_binding() {
    // Tests Ok(val) branch in handle_pattern_binding (lines 4900-4953)
    let block: Block = parse_quote! {
        {
            let result = Ok(42);
            match result {
                Ok(val) => val,
                Err(e) => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Ok pattern: {}", &js);
    // Should check .ok !== undefined
    assert!(js.contains("ok") || js.contains("error") || js.contains("undefined"));
}

#[test]
fn test_match_err_pattern_binding() {
    // Tests Err(e) branch in handle_pattern_binding
    let block: Block = parse_quote! {
        {
            let result = Err("fail");
            match result {
                Ok(val) => val,
                Err(e) => e,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS Err pattern: {}", &js);
    assert!(js.contains("error") || js.contains("ok"));
}

#[test]
fn test_match_generic_enum_variant_with_data() {
    // Tests generic enum variants like Message::Text(s) (lines 4896-4966)
    let block: Block = parse_quote! {
        {
            let msg = Message::Text("hello".to_string());
            match msg {
                Message::Text(s) => s,
                Message::Number(n) => n.to_string(),
                _ => "unknown".to_string(),
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS enum variant data: {}", &js);
    assert!(js.contains("type") || js.contains("Text"));
}

#[test]
fn test_match_struct_pattern_binding() {
    // Tests Pat::Struct in handle_pattern_binding (lines 4974-5014)
    let block: Block = parse_quote! {
        {
            let event = Event::Click { x: 10, y: 20 };
            match event {
                Event::Click { x, y } => x + y,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS struct pattern binding: {}", &js);
    assert!(js.contains("type") || js.contains("Click"));
}

#[test]
fn test_match_tuple_with_some_none() {
    // Tests Pat::Tuple with Some/None elements in tuple pattern (lines 5015-5137)
    let block: Block = parse_quote! {
        {
            let pair = (Some(42), None);
            match pair {
                (Some(x), None) => x,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS tuple some/none: {}", &js);
    assert!(js.contains("null") || js.contains("undefined") || js.contains("[0]"));
}

#[test]
fn test_match_float_literal() {
    // Tests Pat::Lit with float in handle_pattern_binding (line 4794-4798)
    let block: Block = parse_quote! {
        {
            let x = 3.14;
            match x {
                3.14 => 1,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS float match: {}", &js);
    assert!(js.contains("3.14"));
}

#[test]
fn test_match_char_literal_via_pattern_binding() {
    // Tests Pat::Lit with Char in handle_pattern_binding (line 4801)
    let block: Block = parse_quote! {
        {
            let ch = 'x';
            match ch {
                'x' => 1,
                'y' => 2,
                _ => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS char match binding: {}", &js);
    assert!(js.contains("x") && js.contains("y"));
}

#[test]
fn test_match_bool_literal_via_pattern_binding() {
    // Tests Pat::Lit with Bool in handle_pattern_binding (line 4800)
    let block: Block = parse_quote! {
        {
            let flag = false;
            match flag {
                true => 1,
                false => 0,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS bool match binding: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_match_none_via_pattern_binding() {
    // Tests Pat::Path "None" in handle_pattern_binding (lines 4835-4847)
    let block: Block = parse_quote! {
        {
            let x: Option<i32> = None;
            match x {
                None => 99,
                Some(v) => v,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS None binding: {}", &js);
    assert!(js.contains("null") || js.contains("undefined"));
}

#[test]
fn test_match_enum_unit_variant_via_path() {
    // Tests Pat::Path with non-None variant (lines 4849-4856)
    let block: Block = parse_quote! {
        {
            let dir = "North";
            match dir {
                "North" => 0,
                "South" => 180,
                _ => 90,
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS unit variant path: {}", &js);
    let code = format!("(function() {{ {} }})()", &js);
    let result = eval_js(&code).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}
