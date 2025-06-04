use std::fs;
use std::io::{self, Write};
use std::path::Path;
use clap::{Arg, Command};
use syn::{parse_file, Item};

// Import from your mojes-mojo crate
use mojes_mojo::{
    generate_js_methods_for_impl_with_state,
    generate_js_class_for_struct_with_state,
    generate_js_enum_with_state,
    ast_to_code,
    TranspilerState,
};

// Boa imports for JavaScript execution
use boa_engine::{Context, Source};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("rust-js-transpiler")
        .version("1.0.0")
        .about("Transpiles Rust code to JavaScript")
        .arg(
            Arg::new("input")
                .help("Input Rust file to transpile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output JavaScript file (if not specified, outputs to stdout)")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .help("Run the transpiled JavaScript code using Boa. Optionally specify a function to call.")
                .value_name("FUNCTION")
                .num_args(0..=1)
                .require_equals(false),
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .help("Pretty print the JavaScript output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output");
    let should_run = matches.contains_id("run");
    let run_function = matches.get_one::<String>("run");
    let pretty_print = matches.get_flag("pretty");

    // Read the Rust source file
    let rust_code = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read input file '{}': {}", input_file, e))?;

    // Parse the Rust code
    let syntax_tree = parse_file(&rust_code)
        .map_err(|e| format!("Failed to parse Rust code: {}", e))?;

    // Transpile to JavaScript
    let js_code = transpile_rust_file(&syntax_tree)?;

    // Add JavaScript runtime helpers
    let final_js_code = add_js_runtime_helpers(&js_code, pretty_print);

    // Output the JavaScript code
    match output_file {
        Some(file_path) => {
            fs::write(file_path, &final_js_code)
                .map_err(|e| format!("Failed to write output file '{}': {}", file_path, e))?;
            println!("Transpiled JavaScript written to: {}", file_path);
        }
        None => {
            if !should_run {
                print!("{}", final_js_code);
            }
        }
    }

    // Run the JavaScript code if requested
    if should_run {
        println!("Running transpiled JavaScript code...\n");
        run_javascript(&final_js_code, run_function)?;
    }

    Ok(())
}

fn transpile_rust_file(syntax_tree: &syn::File) -> Result<String, Box<dyn std::error::Error>> {
    let mut state = TranspilerState::new();
    let mut js_items = Vec::new();

    // Add file header comment
    js_items.push(swc_ecma_ast::ModuleItem::Stmt(
        swc_ecma_ast::Stmt::Expr(swc_ecma_ast::ExprStmt {
            span: swc_common::DUMMY_SP,
            expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident::new(
                "// Transpiled from Rust using mojes-mojo".into(),
                swc_common::DUMMY_SP,
                swc_common::SyntaxContext::empty(),
            ))),
        })
    ));

    // Process each top-level item in the Rust file
    for item in &syntax_tree.items {
        match item {
            Item::Struct(item_struct) => {
                let class_item = generate_js_class_for_struct_with_state(item_struct)
                    .map_err(|e| format!("Failed to transpile struct '{}': {}", item_struct.ident, e))?;
                js_items.push(class_item);
            }
            Item::Enum(item_enum) => {
                let enum_items = generate_js_enum_with_state(item_enum)
                    .map_err(|e| format!("Failed to transpile enum '{}': {}", item_enum.ident, e))?;
                js_items.extend(enum_items);
            }
            Item::Impl(item_impl) => {
                let impl_items = generate_js_methods_for_impl_with_state(item_impl)
                    .map_err(|e| format!("Failed to transpile impl block: {}", e))?;
                js_items.extend(impl_items);
            }
            Item::Fn(item_fn) => {
                // Handle top-level functions
                let js_function = transpile_function(item_fn, &mut state)?;
                js_items.push(js_function);
            }
            Item::Use(_) => {
                // Skip use statements as they don't translate directly to JavaScript
                continue;
            }
            Item::Mod(item_mod) => {
                // Add a comment for module declarations
                let comment = format!("// Module: {}", item_mod.ident);
                js_items.push(swc_ecma_ast::ModuleItem::Stmt(
                    swc_ecma_ast::Stmt::Expr(swc_ecma_ast::ExprStmt {
                        span: swc_common::DUMMY_SP,
                        expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident::new(
                            comment.into(),
                            swc_common::DUMMY_SP,
                            swc_common::SyntaxContext::empty(),
                        ))),
                    })
                ));
            }
            Item::Const(item_const) => {
                // Handle const declarations
                let js_const = transpile_const(item_const, &mut state)?;
                js_items.push(js_const);
            }
            Item::Static(item_static) => {
                // Handle static declarations
                let js_static = transpile_static(item_static, &mut state)?;
                js_items.push(js_static);
            }
            _ => {
                // Add a comment for unsupported items
                let comment = format!("// Unsupported Rust item: {:?}", item);
                js_items.push(swc_ecma_ast::ModuleItem::Stmt(
                    swc_ecma_ast::Stmt::Expr(swc_ecma_ast::ExprStmt {
                        span: swc_common::DUMMY_SP,
                        expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident::new(
                            comment.into(),
                            swc_common::DUMMY_SP,
                            swc_common::SyntaxContext::empty(),
                        ))),
                    })
                ));
            }
        }
    }

    // Convert AST to JavaScript code
    ast_to_code(&js_items)
        .map_err(|e| format!("Failed to generate JavaScript code: {}", e).into())
}

fn transpile_function(
    item_fn: &syn::ItemFn,
    state: &mut TranspilerState,
) -> Result<swc_ecma_ast::ModuleItem, Box<dyn std::error::Error>> {
    use swc_ecma_ast as js;
    use swc_common::{DUMMY_SP, SyntaxContext};
    use mojes_mojo::{rust_block_to_js_with_state, BlockAction, escape_js_identifier};

    let func_name = item_fn.sig.ident.to_string();
    let js_func_name = escape_js_identifier(&func_name);

    // Convert parameters
    let params: Vec<js::Pat> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                syn::FnArg::Receiver(_) => None, // Skip self parameters
                syn::FnArg::Typed(pat_type) => {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let js_param_name = escape_js_identifier(&param_name);
                        Some(js::Pat::Ident(js::BindingIdent {
                            id: js::Ident::new(js_param_name.into(), DUMMY_SP, SyntaxContext::empty()),
                            type_ann: None,
                        }))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Convert function body
    let body_stmts = rust_block_to_js_with_state(BlockAction::Return, &item_fn.block, state)
        .map_err(|e| format!("Failed to transpile function body: {}", e))?;

    let function_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: body_stmts,
        ctxt: SyntaxContext::empty(),
    };

    let function = js::Function {
        params: params.into_iter().map(|p| state.pat_to_param(p)).collect(),
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(function_body),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    };

    // Create function declaration
    let func_decl = js::FnDecl {
        ident: js::Ident::new(js_func_name.into(), DUMMY_SP, SyntaxContext::empty()),
        declare: false,
        function: Box::new(function),
    };

    Ok(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Fn(func_decl))))
}

fn transpile_const(
    item_const: &syn::ItemConst,
    state: &mut TranspilerState,
) -> Result<swc_ecma_ast::ModuleItem, Box<dyn std::error::Error>> {
    use swc_ecma_ast as js;
    use swc_common::{DUMMY_SP, SyntaxContext};
    use mojes_mojo::{rust_expr_to_js_with_state, escape_js_identifier};

    let const_name = item_const.ident.to_string();
    let js_const_name = escape_js_identifier(&const_name);

    let init_expr = rust_expr_to_js_with_state(&item_const.expr, state)
        .map_err(|e| format!("Failed to transpile const expression: {}", e))?;

    let var_decl = js::VarDecl {
        span: DUMMY_SP,
        kind: js::VarDeclKind::Const,
        declare: false,
        decls: vec![js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new(js_const_name.into(), DUMMY_SP, SyntaxContext::empty()),
                type_ann: None,
            }),
            init: Some(Box::new(init_expr)),
            definite: false,
        }],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Var(Box::new(var_decl)))))
}

fn transpile_static(
    item_static: &syn::ItemStatic,
    state: &mut TranspilerState,
) -> Result<swc_ecma_ast::ModuleItem, Box<dyn std::error::Error>> {
    use swc_ecma_ast as js;
    use swc_common::{DUMMY_SP, SyntaxContext};
    use mojes_mojo::{rust_expr_to_js_with_state, escape_js_identifier};

    let static_name = item_static.ident.to_string();
    let js_static_name = escape_js_identifier(&static_name);

    let init_expr = rust_expr_to_js_with_state(&item_static.expr, state)
        .map_err(|e| format!("Failed to transpile static expression: {}", e))?;

    // Use let for mutable statics, const for immutable ones
    let kind = match item_static.mutability {
        syn::StaticMutability::Mut(_) => js::VarDeclKind::Let,
        syn::StaticMutability::None => js::VarDeclKind::Const,
        _ => js::VarDeclKind::Const, // Default to const for unknown variants
    };

    let var_decl = js::VarDecl {
        span: DUMMY_SP,
        kind,
        declare: false,
        decls: vec![js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new(js_static_name.into(), DUMMY_SP, SyntaxContext::empty()),
                type_ann: None,
            }),
            init: Some(Box::new(init_expr)),
            definite: false,
        }],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Var(Box::new(var_decl)))))
}

fn add_js_runtime_helpers(js_code: &str, pretty: bool) -> String {
    let helpers = r#"
// JavaScript Runtime Helpers for Rust transpiled code

// Debug representation function for {:?} formatting
function debug_repr(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return JSON.stringify(value);
    if (typeof value === 'object' && value.constructor && value.constructor.name) {
        if (Array.isArray(value)) {
            return '[' + value.map(debug_repr).join(', ') + ']';
        }
        // For custom objects with toJSON method
        if (typeof value.toJSON === 'function') {
            return value.constructor.name + '(' + debug_repr(value.toJSON()) + ')';
        }
        // For objects with type property (enums)
        if (value.type) {
            const props = Object.keys(value).filter(k => k !== 'type').map(k => debug_repr(value[k])).join(', ');
            return value.type + (props ? '(' + props + ')' : '');
        }
        return JSON.stringify(value);
    }
    return String(value);
}

// Panic function for Rust panic! macro
function panic(message) {
    throw new Error('Rust panic: ' + (message || 'explicit panic'));
}

// Assert function for Rust assert! macro
function assert(condition, message) {
    if (!condition) {
        panic(message || 'assertion failed');
    }
}

"#;

    let separator = if pretty { "\n\n" } else { "\n" };
    format!("{}{}{}", helpers.trim(), separator, js_code)
}

fn run_javascript(js_code: &str, function_to_call: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new JavaScript context
    let mut context = Context::default();

    // Add console object with log, error, warn, etc.
    let console_code = r#"
const console = {
    log: function(...args) {
        print(args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : String(arg)).join(' '));
    },
    error: function(...args) {
        print('[ERROR] ' + args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : String(arg)).join(' '));
    },
    warn: function(...args) {
        print('[WARN] ' + args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : String(arg)).join(' '));
    },
    info: function(...args) {
        print('[INFO] ' + args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : String(arg)).join(' '));
    },
    debug: function(...args) {
        print('[DEBUG] ' + args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : String(arg)).join(' '));
    }
};
"#;

    // Initialize console object
    context.eval(Source::from_bytes(console_code))?;

    // Evaluate the JavaScript code
    match context.eval(Source::from_bytes(js_code)) {
        Ok(_) => {
            // If a specific function was requested, try to call it
            if let Some(func_name) = function_to_call {
                println!("Calling function: {}()", func_name);
                let call_code = format!("{}()", func_name);
                match context.eval(Source::from_bytes(&call_code)) {
                    Ok(result) => {
                        let result_str = result.display().to_string();
                        if result_str != "undefined" {
                            println!("Function result: {}", result_str);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error calling function '{}': {}", func_name, e);
                        return Err(format!("Function call failed: {}", e).into());
                    }
                }
            } else {
                // Try to call main() if it exists and no specific function was requested
                match context.eval(Source::from_bytes("typeof main !== 'undefined' ? main() : undefined")) {
                    Ok(result) => {
                        let result_str = result.display().to_string();
                        if result_str != "undefined" {
                            println!("main() result: {}", result_str);
                        }
                    }
                    Err(_) => {
                        // main() doesn't exist or failed, that's okay
                        println!("No main() function found or no specific function requested.");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("JavaScript execution error: {}", e);
            return Err(format!("JavaScript execution failed: {}", e).into());
        }
    }

    Ok(())
}

// Cargo.toml dependencies needed:
/*
[dependencies]
mojes-mojo = { path = "../mojes-mojo" }  # Adjust path as needed
clap = { version = "4.0", features = ["derive"] }
syn = { version = "2.0", features = ["full", "parsing", "printing"] }
boa_engine = "0.17"
swc_common = "0.33"
swc_ecma_ast = "0.112"
*/
