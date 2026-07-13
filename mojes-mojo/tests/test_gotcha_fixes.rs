// Fixes for the README "Gotchas" - each one executed under Boa where
// possible, string-asserted where async execution is impractical:
//   1. match / if-let around .await: arm IIFEs become async AND awaited,
//      propagating async-ness outward (no stray Promises in match results).
//   2. Struct literals no longer depend on field declaration order - they
//      construct via named assignments instead of positional arguments.
//   3. String::new() -> "" (a primitive), not `new String()` (a boxed object).
//   4. if/else inside format! returns its value (the _rust_retval pattern).
//   5. `continue` inside a for-with-enumerate loop no longer freezes the
//      index (increment is at the top of the body).

use mojes_mojo::*;
use syn::{parse_quote, Block, Expr};

fn eval_js(code: &str) -> boa_engine::JsResult<boa_engine::JsValue> {
    let mut context = boa_engine::Context::default();
    context.eval(boa_engine::Source::from_bytes(code))
}

fn eval_block_returning(b: &Block) -> boa_engine::JsValue {
    let js = rust_block_to_js(b);
    eval_js(&format!("(function() {{ {} }})()", js)).expect("JS execution failed")
}

fn as_num(v: &boa_engine::JsValue) -> f64 {
    v.as_number().expect("expected a number")
}

fn as_str(v: &boa_engine::JsValue) -> String {
    let mut ctx = boa_engine::Context::default();
    v.to_string(&mut ctx).unwrap().to_std_string().unwrap()
}

#[test]
fn match_around_await_is_awaited() {
    let b: Block = parse_quote!({
        match opt {
            Some(x) => {
                let r = do_thing(x).await;
                r
            }
            None => "none".to_string(),
        }
    });
    let js = rust_block_to_js(&b);
    // The arm containing .await is an async IIFE and is awaited...
    assert!(
        js.contains("await (async function()"),
        "async arm IIFE not awaited:\n{js}"
    );
    // ...and the await propagates: the enclosing match IIFE is async+awaited
    // too, so the value (not a Promise) flows out.
    assert_eq!(
        js.matches("await (async function()").count(),
        2,
        "async-ness did not propagate to the outer match IIFE:\n{js}"
    );
    // Still parses as a valid async function body.
    eval_js(&format!(
        "async function f(opt, do_thing) {{ {} }} f",
        js
    ))
    .expect("generated JS is not valid inside an async function");
}

#[test]
fn struct_literal_field_order_does_not_matter() {
    // P's constructor is positional (a, b); the literal lists b first.
    let decl = r#"
        class P {
            constructor(a, b) { this.a = a; this.b = b; }
        }
    "#;
    let b: Block = parse_quote!({
        let b = "hi".to_string();
        let p = P { b, a: 1 };
        return format!("{}/{}", p.a, p.b);
    });
    let js = rust_block_to_js(&b);
    assert!(
        !js.contains("new P(b, 1)") && !js.contains("new P(\"hi\""),
        "struct literal still constructs positionally:\n{js}"
    );
    let result = eval_js(&format!("{decl}\n(function() {{ {js} }})()"))
        .expect("JS execution failed");
    assert_eq!(as_str(&result), "1/hi");
}

#[test]
fn string_new_is_a_primitive() {
    let b: Block = parse_quote!({
        let s = String::new();
        if s == "" {
            return "primitive";
        }
        return "boxed";
    });
    assert_eq!(as_str(&eval_block_returning(&b)), "primitive");
}

#[test]
fn format_with_if_expression_returns_value() {
    let b: Block = parse_quote!({
        let x = true;
        return format!("val: {}", if x { "YES" } else { "NO" });
    });
    assert_eq!(as_str(&eval_block_returning(&b)), "val: YES");
}

#[test]
fn sequential_loops_reusing_a_variable_name() {
    // Two loops binding the same name: the second gets shadow-renamed
    // (id_1), and its body must reference the RENAMED variable. A body
    // transpiled before the loop variable was declared referenced the stale
    // outer `id` -> "ReferenceError: id is not defined" at runtime (seen in
    // the wild in mojes-conf's draw_composite).
    let b: Block = parse_quote!({
        let mut acc = "".to_string();
        let pairs = vec![("a", 1), ("b", 2)];
        for (id, _n) in pairs.iter().enumerate() {
            acc += &format!("{}", id);
        }
        let names = vec!["x".to_string(), "y".to_string()];
        for id in names {
            acc += &id;
        }
        return acc;
    });
    let js = rust_block_to_js(&b);
    let v = eval_block_returning(&b);
    assert_eq!(as_str(&v), "01xy", "generated JS:\n{js}");
}

#[test]
fn continue_does_not_freeze_enumerate_index() {
    let b: Block = parse_quote!({
        let mut sum = 0;
        let items = vec![10, 20, 30];
        for (idx, v) in items.iter().enumerate() {
            if idx == 1 {
                continue;
            }
            sum += v;
        }
        return sum;
    });
    // Rust: skips only index 1 -> 10 + 30 = 40. The old codegen put idx++
    // at the END of the body, so `continue` froze the index at 1 and
    // every later element was skipped (sum stayed 10).
    assert_eq!(as_num(&eval_block_returning(&b)), 40.0);
}
