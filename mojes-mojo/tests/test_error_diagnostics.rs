//! The fallible transpilation entry points must produce actionable errors:
//! they name the offending Rust source and give a clear reason, instead of
//! panicking with an opaque message.

use mojes_mojo::{source_snippet, try_rust_block_to_js, try_rust_expr_to_js};
use syn::parse_quote;

#[test]
fn unsupported_macro_error_names_macro_and_source() {
    let b: syn::Block = parse_quote!({
        let x = 1;
        totally_unknown_macro!(a, b);
    });
    let err = try_rust_block_to_js(&b).unwrap_err();
    assert!(err.contains("totally_unknown_macro"), "names the macro: {err}");
    assert!(err.contains("Unsupported macro"), "explains the problem: {err}");
    // includes the offending statement source
    assert!(err.contains("in `"), "carries source context: {err}");
}

#[test]
fn wrong_arity_method_error_has_source_context() {
    let b: syn::Block = parse_quote!({
        map.remove(a, b);
    });
    let err = try_rust_block_to_js(&b).unwrap_err();
    assert!(err.contains("remove()"), "explains the problem: {err}");
    assert!(err.contains("map . remove"), "carries source context: {err}");
}

#[test]
fn nested_struct_gives_friendly_guidance() {
    let b: syn::Block = parse_quote!({
        struct Inner {
            x: i32,
        }
        let y = 1;
    });
    let err = try_rust_block_to_js(&b).unwrap_err();
    assert!(err.contains("nested `struct`"), "{err}");
    assert!(err.contains("#[js_type]"), "suggests the fix: {err}");
}

#[test]
fn valid_code_still_transpiles_without_error() {
    let b: syn::Block = parse_quote!({
        let z = format!("{}", 1);
    });
    assert!(try_rust_block_to_js(&b).is_ok());
    let e: syn::Expr = parse_quote!(1 + 2);
    assert_eq!(try_rust_expr_to_js(&e).unwrap().trim(), "1 + 2");
}

#[test]
fn source_snippet_is_compact_and_truncated() {
    let e: syn::Expr = parse_quote!(a.b().c().d());
    let s = source_snippet(&e);
    assert!(!s.contains('\n'));
    let big: syn::Expr = parse_quote!(
        aaaaaaaaaa + bbbbbbbbbb + cccccccccc + dddddddddd + eeeeeeeeee + ffffffffff
            + gggggggggg + hhhhhhhhhh + iiiiiiiiii + jjjjjjjjjj + kkkkkkkkkk + llllllllll
    );
    assert!(source_snippet(&big).ends_with('…'));
}
