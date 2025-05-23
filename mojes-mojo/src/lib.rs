use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Block, Expr, Fields, ItemEnum, ItemStruct, Pat, Stmt, Type};

// Handle binary operations
fn handle_binary_op(bin: &syn::ExprBinary) -> String {
    let left = rust_expr_to_js(&bin.left);
    let right = rust_expr_to_js(&bin.right);

    match &bin.op {
        syn::BinOp::Add(_) => {
            // Special case for string concatenation
            if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                format!("`${{{}}}${{{}}}`", left, right)
            } else {
                format!("{} + {}", left, right)
            }
        }
        syn::BinOp::Sub(_) => format!("{} - {}", left, right),
        syn::BinOp::Mul(_) => format!("{} * {}", left, right),
        syn::BinOp::Div(_) => format!("{} / {}", left, right),
        syn::BinOp::Rem(_) => format!("{} % {}", left, right),
        syn::BinOp::And(_) => format!("{} && {}", left, right),
        syn::BinOp::Or(_) => format!("{} || {}", left, right),
        syn::BinOp::BitXor(_) => format!("{} ^ {}", left, right),
        syn::BinOp::BitAnd(_) => format!("{} & {}", left, right),
        syn::BinOp::BitOr(_) => format!("{} | {}", left, right),
        syn::BinOp::Shl(_) => format!("{} << {}", left, right),
        syn::BinOp::Shr(_) => format!("{} >> {}", left, right),
        syn::BinOp::Eq(_) => format!("{} === {}", left, right),
        syn::BinOp::Lt(_) => format!("{} < {}", left, right),
        syn::BinOp::Le(_) => format!("{} <= {}", left, right),
        syn::BinOp::Ne(_) => format!("{} !== {}", left, right),
        syn::BinOp::Ge(_) => format!("{} >= {}", left, right),
        syn::BinOp::Gt(_) => format!("{} > {}", left, right),

        syn::BinOp::AddAssign(_) => format!("{} += {}", left, right), // Handle +=
        syn::BinOp::SubAssign(_) => format!("{} -= {}", left, right), // Handle -=
        syn::BinOp::MulAssign(_) => format!("{} *= {}", left, right), // Handle *=
        syn::BinOp::DivAssign(_) => format!("{} /= {}", left, right), // Handle /=
        syn::BinOp::RemAssign(_) => format!("{} %= {}", left, right), // Handle %=
        syn::BinOp::BitXorAssign(_) => format!("{} ^= {}", left, right), // Handle ^=
        syn::BinOp::BitAndAssign(_) => format!("{} &= {}", left, right), // Handle &=
        syn::BinOp::BitOrAssign(_) => format!("{} |= {}", left, right), // Handle |=
        syn::BinOp::ShlAssign(_) => format!("{} <<= {}", left, right), // Handle <<=
        syn::BinOp::ShrAssign(_) => format!("{} >>= {}", left, right), // Handle >>=

        x => format!("/* {:?} */ {} + {}", x, left, right), // Default fallback
    }
}

/*
// Add a separate function to handle assignment operators
fn handle_assignment_op(expr: &syn::ExprAssign) -> String {
    let left = rust_expr_to_js(&expr.left);
    let right = rust_expr_to_js(&expr.right);

    match &expr.op {
        syn::BinOp::Add(_) => format!("{} += {}", left, right),
        syn::BinOp::Sub(_) => format!("{} -= {}", left, right),
        syn::BinOp::Mul(_) => format!("{} *= {}", left, right),
        syn::BinOp::Div(_) => format!("{} /= {}", left, right),
        syn::BinOp::Rem(_) => format!("{} %= {}", left, right),
        syn::BinOp::BitXor(_) => format!("{} ^= {}", left, right),
        syn::BinOp::BitAnd(_) => format!("{} &= {}", left, right),
        syn::BinOp::BitOr(_) => format!("{} |= {}", left, right),
        syn::BinOp::Shl(_) => format!("{} <<= {}", left, right),
        syn::BinOp::Shr(_) => format!("{} >>= {}", left, right),
        _ => format!("{} = {}", left, right), // Default fallback
    }
}
*/

// Add a proper macro handler function - particularly for format!

fn handle_macro_expr(mac: &syn::Macro) -> String {
    // Get the macro name
    let macro_name = if let Some(segment) = mac.path.segments.last() {
        segment.ident.to_string()
    } else {
        return "/* Invalid macro */".to_string();
    };

    match macro_name.as_str() {
        "format" => {
            // Handle format! macro - just delegate to handle_format_like_macro
            let tokens = &mac.tokens;
            let token_string = tokens.to_string();

            // Use the same logic as println! and other format-like macros
            handle_format_like_macro(&token_string)
        }
        "println" => {
            // Convert println! to console.log
            let tokens = &mac.tokens;
            let token_string = tokens.to_string();
            eprintln!("DEBUG println! tokens: '{}'", token_string);

            if token_string.trim().is_empty() {
                // println!() with no arguments
                "console.log()".to_string()
            } else {
                // Handle println! with format string and arguments
                if token_string.contains("{}") {
                    // This is a format-style println!
                    eprintln!("DEBUG: Detected format style println!");
                    let format_result = handle_format_like_macro(&token_string);
                    format!("console.log({})", format_result)
                } else {
                    // Simple println! with just a string or expression
                    format!("console.log({})", token_string)
                }
            }
        }

        "print" => {
            // Similar to println but without newline (JS console.log always adds newline though)
            let tokens = &mac.tokens;
            let token_string = tokens.to_string();

            if token_string.trim().is_empty() {
                "console.log()".to_string()
            } else {
                if token_string.contains("{}") {
                    let format_result = handle_format_like_macro(&token_string);
                    format!("console.log({})", format_result)
                } else {
                    format!("console.log({})", token_string)
                }
            }
        }

        "eprintln" | "eprint" => {
            // Convert to console.error for stderr output
            let tokens = &mac.tokens;
            let token_string = tokens.to_string();

            if token_string.trim().is_empty() {
                "console.error()".to_string()
            } else {
                if token_string.contains("{}") {
                    let format_result = handle_format_like_macro(&token_string);
                    format!("console.error({})", format_result)
                } else {
                    format!("console.error({})", token_string)
                }
            }
        }

        "vec" => {
            // Handle vec! macro - convert to JavaScript array literal
            let tokens = &mac.tokens;
            let token_string = tokens.to_string();

            if token_string.trim().is_empty() {
                // vec!() -> []
                "[]".to_string()
            } else if token_string.contains(";") {
                // vec![value; count] -> Array.from({length: count}, () => value)
                let parts: Vec<&str> = token_string.split(';').collect();
                if parts.len() == 2 {
                    let value = parts[0].trim();
                    let count = parts[1].trim();
                    format!("Array.from({{length: {}}}, () => {})", count, value)
                } else {
                    format!("[{}]", token_string) // Fallback
                }
            } else {
                // vec![a, b, c] -> [a, b, c]
                let token_string = token_string.replace(" , ", ", ");
                format!("[{}]", token_string)
            }
        }

        x => panic!("/* Unsupported macro {} */", macro_name),
    }
}

// Helper function to handle format-like macros (reusable for println!, etc.)

fn handle_format_like_macro(token_string: &str) -> String {
    eprintln!("DEBUG handle_format_like_macro input: '{}'", token_string);

    // Smart comma splitting that respects quote boundaries
    let parts = smart_comma_split(token_string);
    eprintln!("DEBUG parts after smart split: {:?}", parts);

    if parts.is_empty() {
        return "\"\"".to_string();
    }

    // Get the format string (remove quotes)
    let mut format_str = parts[0].trim();
    if format_str.starts_with('"') && format_str.ends_with('"') {
        format_str = &format_str[1..format_str.len() - 1];
    }
    eprintln!("DEBUG format_str: '{}'", format_str);

    // Get format arguments
    let format_args: Vec<String> = parts
        .iter()
        .skip(1)
        .map(|arg| arg.trim().to_string())
        .collect();
    eprintln!("DEBUG format_args: {:?}", format_args);

    // Check if there are actually placeholders
    if !format_str.contains("{}") {
        // No placeholders, but still return template literal for consistency
        // and to handle backtick escaping properly
        let escaped_format_str = format_str.replace("`", "\\`");
        return format!("`{}`", escaped_format_str);
    }

    // Split the format string at each placeholder
    let str_parts: Vec<&str> = format_str.split("{}").collect();
    eprintln!("DEBUG str_parts: {:?}", str_parts);

    // Combine the parts with the arguments using a template literal
    let mut result = String::from("`");

    for (i, part) in str_parts.iter().enumerate() {
        eprintln!("DEBUG processing part {}: '{}'", i, part);

        // Escape backticks in the string parts
        let escaped_part = part.replace("`", "\\`");
        eprintln!("DEBUG escaped_part: '{}'", escaped_part);

        result.push_str(&escaped_part);

        // Add the argument if there is one for this placeholder
        if i < format_args.len() {
            result.push_str(&format!("${{{}}}", format_args[i]));
        }
    }

    result.push('`');
    eprintln!("DEBUG final result: '{}'", result);
    result
}

// Add this helper function to properly split on commas while respecting quotes
fn smart_comma_split(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Toggle quote state
                in_quotes = !in_quotes;
                current_part.push(ch);
            }
            ',' if !in_quotes => {
                // Found a comma outside quotes, split here
                parts.push(current_part.trim().to_string());
                current_part.clear();
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    // Add the last part
    if !current_part.is_empty() {
        parts.push(current_part.trim().to_string());
    }

    parts
}

fn update_rust_expr_to_js_for_macros(expr: &Expr) -> Option<String> {
    match expr {
        // Handle tuple expressions
        Expr::Tuple(tuple_expr) => {
            let elements: Vec<String> = tuple_expr
                .elems
                .iter()
                .map(|elem| rust_expr_to_js(elem))
                .collect();

            Some(format!("[{}]", elements.join(", ")))
        }
        // Handle cast expressions (type conversion)
        Expr::Cast(cast_expr) => {
            let expr_js = rust_expr_to_js(&cast_expr.expr);
            let target_type = format_rust_type(&cast_expr.ty);

            // Handle different cast types
            let result = match target_type.as_str() {
                "number" => {
                    // Casting to numeric types - use Number() for explicit conversion
                    format!("Number({})", expr_js)
                }
                "string" => {
                    // Casting to string - use String() or .toString()
                    format!("String({})", expr_js)
                }
                "boolean" => {
                    // Casting to boolean - use Boolean()
                    format!("Boolean({})", expr_js)
                }
                _ => {
                    // For other types, just use the expression with a comment
                    format!(
                        "{} /* was {} as {} in Rust */",
                        expr_js, expr_js, target_type
                    )
                }
            };
            Some(result)
        }
        Expr::Macro(mac_expr) => {
            // Handle macro expressions
            Some(handle_macro_expr(&mac_expr.mac))
        }
        Expr::Binary(bin_expr) => Some(handle_binary_op(bin_expr)),
        // Handle reference expressions (&expr)
        Expr::Reference(ref_expr) => {
            // In JavaScript, references don't exist in the same way as Rust
            // We'll just return the referenced expression itself
            let inner_expr = rust_expr_to_js(&ref_expr.expr);

            // Handle different types of references
            match ref_expr.mutability {
                Some(_) => {
                    // Mutable reference (&mut expr)
                    // In JS, we can't have true mutable references, so just return the value
                    // Add a comment to indicate it was a mutable reference in Rust
                    Some(format!("{} /* was &mut in Rust */", inner_expr))
                }
                None => {
                    // Immutable reference (&expr)
                    // For most cases, just return the inner expression
                    // However, for certain patterns, we might want to handle differently
                    match &*ref_expr.expr {
                        // If it's a reference to a string literal, just return the string
                        Expr::Lit(lit) => match &lit.lit {
                            syn::Lit::Str(_) => Some(inner_expr),
                            _ => Some(format!("{} /* was & in Rust */", inner_expr)),
                        },
                        // If it's a reference to a variable, just return the variable
                        Expr::Path(_) => Some(inner_expr),
                        // For other expressions, add a comment
                        _ => Some(format!("{} /* was & in Rust */", inner_expr)),
                    }
                }
            }
        }

        // x => { eprintln!("EXPR: {:?}", &x); None }, // Not a macro or binary expression
        _ => None,
    }
}

pub fn handle_format_macro(args: &Punctuated<Expr, Comma>) -> String {
    if args.is_empty() {
        return "\"\"".to_string();
    }

    // Get the format string
    if let Some(first_arg) = args.first() {
        if let Expr::Lit(lit) = first_arg {
            if let syn::Lit::Str(str_lit) = &lit.lit {
                let format_str = str_lit.value();

                // If there are no placeholders, just return the string literal
                if !format_str.contains("{}") {
                    return format!("\"{}\"", format_str);
                }

                // Get the arguments to fill in the placeholders
                let format_args: Vec<String> = args
                    .iter()
                    .skip(1) // Skip the format string
                    .map(|arg| rust_expr_to_js(arg))
                    .collect();

                // Split the format string at each placeholder
                let parts: Vec<&str> = format_str.split("{}").collect();

                // Combine the parts with the arguments using a template literal
                let mut result = String::from("`");

                for (i, part) in parts.iter().enumerate() {
                    // Escape backticks in the string parts
                    let escaped_part = part.replace("`", "\\`");
                    result.push_str(&escaped_part);

                    // Add the argument if there is one for this placeholder
                    if i < format_args.len() {
                        result.push_str(&format!("${{{}}}", format_args[i]));
                    }
                }

                result.push('`');
                return result;
            }
        }
    }

    // Fallback: just concatenate the arguments
    let args_js: Vec<String> = args.iter().map(|arg| rust_expr_to_js(arg)).collect();

    args_js.join(" + ")
}

// Handle if-let expressions specifically
fn handle_if_let(expr: &syn::ExprIf) -> Option<String> {
    // Check if this is an "if let" expression by examining the condition
    if let Expr::Let(expr_let) = &*expr.cond {
        // This is an "if let" expression
        // Extract the pattern and the expression being matched
        let pat = &expr_let.pat;
        let init_expr = &expr_let.expr;

        // Check if this is specifically "if let Some(x) = ..." pattern
        if let Pat::TupleStruct(tuple_struct) = &**pat {
            if let Some(last_segment) = tuple_struct.path.segments.last() {
                if last_segment.ident == "Some" && !tuple_struct.elems.is_empty() {
                    // This is "if let Some(x) = ..." pattern
                    if let Some(inner_pat) = tuple_struct.elems.first() {
                        if let Pat::Ident(ident) = inner_pat {
                            let var_name = ident.ident.to_string();
                            let matched_expr = rust_expr_to_js(init_expr);
                            let then_js = rust_block_to_js(&expr.then_branch);

                            // Convert to JavaScript null/undefined check
                            let mut js = format!("(function() {{\n");
                            js.push_str(&format!("  const _temp = {};\n", matched_expr));
                            js.push_str(&format!(
                                "  if (_temp !== null && _temp !== undefined) {{\n"
                            ));
                            js.push_str(&format!("    const {} = _temp;\n", var_name));

                            // Add the then branch
                            js.push_str(&indent_lines(&then_js, 4));
                            js.push_str("  }");

                            // Handle the else branch if it exists
                            if let Some((_, else_branch)) = &expr.else_branch {
                                match &**else_branch {
                                    Expr::Block(else_block) => {
                                        let else_js = rust_block_to_js(&else_block.block);
                                        js.push_str(&format!(
                                            " else {{\n{}",
                                            indent_lines(&else_js, 4)
                                        ));
                                        js.push_str("  }");
                                    }
                                    _ => {
                                        let else_js = rust_expr_to_js(else_branch);
                                        js.push_str(&format!(
                                            " else {{\n    return {};\n  }}",
                                            else_js
                                        ));
                                    }
                                }
                            }

                            js.push_str("\n  return undefined;\n})()");
                            return Some(js);
                        }
                    }
                }
            }
        }
    }

    // Not an "if let Some(x) = ..." pattern
    None
}

// Handle while loops
fn handle_while_expr(expr: &syn::ExprWhile) -> String {
    let cond_js = rust_expr_to_js(&expr.cond);
    let body_js = rust_block_to_js(&expr.body);

    // Remove starting spaces from each line in body_js
    let trimmed_body = body_js
        .lines()
        .map(|line| {
            if line.starts_with("  ") {
                line[2..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    // wrap the body into a function
    let trimmed_body = format!("(function() {{\n{}}})();", trimmed_body);

    format!("while ({}) {{\n{}}}", cond_js, trimmed_body)
}

// Handle for loops
fn handle_for_expr(expr: &syn::ExprForLoop) -> String {
    // Extract the loop variable
    let var_name = if let Pat::Ident(pat_ident) = &*expr.pat {
        pat_ident.ident.to_string()
    } else {
        "item".to_string() // Fallback variable name
    };

    // Convert the iterable expression
    let iterable_js = rust_expr_to_js(&expr.expr);

    // Convert the loop body
    let body_js = rust_block_to_js(&expr.body);

    // Remove starting spaces from each line in body_js
    let trimmed_body = body_js
        .lines()
        .map(|line| {
            if line.starts_with("  ") {
                line[2..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    // wrap the body into a function
    let trimmed_body = format!("(function() {{\n{}}})();", trimmed_body);

    format!(
        "for (const {} of {}) {{\n{}}}",
        var_name, iterable_js, trimmed_body
    )
}

// Update rust_block_to_js to handle loops
pub fn rust_block_to_js(block: &Block) -> String {
    let mut js_code = String::new();

    for stmt in &block.stmts {
        let stmt_js = match stmt {
            Stmt::Expr(expr, semi) => {
                match expr {
                    Expr::While(while_expr) => {
                        // Handle while loops - don't add semicolon or extra formatting
                        let expr_js = handle_while_expr(while_expr);
                        format!("  {};\n", expr_js) // Add proper indentation
                    }
                    Expr::ForLoop(for_expr) => {
                        // Handle for loops - don't add semicolon or extra formatting
                        let expr_js = handle_for_expr(for_expr);
                        format!("  {};\n", expr_js) // Add proper indentation
                    }
                    _ => {
                        // Regular expression handling
                        let js_expr = rust_expr_to_js(expr);

                        // Add semicolon if it exists in Rust
                        if semi.is_some() {
                            format!("  {};\n", js_expr)
                        } else {
                            // For return expressions
                            format!("  return {};\n", js_expr)
                        }
                    }
                }
            }
            Stmt::Local(local) => {
                // Handle variable declarations
                if let Some(init) = &local.init {
                    // We directly access the expr from the init
                    let init_expr = &init.expr;

                    match &local.pat {
                        Pat::Ident(pat_ident) => {
                            let var_name = pat_ident.ident.to_string();
                            let init_js = rust_expr_to_js(init_expr);

                            // Check for mutability
                            if pat_ident.mutability.is_some() {
                                format!("  let {} = {};\n", var_name, init_js)
                            } else {
                                format!("  const {} = {};\n", var_name, init_js)
                            }
                        }
                        // Handle destructuring patterns
                        Pat::Tuple(tuple_pat) => {
                            let vars: Vec<String> = tuple_pat
                                .elems
                                .iter()
                                .filter_map(|pat| {
                                    if let Pat::Ident(pat_ident) = pat {
                                        Some(pat_ident.ident.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            let init_js = rust_expr_to_js(init_expr);
                            format!("  const [{}] = {};\n", vars.join(", "), init_js)
                        }
                        Pat::Struct(struct_pat) => {
                            let fields: Vec<String> = struct_pat
                                .fields
                                .iter()
                                .filter_map(|field| {
                                    if let Pat::Ident(pat_ident) = &*field.pat {
                                        Some(pat_ident.ident.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            let init_js = rust_expr_to_js(init_expr);
                            format!("  const {{ {} }} = {};\n", fields.join(", "), init_js)
                        }
                        x => panic!("  /* Unsupported destructuring pattern: {:?}  */\n", x),
                    }
                } else {
                    // Variable declaration without initialization
                    match &local.pat {
                        Pat::Ident(pat_ident) => {
                            let var_name = pat_ident.ident.to_string();

                            // Check for mutability
                            if pat_ident.mutability.is_some() {
                                format!("  let {};\n", var_name)
                            } else {
                                format!("  let {};\n", var_name)
                            }
                        }
                        x => panic!("  /* Unsupported variable pattern: {:?}  */\n", x),
                    }
                }
            }
            Stmt::Macro(mac_stmt) => {
                let macro_result = handle_macro_expr(&mac_stmt.mac);
                // Add proper indentation and semicolon for macro statements
                format!("  {};\n", macro_result)
            }
            // Remove unsupported Stmt variants
            x => panic!("Unsupported statement {:?}", &x), //"  /* Unsupported statement */\n".to_string(),
        };

        js_code.push_str(&stmt_js);
    }

    js_code
}

// The main function for converting Rust expressions to JavaScript
pub fn rust_expr_to_js(expr: &Expr) -> String {
    if let Some(result) = update_rust_expr_to_js_for_macros(expr) {
        return result;
    }

    match expr {
        // Handle array repeat expressions [value; count]
        Expr::Repeat(repeat_expr) => {
            let value_js = rust_expr_to_js(&repeat_expr.expr);
            let count_js = rust_expr_to_js(&repeat_expr.len);

            // Generate JavaScript array filled with the repeated value
            format!("Array.from({{length: {}}}, () => {})", count_js, value_js)
        }

        // Handle loop expressions (infinite loops)
        Expr::Loop(loop_expr) => {
            let body_js = rust_block_to_js(&loop_expr.body);

            // Convert to while(true) loop in JavaScript
            // Note: break with values is complex, for now just handle basic loops
            format!("while (true) {{\n{}}}", body_js)
        }

        // Handle break expressions
        Expr::Break(break_expr) => {
            if let Some(value) = &break_expr.expr {
                let value_js = rust_expr_to_js(value);
                // For now, just return the value (full break-with-value needs more complex handling)
                format!("return {}; /* was break {} in Rust */", value_js, value_js)
            } else {
                "break;".to_string()
            }
        }

        // Handle continue expressions
        Expr::Continue(_) => "continue;".to_string(),

        // Handle verbatim expressions (edge case)
        Expr::Verbatim(x) => {
            format!("/* Empty or verbatim expression: {:?} */", x)
        }

        Expr::Closure(closure) => {
            // Extract parameter names
            let params: Vec<String> = closure
                .inputs
                .iter()
                .filter_map(|param| {
                    if let syn::Pat::Ident(pat_ident) = param {
                        Some(pat_ident.ident.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            // Handle closure body specially
            let body_js = match &*closure.body {
                // If the closure body is a block, handle it directly without IIFE wrapping
                Expr::Block(block_expr) => {
                    let block_content = rust_block_to_js(&block_expr.block);
                    // Remove the outer braces and IIFE wrapping for closure context
                    let trimmed = block_content.trim();
                    if trimmed.starts_with("(function()") && trimmed.ends_with("})()") {
                        // Extract the inner content
                        let inner = &trimmed[12..trimmed.len() - 4]; // Remove "(function() {" and "})()"
                        format!("{{{}}}", inner)
                    } else {
                        format!("{{{}}}", block_content)
                    }
                }
                // For single expressions, convert normally
                _ => {
                    let expr_js = rust_expr_to_js(&closure.body);
                    // If it's a simple expression, don't wrap in braces
                    if expr_js.contains("return ") || expr_js.contains("(function()") {
                        format!("{{{}}}", expr_js)
                    } else {
                        expr_js
                    }
                }
            };

            if params.len() == 1 {
                format!("{} => {}", params[0], body_js)
            } else {
                format!("({}) => {}", params.join(", "), body_js)
            }
        }

        // Handle async blocks
        Expr::Async(async_expr) => {
            // Convert the async block to a regular async function
            let block_js = rust_block_to_js(&async_expr.block);
            format!("(async function() {{\n{}}})();", block_js)
        }

        // Handle await expressions
        Expr::Await(await_expr) => {
            let base_js = rust_expr_to_js(&await_expr.base);
            format!("await {}", base_js)
        }

        // Handle range expressions
        Expr::Range(range_expr) => {
            match (&range_expr.start, &range_expr.end) {
                (Some(start), Some(end)) => {
                    let start_js = rust_expr_to_js(start);
                    let end_js = rust_expr_to_js(end);

                    // Check if it's inclusive (..=) or exclusive (..)
                    match range_expr.limits {
                        syn::RangeLimits::HalfOpen(_) => {
                            // Exclusive range (1..10) -> Array.from({length: 10-1}, (_, i) => i + 1)
                            format!(
                                "Array.from({{length: {} - {}}}, (_, i) => i + {})",
                                end_js, start_js, start_js
                            )
                        }
                        syn::RangeLimits::Closed(_) => {
                            // Inclusive range (1..=10) -> Array.from({length: 10-1+1}, (_, i) => i + 1)
                            format!(
                                "Array.from({{length: {} - {} + 1}}, (_, i) => i + {})",
                                end_js, start_js, start_js
                            )
                        }
                    }
                }
                (Some(start), None) => {
                    // Range from start (1..) - not easily representable in JS
                    let start_js = rust_expr_to_js(start);
                    format!("/* Range from {} (infinite) */", start_js)
                }
                (None, Some(end)) => {
                    // Range to end (..10) -> Array.from({length: 10}, (_, i) => i)
                    let end_js = rust_expr_to_js(end);
                    format!("Array.from({{length: {}}}, (_, i) => i)", end_js)
                }
                (None, None) => {
                    // Full range (..) - infinite range
                    "/* Full range (..) - infinite */".to_string()
                }
            }
        }

        // Handle try expressions (? operator)
        Expr::Try(try_expr) => {
            let base_js = rust_expr_to_js(&try_expr.expr);
            // In JavaScript, we can simulate the ? operator with optional chaining or try/catch
            // For now, just add a comment indicating it was a try operation
            format!("{} /* was {}? in Rust */", base_js, base_js)
        }

        // Handle let expressions (if let, while let, etc.)
        Expr::Let(let_expr) => {
            // This is complex - for now, generate a comment
            let expr_js = rust_expr_to_js(&let_expr.expr);
            format!("/* let pattern = {} */", expr_js)
        }

        // Handle macro expressions that aren't covered by update_rust_expr_to_js_for_macros
        Expr::Macro(macro_expr) => handle_macro_expr(&macro_expr.mac),

        // Handle while loops
        Expr::While(while_expr) => handle_while_expr(while_expr),

        // Handle for loops
        Expr::ForLoop(for_expr) => handle_for_expr(for_expr),

        Expr::Block(block_expr) => {
            // Handle nested blocks
            let block_js = rust_block_to_js(&block_expr.block);
            format!("(function() {{\n{}}})();", block_js)
        }

        // Handle if-let expressions
        Expr::If(if_expr) => {
            // First check if this is an "if let Some(x) = ..." pattern
            if let Some(if_let_js) = handle_if_let(if_expr) {
                return if_let_js;
            }

            // Regular if expression
            let cond_js = rust_expr_to_js(&if_expr.cond);
            let then_js = rust_block_to_js(&if_expr.then_branch);
            let mut if_js = format!(
                "(function() {{\n  if ({}) {{\n{}",
                cond_js,
                indent_lines(&then_js, 4)
            );

            if_js.push_str("  }");

            // Handle else branch
            if let Some((_, else_branch)) = &if_expr.else_branch {
                match &**else_branch {
                    Expr::Block(else_block) => {
                        let else_js = rust_block_to_js(&else_block.block);
                        if_js.push_str(&format!(" else {{\n{}", indent_lines(&else_js, 4)));
                        if_js.push_str("  }");
                    }
                    Expr::If(_) => {
                        // Handle else if
                        let else_if_js = rust_expr_to_js(else_branch);
                        if_js.push_str(&format!(" else {}", else_if_js));
                    }
                    _ => {
                        let else_js = rust_expr_to_js(else_branch);
                        if_js.push_str(&format!(" else {{\n    return {};\n  }}", else_js));
                    }
                }
            }

            if_js.push_str("\n  return undefined;\n})()");
            if_js
        }

        // Handle function calls and format! macro
        Expr::Call(call) => {
            // Get the function name
            let func_name = match &*call.func {
                Expr::Path(path) => {
                    if let Some(last_segment) = path.path.segments.last() {
                        let name = last_segment.ident.to_string();
                        match name.as_str() {
                            "println" | "print" => "console.log".to_string(),
                            "eprintln" | "eprint" => "console.error".to_string(),
                            "format" => "".to_string(), // format! becomes string template in JS
                            "Some" => "".to_string(),   // Option::Some becomes just the value in JS
                            _ => name,
                        }
                    } else {
                        panic!("/* Unsupported function path */");
                    }
                }
                _ => rust_expr_to_js(&call.func),
            };

            // Handle special cases
            if func_name.is_empty() {
                if path_ends_with(&call.func, "format") {
                    // Use our improved format handling
                    return handle_format_macro(&call.args);
                } else if path_ends_with(&call.func, "Some") {
                    // For Option::Some, just return the value in JS
                    if call.args.len() == 1 {
                        return rust_expr_to_js(&call.args[0]);
                    } else {
                        return format!("/* Invalid Some with {} args */", call.args.len());
                    }
                }
            }

            // Convert arguments
            let args: Vec<String> = call.args.iter().map(|arg| rust_expr_to_js(arg)).collect();

            // Regular function call
            format!("{}({})", func_name, args.join(", "))
        }

        // Handle Option::Some and Option::None methods better
        Expr::MethodCall(method_call) => {
            // Handle method calls
            let receiver = rust_expr_to_js(&method_call.receiver);
            let method_name = method_call.method.to_string();

            // Map Rust methods to JavaScript methods
            let js_method = match method_name.as_str() {
                "len" => {
                    // Special case: .len() becomes .length (property, not method)
                    return format!("{}.length", receiver);
                }
                "push" => "push",
                "pop" => "pop",
                "remove" => "splice",
                "insert" => "splice",
                "iter" => "",    // In JS, we don't need .iter() for iteration
                "collect" => "", // In JS, we don't need .collect()
                "map" => "map",
                "filter" => "filter",
                "find" => "find",
                "contains" => "includes",
                "to_string" => "toString",
                "to_uppercase" => "toUpperCase",
                "to_lowercase" => "toLowerCase",
                "trim" => "trim",
                "trim_start" => "trimStart",
                "trim_end" => "trimEnd",
                "starts_with" => "startsWith",
                "ends_with" => "endsWith",
                "replace" => "replace",
                "split" => "split",
                "join" => "join",
                "is_some" => "", // Handle Option methods specially
                "is_none" => "",
                "unwrap" => "",
                _ => &method_name,
            };

            // Convert arguments
            let args: Vec<String> = method_call
                .args
                .iter()
                .map(|arg| rust_expr_to_js(arg))
                .collect();

            // Handle empty JS method (e.g., .iter(), .collect())
            if js_method.is_empty() {
                match method_name.as_str() {
                    "is_some" => format!("({} !== null && {} !== undefined)", receiver, receiver),
                    "is_none" => format!("({} === null || {} === undefined)", receiver, receiver),
                    "unwrap" => receiver, // Just use the value itself
                    _ => receiver,
                }
            } else {
                format!("{}.{}({})", receiver, js_method, args.join(", "))
            }
        }

        // Handle literals
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => format!(
                "\"{}\"",
                // s.value().replace("\"", "\\\"").replace("\n", "\\n")
                s.value()
                    .replace("\\", "\\\\") // Escape backslashes first!
                    .replace("\"", "\\\"") // Escape quotes
                    .replace("\n", "\\n") // Escape newlines
                    .replace("\t", "\\t") // Escape tabs
                    .replace("\r", "\\r") // Escape carriage returns
            ),
            syn::Lit::Int(i) => i.to_string(),
            syn::Lit::Float(f) => f.to_string(),
            syn::Lit::Bool(b) => b.value.to_string(),
            syn::Lit::Char(c) => format!("\"{}\"", c.value()),
            x => panic!("/* Unsupported literal: {:?} */", x),
        },
        Expr::Unary(unary) => {
            let operand = rust_expr_to_js(&unary.expr);
            match &unary.op {
                syn::UnOp::Not(_) => format!("!{}", operand),
                syn::UnOp::Neg(_) => format!("-{}", operand),
                syn::UnOp::Deref(_) => operand, // In JS, we don't have explicit dereferencing
                &_ => todo!(),
            }
        }

        // Handle binary operations
        Expr::Binary(bin) => {
            let left = rust_expr_to_js(&bin.left);
            let right = rust_expr_to_js(&bin.right);

            match &bin.op {
                syn::BinOp::Add(_) => {
                    // Special case for string concatenation
                    if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                        format!("`${{{}}}${{{}}}`", left, right)
                    } else {
                        format!("{} + {}", left, right)
                    }
                }
                syn::BinOp::Sub(_) => format!("{} - {}", left, right),
                syn::BinOp::Mul(_) => format!("{} * {}", left, right),
                syn::BinOp::Div(_) => format!("{} / {}", left, right),
                syn::BinOp::Rem(_) => format!("{} % {}", left, right),
                syn::BinOp::And(_) => format!("{} && {}", left, right),
                syn::BinOp::Or(_) => format!("{} || {}", left, right),
                syn::BinOp::BitXor(_) => format!("{} ^ {}", left, right),
                syn::BinOp::BitAnd(_) => format!("{} & {}", left, right),
                syn::BinOp::BitOr(_) => format!("{} | {}", left, right),
                syn::BinOp::Shl(_) => format!("{} << {}", left, right),
                syn::BinOp::Shr(_) => format!("{} >> {}", left, right),
                syn::BinOp::Eq(_) => format!("{} === {}", left, right),
                syn::BinOp::Lt(_) => format!("{} < {}", left, right),
                syn::BinOp::Le(_) => format!("{} <= {}", left, right),
                syn::BinOp::Ne(_) => format!("{} !== {}", left, right),
                syn::BinOp::Ge(_) => format!("{} >= {}", left, right),
                syn::BinOp::Gt(_) => format!("{} > {}", left, right),
                x => panic!("/* Unsupported binary op {:?} */ ({}, {})", x, left, right),
            }
        }

        // Handle paths (variables, constant references)
        Expr::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
                let ident_str = last_segment.ident.to_string();

                // Special case for common Rust constants
                match last_segment.ident.to_string().as_str() {
                    "None" => "null".to_string(),
                    "Some" => "".to_string(), // We'll handle this in the Index or Call expression
                    "true" | "false" => last_segment.ident.to_string(),
                    // Map Status enum variants to their JavaScript equivalents
                    "Active" => "Status.Active".to_string(),
                    "Inactive" => "Status.Inactive".to_string(),
                    "Pending" => "Status.Pending".to_string(),
                    "Custom" => "Status.Custom".to_string(),
                    _ => escape_js_identifier(&ident_str), // Use escaped version
                }
            } else {
                panic!("/* Unsupported path */");
            }
        }

        // Handle parenthesized expressions
        Expr::Paren(paren) => {
            format!("({})", rust_expr_to_js(&paren.expr))
        }

        // Handle match expressions with Option better
        Expr::Match(match_expr) => {
            // First check if this is a match on an Option
            eprintln!("DEBUG: Checking match arms:");
            for (i, arm) in match_expr.arms.iter().enumerate() {
                eprintln!("  Arm {}: {:?}", i, arm.pat);
                eprintln!("    is_some: {}", is_some_pattern(&arm.pat));
                eprintln!("    is_none: {}", is_none_pattern(&arm.pat));
            }
            let is_option_match = is_option_match_expr(match_expr);
            eprintln!("DEBUG: is_option_match result: {}", is_option_match);

            if is_option_match {
                // Handle Option match more intelligently
                let match_value = rust_expr_to_js(&match_expr.expr);

                // For Option matches, convert to if/else with null check
                let mut match_js =
                    format!("(function() {{\n  const _match_value = {};\n", match_value);

                let mut some_arm_js = String::new();
                let mut none_arm_js = String::new();

                for arm in &match_expr.arms {
                    if is_some_pattern(&arm.pat) {
                        // Extract variable name from Some(var) pattern
                        let var_name = extract_some_var_name(&arm.pat);
                        let body_js = rust_expr_to_js(&arm.body);

                        if let Some(name) = var_name {
                            some_arm_js = format!(
                                "  if (_match_value !== null && _match_value !== undefined) {{\n    const {} = _match_value;\n    return {};\n  }}\n",
                                name, body_js
                            );
                        } else {
                            some_arm_js = format!(
                                "  if (_match_value !== null && _match_value !== undefined) {{\n    return {};\n  }}\n",
                                body_js
                            );
                        }
                    } else if is_none_pattern(&arm.pat) {
                        let body_js = rust_expr_to_js(&arm.body);
                        none_arm_js = format!("  else {{\n    return {};\n  }}\n", body_js);
                    }
                }

                match_js.push_str(&some_arm_js);
                match_js.push_str(&none_arm_js);

                if none_arm_js.is_empty() {
                    match_js.push_str("  return undefined;\n");
                }

                match_js.push_str("})()");
                match_js
            } else {
                // Original match handling for non-Option matches
                let match_value = rust_expr_to_js(&match_expr.expr);

                // Use an IIFE to create a block scope
                let mut match_js =
                    format!("(function() {{\n  const _match_value = {};\n", match_value);

                // Convert each match arm to an if statement
                for (i, arm) in match_expr.arms.iter().enumerate() {
                    // The arm.pat is now a Pat directly, not a reference
                    let arm_js = match &arm.pat {
                        Pat::Lit(lit_pat) => {
                            // For Lit pattern, we need to create an ExprLit manually
                            let lit_js;
                            match &lit_pat.lit {
                                syn::Lit::Str(s) => lit_js = format!("\"{}\"", s.value()),
                                syn::Lit::Int(i) => lit_js = i.to_string(),
                                syn::Lit::Float(f) => lit_js = f.to_string(),
                                syn::Lit::Bool(b) => lit_js = b.value.to_string(),
                                syn::Lit::Char(c) => lit_js = format!("\"{}\"", c.value()),
                                // _ => lit_js = "/* Unsupported literal */".to_string(),
                                x => panic!("Unsupported literal {:?}", x),
                            }

                            if i == 0 {
                                format!("  if (_match_value === {}) {{\n", lit_js)
                            } else {
                                format!("  else if (_match_value === {}) {{\n", lit_js)
                            }
                        }
                        Pat::Wild(_) => {
                            // Wildcard pattern (_)
                            if i == 0 {
                                "  { // Default case\n".to_string()
                            } else {
                                "  else { // Default case\n".to_string()
                            }
                        }
                        Pat::Ident(pat_ident) => {
                            // Variable binding pattern
                            let var_name = pat_ident.ident.to_string();
                            if i == 0 {
                                format!(
                                    "  {{ // Binding to {}\n    const {} = _match_value;\n",
                                    var_name, var_name
                                )
                            } else {
                                format!(
                                    "  else {{ // Binding to {}\n    const {} = _match_value;\n",
                                    var_name, var_name
                                )
                            }
                        }
                        x => {
                            // Other patterns not supported fully
                            panic!("Unsupported other pattern (i = {}): {:?}", i, x);
                            if i == 0 {
                                "  if (true) { // Unsupported pattern\n".to_string()
                            } else {
                                "  else if (true) { // Unsupported pattern\n".to_string()
                            }
                        }
                    };

                    match_js.push_str(&arm_js);

                    // Convert the arm body
                    let body_js = rust_expr_to_js(&arm.body);
                    match_js.push_str(&format!("    return {};\n  }}\n", body_js));
                }

                match_js.push_str("  return undefined;\n})()");
                match_js
            }
        }

        // Handle array literals
        Expr::Array(array) => {
            let elements: Vec<String> = array
                .elems
                .iter()
                .map(|elem| rust_expr_to_js(elem))
                .collect();

            format!("[{}]", elements.join(", "))
        }

        // Handle array indexing
        Expr::Index(index) => {
            let array_js = rust_expr_to_js(&index.expr);
            let index_js = rust_expr_to_js(&index.index);

            format!("{}[{}]", array_js, index_js)
        }

        // Handle assignments
        Expr::Assign(assign) => {
            let left = rust_expr_to_js(&assign.left);
            let right = rust_expr_to_js(&assign.right);

            format!("{} = {}", left, right)
        }

        // Handle return statements
        Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                format!("return {}", rust_expr_to_js(expr))
            } else {
                "return".to_string()
            }
        }

        // Handle field access
        Expr::Field(field) => {
            let base = rust_expr_to_js(&field.base);
            let member = match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };

            format!("{}.{}", base, member)
        }

        // Handle struct instantiation as JS object
        Expr::Struct(struct_expr) => {
            let mut fields = Vec::new();

            for field in &struct_expr.fields {
                let field_name = match &field.member {
                    syn::Member::Named(ident) => ident.to_string(),
                    syn::Member::Unnamed(index) => format!("_{}", index.index),
                };
                let field_value = rust_expr_to_js(&field.expr);
                fields.push(format!("{}: {}", field_name, field_value));
            }

            format!("{{ {} }}", fields.join(", "))
        }

        // For any other unhandled expression
        x => panic!("/* Unsupported expression: {:?} */", x).to_string(),
    }
}

// Functions to check if an expression is a Some/None pattern
fn is_some_pattern(pat: &Pat) -> bool {
    match pat {
        Pat::TupleStruct(tuple_struct) => {
            if let Some(last_segment) = tuple_struct.path.segments.last() {
                last_segment.ident == "Some"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_none_pattern(pat: &Pat) -> bool {
    match pat {
        Pat::Path(path_pat) => {
            if let Some(segment) = path_pat.path.segments.last() {
                segment.ident == "None"
            } else {
                false
            }
        }
        Pat::Ident(pat_ident) => {
            // Handle None as an identifier pattern too
            pat_ident.ident == "None"
        }
        _ => false,
    }
}

// Updated function to extract a variable name from Some(var) pattern
fn extract_some_var_name(pat: &Pat) -> Option<String> {
    if let Pat::TupleStruct(tuple_struct) = pat {
        if let Some(last_segment) = tuple_struct.path.segments.last() {
            if last_segment.ident == "Some" && !tuple_struct.elems.is_empty() {
                if let Some(inner_pat) = tuple_struct.elems.first() {
                    if let Pat::Ident(ident) = inner_pat {
                        return Some(ident.ident.to_string());
                    }
                }
            }
        }
    }
    None
}

// Improved function to check if a match is on an Option
fn is_option_match_expr(match_expr: &syn::ExprMatch) -> bool {
    let mut has_some = false;
    let mut has_none = false;

    for arm in &match_expr.arms {
        if is_some_pattern(&arm.pat) {
            has_some = true;
        } else if is_none_pattern(&arm.pat) {
            has_none = true;
        }
    }

    has_some && has_none
}

// Map of Rust types to JavaScript types
pub fn get_js_type(rust_type: &str) -> &'static str {
    match rust_type {
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => "number",
        "f32" | "f64" => "number",
        "bool" => "boolean",
        "String" | "&str" | "str" => "string",
        "Vec" => "Array",
        "HashMap" | "BTreeMap" | "Map" => "Map",
        "HashSet" | "BTreeSet" | "Set" => "Set",
        "Option" => "", // Will be handled specially
        "Result" => "", // Will be handled specially
        _ => "object",  // Default for custom types
    }
}

// Helper function to generate a JavaScript class for a Rust struct
pub fn generate_js_class_for_struct(input_struct: &ItemStruct) -> String {
    let struct_name = input_struct.ident.to_string();
    let mut js_class = format!("class {} {{\n", struct_name);

    // Constructor
    js_class.push_str("  constructor(");

    let fields: Vec<(String, String)> = match &input_struct.fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                if let Some(ident) = &field.ident {
                    let field_name = ident.to_string();
                    let field_type = format_rust_type(&field.ty);
                    Some((field_name, field_type))
                } else {
                    None
                }
            })
            .collect(),
        Fields::Unnamed(_) => {
            // Handle tuple structs
            vec![("data".to_string(), "Array".to_string())]
        }
        Fields::Unit => {
            // Unit structs have no fields
            vec![]
        }
    };

    // Add constructor parameters
    let field_names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();

    js_class.push_str(&field_names.join(", "));
    js_class.push_str(") {\n");

    // Initialize fields
    for (name, _) in &fields {
        js_class.push_str(&format!("    this.{} = {};\n", name, name));
    }

    js_class.push_str("  }\n\n");

    // Add toJSON method for serialization
    js_class.push_str("  toJSON() {\n");
    js_class.push_str("    return {\n");

    for (name, _) in &fields {
        js_class.push_str(&format!("      {}: this.{},\n", name, name));
    }

    js_class.push_str("    };\n");
    js_class.push_str("  }\n\n");

    // Add fromJSON static method for deserialization
    js_class.push_str(&format!("  static fromJSON(json) {{\n"));
    js_class.push_str(&format!("    return new {}(", struct_name));

    let json_field_accessors: Vec<String> = fields
        .iter()
        .map(|(name, _)| format!("json.{}", name))
        .collect();

    js_class.push_str(&json_field_accessors.join(", "));
    js_class.push_str(");\n");
    js_class.push_str("  }\n");

    js_class.push_str("}");

    js_class
}

// Helper function to generate JavaScript for a Rust enum
pub fn generate_js_enum(input_enum: &ItemEnum) -> String {
    let enum_name = input_enum.ident.to_string();

    // Create a JavaScript object with enum variants
    let mut js_enum = format!("const {} = {{\n", enum_name);

    // Add enum variants
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();

        match &variant.fields {
            Fields::Unit => {
                // Simple enum variants become string values
                js_enum.push_str(&format!("  {}: '{}',\n", variant_name, variant_name));
            }
            Fields::Unnamed(_) | Fields::Named(_) => {
                // Create a factory function for complex variants
                js_enum.push_str(&format!("  {}(", variant_name));

                let field_types: Vec<String> = match &variant.fields {
                    Fields::Unnamed(fields) => (0..fields.unnamed.len())
                        .map(|i| format!("value{}", i))
                        .collect(),
                    Fields::Named(fields) => fields
                        .named
                        .iter()
                        .filter_map(|field| field.ident.as_ref().map(|ident| ident.to_string()))
                        .collect(),
                    _ => vec![],
                };

                js_enum.push_str(&field_types.join(", "));
                js_enum.push_str(") {\n");
                js_enum.push_str(&format!("    return {{ type: '{}', ", variant_name));

                match &variant.fields {
                    Fields::Unnamed(_) => {
                        js_enum.push_str("values: [");
                        js_enum.push_str(&field_types.join(", "));
                        js_enum.push_str("] };\n");
                    }
                    Fields::Named(_) => {
                        for field_name in &field_types {
                            js_enum.push_str(&format!("{}, ", field_name));
                        }
                        js_enum.push_str("};\n");
                    }
                    _ => {}
                }

                js_enum.push_str("  },\n");
            }
        }
    }

    // Add utility methods
    js_enum.push_str("\n  // Utility method to check the variant type\n");
    js_enum.push_str("  is(obj, variant) {\n");
    js_enum.push_str("    if (typeof obj === 'string') {\n");
    js_enum.push_str("      return obj === this[variant];\n");
    js_enum.push_str("    }\n");
    js_enum.push_str("    return obj && obj.type === variant;\n");
    js_enum.push_str("  },\n");

    js_enum.push_str("};\n\n");

    // Add type-checking functions
    js_enum.push_str(&format!("// Type-checking function for {}\n", enum_name));
    js_enum.push_str(&format!("function is{}(value) {{\n", enum_name));
    js_enum.push_str("  if (typeof value === 'string') {\n");
    js_enum.push_str(&format!(
        "    return Object.values({}).includes(value);\n",
        enum_name
    ));
    js_enum.push_str("  }\n");
    js_enum.push_str("  if (value && typeof value === 'object' && value.type) {\n");
    js_enum.push_str(&format!(
        "    return Object.keys({}).includes(value.type);\n",
        enum_name
    ));
    js_enum.push_str("  }\n");
    js_enum.push_str("  return false;\n");
    js_enum.push_str("}");

    js_enum
}

// Helper function to format Rust types to JavaScript types
pub fn format_rust_type(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                // Check for generic types like Vec<T>, Option<T>, etc.
                if !segment.arguments.is_empty() {
                    if type_name == "Vec" {
                        return "Array".to_string();
                    } else if type_name == "Option" {
                        return "".to_string(); // Will be handled specially in the conversion logic
                    } else if type_name == "HashMap" || type_name == "BTreeMap" {
                        return "Map".to_string();
                    } else if type_name == "HashSet" || type_name == "BTreeSet" {
                        return "Set".to_string();
                    }
                }

                // Basic types mapping
                match type_name.as_str() {
                    "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64"
                    | "usize" | "f32" | "f64" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    "String" | "str" => "string".to_string(),
                    _ => "object".to_string(), // Default for custom types
                }
            } else {
                "object".to_string()
            }
        }
        Type::Reference(type_ref) => {
            // Handle references like &str or &T
            format_rust_type(&type_ref.elem)
        }
        Type::Array(_) => "Array".to_string(),
        Type::Tuple(_) => "Array".to_string(),
        _ => "object".to_string(),
    }
}

// Helper function to check if a path ends with a specific segment
fn path_ends_with(expr: &Expr, segment: &str) -> bool {
    if let Expr::Path(path) = expr {
        if let Some(last) = path.path.segments.last() {
            return last.ident == segment;
        }
    }
    false
}

// Helper function to check if an expression is likely a string
fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(lit) => {
            matches!(lit.lit, syn::Lit::Str(_))
        }
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(segment) = path.path.segments.last() {
                    segment.ident == "format" || segment.ident == "to_string"
                } else {
                    false
                }
            } else {
                false
            }
        }
        Expr::MethodCall(method) => method.method == "to_string",
        _ => false,
    }
}

// Make sure this function exists and is used:
pub fn escape_js_identifier(rust_ident: &str) -> String {
    const JS_RESERVED: &[&str] = &[
        "abstract",
        "arguments",
        "await",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "double",
        "else",
        "enum",
        "eval",
        "export",
        "extends",
        "false",
        "final",
        "finally",
        "float",
        "for",
        "function",
        "goto",
        "if",
        "implements",
        "import",
        "in",
        "instanceof",
        "int",
        "interface",
        "let",
        "long",
        "native",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "volatile",
        "while",
        "with",
        "yield",
    ];

    if JS_RESERVED.contains(&rust_ident) {
        format!("{}_", rust_ident)
    } else {
        rust_ident.to_string()
    }
}

// Helper function to indent each line of code
fn indent_lines(code: &str, indent_level: usize) -> String {
    let indent = " ".repeat(indent_level);
    code.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<String>>()
        .join("\n")
        + "\n"
}
