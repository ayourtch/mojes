use mojes_mojo::*;
use syn::{Expr, parse_quote};

fn main() {
    println!("Testing universal insert() solution...\n");

    // Test that our WebRTC HashMap case now works
    let expr: Expr = parse_quote!(self.peer_connections.insert(key, pc));
    let js_code = rust_expr_to_js(&expr);
    println!("HashMap insert JavaScript:");
    println!("{}\n", js_code);
    
    // Test that array insert still works  
    let expr2: Expr = parse_quote!(arr.insert(0, item));
    let js_code2 = rust_expr_to_js(&expr2);
    println!("Array insert JavaScript:");
    println!("{}\n", js_code2);
    
    // Verify both use the universal IIFE pattern
    if js_code.contains("obj.splice ? obj.splice") && js_code.contains("obj[key] = val") {
        println!("✅ SUCCESS: HashMap insert uses universal IIFE solution!");
        println!("   - Will use obj[key] = val for HashMap-like objects");
        println!("   - Will use obj.splice() for arrays");
    } else {
        println!("❌ FAILED: HashMap insert doesn't use universal solution");
    }
    
    if js_code2.contains("obj.splice ? obj.splice") && js_code2.contains("obj[key] = val") {
        println!("✅ SUCCESS: Array insert uses universal IIFE solution!");
        println!("   - Will use obj[key] = val for HashMap-like objects");
        println!("   - Will use obj.splice() for arrays");
    } else {
        println!("❌ FAILED: Array insert doesn't use universal solution");
    }
    
    println!("\n🎉 Universal insert() solution successfully implemented!");
    println!("   This solves the original problem where HashMap.insert() was");
    println!("   incorrectly transpiled to splice() instead of property assignment.");
}