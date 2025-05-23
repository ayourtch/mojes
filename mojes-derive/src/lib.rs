use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ItemImpl;
use syn::{ItemEnum, ItemFn, ItemStruct, Pat, parse_macro_input};

use mojes_mojo::format_rust_type;
use mojes_mojo::generate_js_class_for_struct;
use mojes_mojo::generate_js_enum;
use mojes_mojo::rust_block_to_js;

#[proc_macro_attribute]
pub fn impl_to_js(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_impl = parse_macro_input!(item as ItemImpl);

    // Create the debug string
    let js_debug = format!("/* {:#?} */", &input_impl);

    // Generate the output
    let output = quote! {
        #input_impl

        static TEST: &str = #js_debug;
    };

    output.into()
}

#[proc_macro_attribute]
pub fn to_js(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Get function name
    let fn_name = &input_fn.sig.ident;
    let js_fn_name = fn_name.to_string();

    // Extract function arguments with their types
    let args: Vec<(String, Option<String>)> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let arg_name = pat_ident.ident.to_string();
                    let arg_type = format_rust_type(&pat_type.ty);
                    return Some((arg_name, Some(arg_type)));
                }
            }
            None
        })
        .collect();

    // Join arguments with commas for JS function signature
    let js_args = args
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    // Add type validation if types are available
    let mut js_body = String::new();
    for (arg_name, arg_type_opt) in &args {
        if let Some(arg_type) = arg_type_opt {
            // Only add type validation for basic types
            match arg_type.as_str() {
                "number" | "string" | "boolean" => {
                    js_body.push_str(&format!("  // Type validation for {}\n", arg_name));
                    js_body.push_str(&format!(
                        "  if (typeof {} !== '{}') {{\n",
                        arg_name, arg_type
                    ));
                    js_body.push_str(&format!(
                        "    throw new TypeError('Expected {} to be of type {}, got ' + typeof {});\n",
                        arg_name, arg_type, arg_name
                    ));
                    js_body.push_str("  }\n");
                }
                "Array" => {
                    js_body.push_str(&format!("  // Type validation for {}\n", arg_name));
                    js_body.push_str(&format!("  if (!Array.isArray({})) {{\n", arg_name));
                    js_body.push_str(&format!(
                        "    throw new TypeError('Expected {} to be an Array, got ' + typeof {});\n",
                        arg_name, arg_name
                    ));
                    js_body.push_str("  }\n");
                }
                _ => {
                    // Skip complex types for now
                }
            }
        }
    }

    // Convert function body to JavaScript
    js_body.push_str(&rust_block_to_js(&input_fn.block));

    // Create the JavaScript function string
    let js_function = format!("function {}({}) {{\n{}}}", js_fn_name, js_args, js_body);

    // Create a string constant with the JavaScript function
    let js_const_name = format_ident!("{}_JS", fn_name.to_string().to_uppercase());

    // Generate the output with proper distributed_slice syntax
    let output = quote! {
        #input_fn

        #[linkme::distributed_slice(crate::JS)]
        static #js_const_name: &str = #js_function;
    };

    output.into()
}

// New procedural macro for structs
#[proc_macro_attribute]
pub fn js_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();

    // Try to parse as struct or enum
    if let Ok(input_struct) = syn::parse::<ItemStruct>(input.clone()) {
        let struct_name = input_struct.ident.to_string();
        let js_class = generate_js_class_for_struct(&input_struct);

        let js_const_name = format_ident!("{}_JS_CLASS", struct_name.to_uppercase());

        let output = quote! {
            #input_struct

            #[linkme::distributed_slice(crate::JS)]
            static #js_const_name: &str = #js_class;
        };

        return output.into();
    }

    // Try as enum
    if let Ok(input_enum) = syn::parse::<ItemEnum>(input.clone()) {
        let enum_name = input_enum.ident.to_string();
        let js_enum = generate_js_enum(&input_enum);

        let js_const_name = format_ident!("{}_JS_ENUM", enum_name.to_uppercase());

        let output = quote! {
            #input_enum

            #[linkme::distributed_slice(crate::JS)]
            static #js_const_name: &str = #js_enum;
        };

        return output.into();
    }

    // If we get here, it's neither a struct nor an enum
    let error = syn::Error::new_spanned(
        proc_macro2::TokenStream::from(item.clone()),
        "js_type can only be applied to structs or enums",
    );

    error.to_compile_error().into()
}
