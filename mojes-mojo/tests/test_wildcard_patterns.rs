use mojes_mojo::*;
use syn::parse_quote;

fn main() {
    println!("Testing wildcard patterns in tuple struct destructuring...\n");

    // Test enum with wildcard pattern
    let match_expr = parse_quote! {
        match result {
            Ok(value) => value,
            Err(_) => "error occurred",  // Wildcard pattern should work
        }
    };

    let js_code = rust_expr_to_js(&match_expr);
    println!("Enum wildcard pattern result:");
    println!("{}", js_code);
    
    // Test more complex wildcard pattern
    let match_expr2 = parse_quote! {
        match data {
            Some(value, _) => value,  // Mixed pattern with wildcard
            None => "nothing",
        }
    };
    
    let js_code2 = rust_expr_to_js(&match_expr2);
    println!("\nMixed wildcard pattern result:");
    println!("{}", js_code2);

    println!("\n✅ Wildcard patterns should now work!");
}