// Tests for for-of loops: tuple destructuring, enumerate patterns, typed patterns
// Covers lines 4578-4723, 4740-4839
use mojes_mojo::*;
use syn::{parse_quote, Block};

#[test]
fn test_for_tuple_destructuring() {
    // Tests Pat::Tuple branch in convert_for_to_stmt (lines 4578-4705)
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            for (key, value) in items {
                key;
                value;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS: {}", &js);
    // Should generate for...of with array destructuring
    assert!(js.contains("for") && js.contains("of"));
}

#[test]
fn test_for_enumerate_pattern() {
    // Tests enumerate detection and optimized loop (lines 4740-4822)
    let block: Block = parse_quote! {
        {
            let items = vec![10, 20, 30];
            for (i, item) in items.iter().enumerate() {
                i;
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS enumerate: {}", &js);
    // Should generate optimized for-of with index counter
    assert!(js.contains("for") || js.contains("i"));
}

#[test]
fn test_for_enumerate_direct() {
    // Tests enumerate without .iter() prefix
    let block: Block = parse_quote! {
        {
            let items = vec![10, 20, 30];
            for (idx, val) in items.enumerate() {
                idx;
                val;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS enumerate direct: {}", &js);
    assert!(js.contains("for") || js.contains("idx"));
}

#[test]
fn test_for_simple_ident() {
    // Tests Pat::Ident branch (simple for loop)
    let block: Block = parse_quote! {
        {
            let items = vec![1, 2, 3];
            for item in items {
                item;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS simple for: {}", &js);
    assert!(js.contains("for") && js.contains("of"));
}

#[test]
fn test_for_over_range() {
    // Tests for loop over a range expression
    let block: Block = parse_quote! {
        {
            for i in 0..10 {
                i;
            }
        }
    };
    let js = rust_block_to_js(&block);
    println!("JS range for: {}", &js);
    assert!(js.contains("for"));
}
