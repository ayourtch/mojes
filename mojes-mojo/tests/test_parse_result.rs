// str.parse() transpiles to a Result-shaped value ({ok: n} / {error: msg}),
// the same representation Ok()/Err() use, so every consumption style works:
// match with Ok/Err arms, if-let, .unwrap(), .unwrap_or(), .is_ok(), .ok().
// Each case is executed under Boa and compared against the Rust semantics.

use mojes_mojo::*;
use syn::{parse_quote, Block};

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
fn parse_result_matches_ok_and_err_arms() {
    let ok_case: Block = parse_quote!({
        let text = "123";
        match text.parse() {
            Ok(n) => {
                return n;
            }
            Err(_e) => {
                return -1;
            }
        }
    });
    assert_eq!(as_num(&eval_block_returning(&ok_case)), 123.0);

    let err_case: Block = parse_quote!({
        let text = "not a number";
        match text.parse() {
            Ok(n) => {
                return n;
            }
            Err(_e) => {
                return -1;
            }
        }
    });
    assert_eq!(as_num(&eval_block_returning(&err_case)), -1.0);
}

#[test]
fn parse_unwrap_yields_the_number() {
    let b: Block = parse_quote!({
        let text = "42";
        let n: u32 = text.parse().unwrap();
        return n + 1;
    });
    assert_eq!(as_num(&eval_block_returning(&b)), 43.0);
}

#[test]
fn parse_unwrap_or_takes_default_on_garbage_and_empty() {
    let garbage: Block = parse_quote!({
        let n: u32 = "nope".parse().unwrap_or(7);
        return n;
    });
    assert_eq!(as_num(&eval_block_returning(&garbage)), 7.0);

    // Number("") is 0 in JS, but Rust's "".parse() is an Err.
    let empty: Block = parse_quote!({
        let n: u32 = "".parse().unwrap_or(7);
        return n;
    });
    assert_eq!(as_num(&eval_block_returning(&empty)), 7.0);
}

#[test]
fn parse_is_ok_is_err_and_ok_adapter() {
    let b: Block = parse_quote!({
        let good = "5".parse::<i32>();
        let bad = "x".parse::<i32>();
        if good.is_ok() && bad.is_err() && good.ok() == Some(5) && bad.ok() == None {
            return "all-good";
        }
        return "broken";
    });
    assert_eq!(as_str(&eval_block_returning(&b)), "all-good");
}

#[test]
fn unwrap_still_passes_plain_values_through() {
    // Non-Result receivers (Options, plain JS objects) are unchanged.
    let b: Block = parse_quote!({
        let opt = Some(11);
        let v = opt.unwrap();
        return v;
    });
    assert_eq!(as_num(&eval_block_returning(&b)), 11.0);
}
