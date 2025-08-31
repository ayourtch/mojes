use mojes_mojo::*;
use syn::parse_quote;

fn main() {
    println!("Testing HashMap.len() and Array.len() transpilation...\n");
    
    // Test HashMap.len()
    let hashmap_code = parse_quote! {
        impl Test {
            fn count_connections(&self) -> usize {
                self.peer_connections.len()
            }
        }
    };
    
    let js_hashmap = generate_js_methods_for_impl(&hashmap_code);
    println!("HashMap.len() transpilation:");
    println!("{}", js_hashmap);
    
    // Test Array.len()
    let array_code = parse_quote! {
        impl Test {
            fn count_items(&self) -> usize {
                self.items.len()
            }
        }
    };
    
    let js_array = generate_js_methods_for_impl(&array_code);
    println!("\nArray.len() transpilation:");
    println!("{}", js_array);
    
    // Test String.len()
    let string_code = parse_quote! {
        impl Test {
            fn get_name_length(&self) -> usize {
                self.name.len()
            }
        }
    };
    
    let js_string = generate_js_methods_for_impl(&string_code);
    println!("\nString.len() transpilation:");
    println!("{}", js_string);
    
    // Check that the universal solution is generated
    if js_hashmap.contains("Object.keys") && js_hashmap.contains("!== undefined") {
        println!("\n✅ Universal len() solution detected for HashMap!");
    } else {
        println!("\n❌ Universal len() solution NOT detected!");
    }
    
    if js_array.contains("Object.keys") && js_array.contains("!== undefined") {
        println!("✅ Universal len() solution detected for Array!");
    } else {
        println!("❌ Universal len() solution NOT detected for Array!");
    }
}