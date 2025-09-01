use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;
use swc_ecma_codegen;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Block, Expr, Fields, ItemEnum, ItemStruct, Pat, Stmt, Type};
// use syn::{FnArg, ImplItem, ItemImpl, ReturnType, Signature};
use syn::{FnArg, ImplItem, ItemImpl};

/// Debug macro that only prints when MOJES_DEBUG environment variable is set
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if std::env::var("MOJES_DEBUG").is_ok() {
            println!($($arg)*);
        }
    };
}

/// Transpiler state for managing context and symbols during translation
pub struct TranspilerState {
    /// Symbol table for variable name mapping and type tracking
    symbol_table: HashMap<String, SymbolInfo>,
    /// Stack of scopes for proper variable resolution
    scope_stack: Vec<HashMap<String, String>>,
    /// Collected errors during transpilation
    errors: Vec<String>,
    /// Collected warnings during transpilation
    warnings: Vec<String>,
    /// Counter for generating unique temporary variable names
    temp_var_counter: usize,
    /// Current struct name for Self resolution
    current_struct_name: Option<String>,
    /// Whether we're currently in a static method context
    is_in_static_method: bool,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum BlockAction {
    Return,
    NoReturn,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub js_name: String,
    pub rust_type: String,
    pub is_mutable: bool,
}

impl TranspilerState {
    pub fn new() -> Self {
        TranspilerState {
            symbol_table: HashMap::new(),
            scope_stack: vec![HashMap::new()], // Start with global scope
            errors: Vec::new(),
            warnings: Vec::new(),
            temp_var_counter: 0,
            current_struct_name: None,
            is_in_static_method: false,
        }
    }

    /// Set the current struct name for Self resolution
    pub fn set_current_struct_name(&mut self, name: Option<String>) {
        self.current_struct_name = name;
    }

    /// Get the current struct name
    pub fn get_current_struct_name(&self) -> Option<&String> {
        self.current_struct_name.as_ref()
    }

    /// Set whether we're in a static method context
    pub fn set_in_static_method(&mut self, is_static: bool) {
        self.is_in_static_method = is_static;
    }

    /// Check if we're in a static method context
    pub fn is_in_static_method(&self) -> bool {
        self.is_in_static_method
    }
    /// Convert Pat to Param for function parameters
    pub fn pat_to_param(&self, pat: js::Pat) -> js::Param {
        js::Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat,
        }
    }
    pub fn expr_to_assign_target(&self, expr: js::Expr) -> Result<js::AssignTarget, String> {
        match expr {
            js::Expr::Ident(ident) => Ok(js::AssignTarget::Simple(js::SimpleAssignTarget::Ident(
                js::BindingIdent {
                    id: ident,
                    type_ann: None,
                },
            ))),
            js::Expr::Member(member) => Ok(js::AssignTarget::Simple(
                js::SimpleAssignTarget::Member(member),
            )),
            js::Expr::This(_) => Err("Cannot assign to 'this'".to_string()),
            _ => panic!("Unsupported assignment target expression: {:?}", &expr),
        }
    }
    pub fn mk_ident_name(&self, name: &str) -> js::IdentName {
        js::IdentName::new(name.into(), DUMMY_SP)
    }

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn declare_variable(&mut self, rust_name: String, js_name: String, is_mutable: bool) -> String {
        // Check for conflicts and generate a unique name if needed
        let unique_js_name = self.ensure_unique_js_name(&js_name);
        
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.insert(rust_name.clone(), unique_js_name.clone());
        }
        self.symbol_table.insert(
            rust_name,
            SymbolInfo {
                js_name: unique_js_name.clone(),
                rust_type: "unknown".to_string(),
                is_mutable,
            },
        );
        
        // Return the unique name so caller can use it
        unique_js_name
    }

    /// Ensure the JavaScript variable name is unique in the current scope chain
    fn ensure_unique_js_name(&self, proposed_name: &str) -> String {
        let mut candidate_name = proposed_name.to_string();
        let mut counter = 1;
        
        // Check if the name conflicts with any existing variable in any scope
        while self.js_name_exists_in_scope_chain(&candidate_name) {
            candidate_name = format!("{}_{}", proposed_name, counter);
            counter += 1;
        }
        
        candidate_name
    }
    
    /// Check if a JavaScript variable name exists anywhere in the scope chain
    fn js_name_exists_in_scope_chain(&self, js_name: &str) -> bool {
        // Check all scopes from innermost to outermost
        for scope in self.scope_stack.iter().rev() {
            for existing_js_name in scope.values() {
                if existing_js_name == js_name {
                    return true;
                }
            }
        }
        false
    }

    pub fn resolve_variable(&self, rust_name: &str) -> Option<String> {
        // Check scopes from innermost to outermost
        for scope in self.scope_stack.iter().rev() {
            if let Some(js_name) = scope.get(rust_name) {
                return Some(js_name.clone());
            }
        }

        // Fallback to symbol table
        self.symbol_table
            .get(rust_name)
            .map(|info| info.js_name.clone())
    }

    pub fn add_error(&mut self, error_message: String) {
        self.errors.push(error_message);
    }

    pub fn add_warning(&mut self, warning_message: String) {
        self.warnings.push(warning_message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }

    pub fn get_warnings(&self) -> &Vec<String> {
        &self.warnings
    }

    pub fn generate_temp_var(&mut self) -> String {
        self.temp_var_counter += 1;
        format!("_temp{}", self.temp_var_counter)
    }

    // Helper methods for creating common AST nodes
    pub fn mk_ident(&self, name: &str) -> js::Ident {
        js::Ident::new(name.into(), DUMMY_SP, SyntaxContext::empty())
    }

    pub fn mk_str_lit(&self, value: &str) -> js::Expr {
        self.mk_str_lit_double_quote(value)
    }

    pub fn mk_str_lit_double_quote(&self, value: &str) -> js::Expr {
        js::Expr::Lit(js::Lit::Str(js::Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: None,
        }))
    }

    pub fn mk_str_lit_single_quote(&self, value: &str) -> js::Expr {
        js::Expr::Lit(js::Lit::Str(js::Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: Some(swc_atoms::Atom::new(format!("'{}'", value))),
        }))
    }

    pub fn mk_num_lit(&self, value: f64) -> js::Expr {
        js::Expr::Lit(js::Lit::Num(js::Number {
            span: DUMMY_SP,
            value,
            raw: None,
        }))
    }

    pub fn mk_bool_lit(&self, value: bool) -> js::Expr {
        js::Expr::Lit(js::Lit::Bool(js::Bool {
            span: DUMMY_SP,
            value,
        }))
    }

    pub fn mk_null_lit(&self) -> js::Expr {
        js::Expr::Lit(js::Lit::Null(js::Null { span: DUMMY_SP }))
    }

    pub fn mk_undefined(&self) -> js::Expr {
        js::Expr::Ident(self.mk_ident("undefined"))
    }

    pub fn mk_this_expr(&self) -> js::Expr {
        js::Expr::This(js::ThisExpr { span: DUMMY_SP })
    }

    pub fn mk_member_expr(&self, obj: js::Expr, prop: &str) -> js::Expr {
        js::Expr::Member(js::MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(obj),
            prop: js::MemberProp::Ident(self.mk_ident_name(prop)),
        })
    }

    pub fn mk_call_expr(&self, callee: js::Expr, args: Vec<js::Expr>) -> js::Expr {
        let js_args: Vec<js::ExprOrSpread> = args
            .into_iter()
            .map(|expr| js::ExprOrSpread {
                spread: None,
                expr: Box::new(expr),
            })
            .collect();

        js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            callee: js::Callee::Expr(Box::new(callee)),
            args: js_args,
            type_args: None,
            ctxt: SyntaxContext::empty(),
        })
    }
    pub fn mk_iife(&self, stmts: Vec<js::Stmt>) -> js::Expr {
        self.mk_iife_with_this_context(stmts)
    }

    pub fn mk_iife_with_this_context(&self, stmts: Vec<js::Stmt>) -> js::Expr {
        let body = js::BlockStmt {
            span: DUMMY_SP,
            stmts,
            ctxt: SyntaxContext::empty(),
        };

        // Use traditional function instead of arrow function for proper this context
        let function = js::Function {
            params: vec![],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(body),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
            ctxt: SyntaxContext::empty(),
        };

        let wrapped_fn = js::Expr::Paren(js::ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(js::Expr::Fn(js::FnExpr {
                ident: None,
                function: Box::new(function),
            })),
        });

        // Use .call(this) with traditional function (this will work properly)
        let call_expr = self.mk_member_expr(wrapped_fn, "call");
        self.mk_call_expr(call_expr, vec![self.mk_this_expr()])
    }
    pub fn mk_iife_without_context(&self, stmts: Vec<js::Stmt>) -> js::Expr {
        let body = js::BlockStmt {
            span: DUMMY_SP,
            stmts,
            ctxt: SyntaxContext::empty(),
        };

        let arrow_fn = js::ArrowExpr {
            span: DUMMY_SP,
            params: vec![],
            body: Box::new(js::BlockStmtOrExpr::BlockStmt(body)),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
            ctxt: SyntaxContext::empty(),
        };

        // Wrap just the arrow function in parentheses for proper JavaScript syntax
        let wrapped_arrow = js::Expr::Paren(js::ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(js::Expr::Arrow(arrow_fn)),
        });

        self.mk_call_expr(wrapped_arrow, vec![])
    }
    pub fn mk_var_decl(&self, name: &str, init: Option<js::Expr>, is_const: bool) -> js::Stmt {
        let kind = if is_const {
            js::VarDeclKind::Const
        } else {
            js::VarDeclKind::Let
        };

        js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
            span: DUMMY_SP,
            kind,
            declare: false,
            decls: vec![js::VarDeclarator {
                span: DUMMY_SP,
                name: js::Pat::Ident(js::BindingIdent {
                    id: self.mk_ident(name),
                    type_ann: None,
                }),
                init: init.map(Box::new),
                definite: false,
            }],
            ctxt: SyntaxContext::empty(),
        })))
    }

    pub fn mk_return_stmt(&self, arg: Option<js::Expr>) -> js::Stmt {
        js::Stmt::Return(js::ReturnStmt {
            span: DUMMY_SP,
            arg: arg.map(Box::new),
        })
    }

    pub fn mk_expr_stmt(&self, expr: js::Expr) -> js::Stmt {
        js::Stmt::Expr(js::ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(expr),
        })
    }

    pub fn mk_binary_expr(&self, left: js::Expr, op: js::BinaryOp, right: js::Expr) -> js::Expr {
        js::Expr::Bin(js::BinExpr {
            span: DUMMY_SP,
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    pub fn mk_template_literal(&self, parts: Vec<String>, exprs: Vec<js::Expr>) -> js::Expr {
        // Ensure we have the right number of parts vs expressions
        // Template literals need parts.len() == exprs.len() + 1
        let mut final_parts = parts;
        let mut final_exprs = exprs;

        // Adjust parts to match expressions + 1
        while final_parts.len() < final_exprs.len() + 1 {
            final_parts.push("".to_string());
        }
        while final_parts.len() > final_exprs.len() + 1 {
            final_parts.pop();
        }

        let parts_len = final_parts.len();
        let quasis: Vec<js::TplElement> = final_parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| {
                let escaped_part = part.replace('`', "\\`");
                js::TplElement {
                    span: DUMMY_SP,
                    tail: i == parts_len - 1,
                    cooked: Some(escaped_part.clone().into()),
                    raw: swc_atoms::Atom::new(escaped_part),
                }
            })
            .collect();

        js::Expr::Tpl(js::Tpl {
            span: DUMMY_SP,
            exprs: final_exprs.into_iter().map(Box::new).collect(),
            quasis,
        })
    }
}

/// Generate JavaScript methods for a Rust impl block
pub fn generate_js_methods_for_impl_with_state(
    input_impl: &ItemImpl,
) -> Result<Vec<js::ModuleItem>, String> {
    let mut state = TranspilerState::new();

    let struct_name = if let syn::Type::Path(type_path) = &*input_impl.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident.to_string()
        } else {
            return Err("Could not determine struct name".to_string());
        }
    } else {
        return Err("Invalid impl type".to_string());
    };

    // Set the current struct name for Self resolution
    state.set_current_struct_name(Some(struct_name.clone()));

    let mut js_items = Vec::new();

    // Add header comment for the methods
    let header_comment = format!("// Methods for {}", struct_name);
    let comment_stmt = js::Stmt::Expr(js::ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(js::Expr::Ident(js::Ident::new(
            header_comment.into(),
            DUMMY_SP,
            SyntaxContext::empty(),
        ))),
    });
    js_items.push(js::ModuleItem::Stmt(comment_stmt));

    for item in &input_impl.items {
        if let ImplItem::Fn(method) = item {
            match generate_js_method(&struct_name, method, &mut state) {
                Ok(method_item) => js_items.push(method_item),
                Err(e) => state.add_error(e),
            }
        }
    }

    if state.has_errors() {
        return Err(format!("Transpilation errors: {:?}", state.get_errors()));
    }

    Ok(js_items)
}

/// Generate JavaScript method for a single Rust method
fn generate_js_method(
    struct_name: &str,
    method: &syn::ImplItemFn,
    state: &mut TranspilerState,
) -> Result<js::ModuleItem, String> {
    let method_name = method.sig.ident.to_string();
    let sig = &method.sig;

    // Check if this is a static method (no self parameter)
    let is_static = !sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)));

    // Extract non-self parameters (don't register in scope yet - will be done in function body)
    let params: Vec<js::Pat> = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Receiver(_) => None, // Skip self
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let js_param_name = escape_js_identifier(&param_name);
                        
                        Some(js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&js_param_name),
                            type_ann: None,
                        }))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Collect parameter information for scope registration
    let param_info: Vec<(String, String)> = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let js_param_name = escape_js_identifier(&param_name);
                        Some((param_name, js_param_name))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Set static method context before converting method body
    state.set_in_static_method(is_static);
    // Convert method body to JavaScript with parameter registration
    let body_stmts = rust_block_to_js_with_params_and_state(BlockAction::Return, &method.block, &param_info, state)?;
    // Reset static method context after conversion
    state.set_in_static_method(false);
    let body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: body_stmts,
        ctxt: SyntaxContext::empty(),
    };

    let function = js::Function {
        params: params.into_iter().map(|p| state.pat_to_param(p)).collect(),
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(body),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    };

    // Create the assignment statement
    let target = if is_static {
        // Static method: StructName.methodName
        state.mk_member_expr(js::Expr::Ident(state.mk_ident(struct_name)), &method_name)
    } else {
        // Instance method: StructName.prototype.methodName
        let prototype =
            state.mk_member_expr(js::Expr::Ident(state.mk_ident(struct_name)), "prototype");
        state.mk_member_expr(prototype, &method_name)
    };

    let assignment = js::Expr::Assign(js::AssignExpr {
        span: DUMMY_SP,
        op: js::AssignOp::Assign,
        left: state.expr_to_assign_target(target)?,
        right: Box::new(js::Expr::Fn(js::FnExpr {
            ident: None,
            function: Box::new(function),
        })),
    });

    Ok(js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(assignment),
    })))
}

fn can_be_statement(expr: &Expr) -> bool {
    matches!(expr, Expr::While(_) | Expr::ForLoop(_) | Expr::Loop(_))
}

/*
fn convert_while_to_stmt(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
}
fn convert_for_to_stmt(for_expr: &syn::ExprForLoop, state: &mut TranspilerState) -> Result<js::Stmt, String>
{
}
fn convert_loop_to_stmt(loop_expr: &syn::ExprLoop, state: &mut TranspilerState) -> Result<js::Stmt, String>
{
}
*/

/// Core function that converts Rust if expression to JavaScript if statement
fn convert_if_to_stmt(
    block_action: BlockAction,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    debug_print!("DEBUG IF: {:?} {:?}", &block_action, &if_expr);
    // panic!("XXX");
    // Check if this is an if-let expression
    if let Some(if_let_stmt) = handle_if_let_as_stmt(block_action, if_expr, state)? {
        return Ok(if_let_stmt);
    }

    // Regular if expression
    let test = rust_expr_to_js_with_state(&if_expr.cond, state)?;
    let consequent_stmts = rust_block_to_js_with_state(block_action, &if_expr.then_branch, state)?;

    let consequent = js::Stmt::Block(js::BlockStmt {
        span: DUMMY_SP,
        stmts: consequent_stmts,
        ctxt: SyntaxContext::empty(),
    });

    // Handle else branch
    let alternate = if let Some((_, else_branch)) = &if_expr.else_branch {
        match &**else_branch {
            Expr::Block(else_block) => {
                let else_stmts =
                    rust_block_to_js_with_state(block_action, &else_block.block, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: else_stmts,
                    ctxt: SyntaxContext::empty(),
                })))
            }
            Expr::If(nested_if) => {
                // Handle else if - recursively convert
                Some(Box::new(convert_if_to_stmt(
                    block_action,
                    nested_if,
                    state,
                )?))
            }
            _ => {
                // Single expression else
                let else_expr = rust_expr_to_js_with_state(else_branch, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![state.mk_expr_stmt(else_expr)],
                    ctxt: SyntaxContext::empty(),
                })))
            }
        }
    } else {
        None
    };

    Ok(js::Stmt::If(js::IfStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(consequent),
        alt: alternate,
    }))
}

/// Enhanced if-let handling that returns a statement
fn handle_if_let_as_stmt(
    block_action: BlockAction,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<Option<js::Stmt>, String> {
    // Check if this is an "if let" expression by examining the condition
    if let Expr::Let(expr_let) = &*if_expr.cond {
        let pat = &expr_let.pat;
        let init_expr = &expr_let.expr;

        // Handle "if let Some(x) = ..." pattern
        if let Pat::TupleStruct(tuple_struct) = &**pat {
            if let Some(last_segment) = tuple_struct.path.segments.last() {
                if last_segment.ident == "Some" && !tuple_struct.elems.is_empty() {
                    return Ok(Some(convert_if_let_some_to_stmt(
                        block_action,
                        tuple_struct,
                        init_expr,
                        if_expr,
                        state,
                    )?));
                }
            }
        }

        // Handle other if-let patterns (None, custom enums, etc.)
        return Ok(Some(convert_generic_if_let_to_stmt(
            block_action,
            pat,
            init_expr,
            if_expr,
            state,
        )?));
    }

    Ok(None)
}

/// Convert "if let Some(x) = expr" to optimized JavaScript statement
fn convert_if_let_some_to_stmt(
    block_action: BlockAction,
    tuple_struct: &syn::PatTupleStruct,
    init_expr: &Expr,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    let matched_expr = rust_expr_to_js_with_state(init_expr, state)?;
    let then_stmts = rust_block_to_js_with_state(block_action, &if_expr.then_branch, state)?;

    // Create null/undefined check condition
    let not_null = state.mk_binary_expr(
        matched_expr.clone(),
        js::BinaryOp::NotEqEq,
        state.mk_null_lit(),
    );
    let not_undefined = state.mk_binary_expr(
        matched_expr.clone(),
        js::BinaryOp::NotEqEq,
        state.mk_undefined(),
    );
    let condition = state.mk_binary_expr(not_null, js::BinaryOp::LogicalAnd, not_undefined);

    // Handle variable binding if present
    let mut consequent_stmts = Vec::new();
    if let Some(inner_pat) = tuple_struct.elems.first() {
        if let Pat::Ident(pat_ident) = inner_pat {
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            let unique_js_var_name = state.declare_variable(var_name, js_var_name, false);

            // Add: const x = matched_expr;
            consequent_stmts.push(state.mk_var_decl(&unique_js_var_name, Some(matched_expr), true));
        }
    }

    // Add the then branch statements
    consequent_stmts.extend(then_stmts);

    // Handle else branch
    let alternate = if let Some((_, else_branch)) = &if_expr.else_branch {
        match &**else_branch {
            Expr::Block(else_block) => {
                let else_stmts =
                    rust_block_to_js_with_state(block_action, &else_block.block, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: else_stmts,
                    ctxt: SyntaxContext::empty(),
                })))
            }
            Expr::If(nested_if) => Some(Box::new(convert_if_to_stmt(
                block_action,
                nested_if,
                state,
            )?)),
            _ => {
                let else_expr = rust_expr_to_js_with_state(else_branch, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![state.mk_expr_stmt(else_expr)],
                    ctxt: SyntaxContext::empty(),
                })))
            }
        }
    } else {
        None
    };

    Ok(js::Stmt::If(js::IfStmt {
        span: DUMMY_SP,
        test: Box::new(condition),
        cons: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: consequent_stmts,
            ctxt: SyntaxContext::empty(),
        })),
        alt: alternate,
    }))
}

/// Convert generic if-let patterns to JavaScript statements
fn convert_generic_if_let_to_stmt(
    block_action: BlockAction,
    pat: &Pat,
    init_expr: &Expr,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    let matched_expr = rust_expr_to_js_with_state(init_expr, state)?;
    let temp_var = state.generate_temp_var();

    let mut stmts = vec![state.mk_var_decl(&temp_var, Some(matched_expr), true)];

    let (condition, mut binding_stmts) = handle_pattern_binding(pat, &temp_var, state)?;
    let then_stmts = rust_block_to_js_with_state(block_action, &if_expr.then_branch, state)?;

    // Combine binding statements with then branch
    binding_stmts.extend(then_stmts);

    // Handle else branch
    let alternate = if let Some((_, else_branch)) = &if_expr.else_branch {
        match &**else_branch {
            Expr::Block(else_block) => {
                let else_stmts =
                    rust_block_to_js_with_state(block_action, &else_block.block, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: else_stmts,
                    ctxt: SyntaxContext::empty(),
                })))
            }
            Expr::If(nested_if) => Some(Box::new(convert_if_to_stmt(
                block_action,
                nested_if,
                state,
            )?)),
            _ => {
                let else_expr = rust_expr_to_js_with_state(else_branch, state)?;
                Some(Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![state.mk_expr_stmt(else_expr)],
                    ctxt: SyntaxContext::empty(),
                })))
            }
        }
    } else {
        None
    };

    let if_stmt = js::Stmt::If(js::IfStmt {
        span: DUMMY_SP,
        test: Box::new(condition),
        cons: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: binding_stmts,
            ctxt: SyntaxContext::empty(),
        })),
        alt: alternate,
    });

    stmts.push(if_stmt);

    // Return a block containing the temp variable and the if statement
    Ok(js::Stmt::Block(js::BlockStmt {
        span: DUMMY_SP,
        stmts,
        ctxt: SyntaxContext::empty(),
    }))
}

/// Modified handle_if_expr that reuses the core logic
fn handle_if_expr(
    block_action: BlockAction,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Check if this is an if-let expression that should be handled specially
    if let Some(if_let_expr) = handle_if_let_as_expr(block_action, if_expr, state)? {
        return Ok(if_let_expr);
    }

    // For regular if expressions, convert to statement and wrap in IIFE
    let if_stmt = convert_if_to_stmt(block_action, if_expr, state)?;

    // Wrap in IIFE and add a default return
    let mut stmts = vec![if_stmt];
    stmts.push(state.mk_return_stmt(Some(state.mk_undefined())));

    Ok(state.mk_iife(stmts))
}

/// Handle if-let expressions that need to return values (expression context)
fn handle_if_let_as_expr(
    block_action: BlockAction,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<Option<js::Expr>, String> {
    if let Expr::Let(expr_let) = &*if_expr.cond {
        let pat = &expr_let.pat;
        let init_expr = &expr_let.expr;

        // Handle "if let Some(x) = ..." pattern in expression context
        if let Pat::TupleStruct(tuple_struct) = &**pat {
            if let Some(last_segment) = tuple_struct.path.segments.last() {
                if last_segment.ident == "Some" && !tuple_struct.elems.is_empty() {
                    return Ok(Some(convert_if_let_some_to_expr(
                        block_action,
                        tuple_struct,
                        init_expr,
                        if_expr,
                        state,
                    )?));
                }
            }
        }
    }

    Ok(None)
}

/// Convert "if let Some(x) = expr" to IIFE expression (like legacy version)
fn convert_if_let_some_to_expr(
    block_action: BlockAction,
    tuple_struct: &syn::PatTupleStruct,
    init_expr: &Expr,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let matched_expr = rust_expr_to_js_with_state(init_expr, state)?;
    let temp_var = "_temp";

    let mut stmts = vec![state.mk_var_decl(temp_var, Some(matched_expr), true)];

    // Create condition
    let not_null = state.mk_binary_expr(
        js::Expr::Ident(state.mk_ident(temp_var)),
        js::BinaryOp::NotEqEq,
        state.mk_null_lit(),
    );
    let not_undefined = state.mk_binary_expr(
        js::Expr::Ident(state.mk_ident(temp_var)),
        js::BinaryOp::NotEqEq,
        state.mk_undefined(),
    );
    let condition = state.mk_binary_expr(not_null, js::BinaryOp::LogicalAnd, not_undefined);

    // Handle variable binding and then branch
    let mut then_stmts = Vec::new();
    if let Some(inner_pat) = tuple_struct.elems.first() {
        if let Pat::Ident(pat_ident) = inner_pat {
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            state.declare_variable(var_name, js_var_name.clone(), false);

            then_stmts.push(state.mk_var_decl(
                &js_var_name,
                Some(js::Expr::Ident(state.mk_ident(temp_var))),
                true,
            ));
        }
    }

    let then_body_stmts = rust_block_to_js_with_state(block_action, &if_expr.then_branch, state)?;
    then_stmts.extend(then_body_stmts);

    // Handle else branch
    let else_stmts = if let Some((_, else_branch)) = &if_expr.else_branch {
        match &**else_branch {
            Expr::Block(else_block) => {
                rust_block_to_js_with_state(block_action, &else_block.block, state)?
            }
            _ => {
                let else_expr = rust_expr_to_js_with_state(else_branch, state)?;
                vec![state.mk_return_stmt(Some(else_expr))]
            }
        }
    } else {
        vec![]
    };

    let if_stmt = js::Stmt::If(js::IfStmt {
        span: DUMMY_SP,
        test: Box::new(condition),
        cons: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: then_stmts,
            ctxt: SyntaxContext::empty(),
        })),
        alt: if !else_stmts.is_empty() {
            Some(Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: else_stmts,
                ctxt: SyntaxContext::empty(),
            })))
        } else {
            None
        },
    });

    stmts.push(if_stmt);
    stmts.push(state.mk_return_stmt(Some(state.mk_undefined())));

    Ok(state.mk_iife(stmts))
}

/// Context-aware wrapper for if expressions
pub fn handle_if_context_aware(
    block_action: BlockAction,
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
    in_statement_context: bool,
) -> Result<Either<js::Stmt, js::Expr>, String> {
    if in_statement_context {
        convert_if_to_stmt(block_action, if_expr, state).map(Either::Left)
    } else {
        handle_if_expr(block_action, if_expr, state).map(Either::Right)
    }
}

/// Convert Rust block to JavaScript statements
pub fn rust_block_to_js_with_state(
    block_action: BlockAction,
    block: &Block,
    state: &mut TranspilerState,
) -> Result<Vec<js::Stmt>, String> {
    rust_block_to_js_with_params_and_state(block_action, block, &[], state)
}

/// Convert Rust block to JavaScript statements with function parameters
pub fn rust_block_to_js_with_params_and_state(
    block_action: BlockAction,
    block: &Block,
    params: &[(String, String)], // (rust_name, js_name) pairs
    state: &mut TranspilerState,
) -> Result<Vec<js::Stmt>, String> {
    let mut js_stmts = Vec::new();
    debug_print!("DEBUG BLK: {:?} {:?}", &block_action, &block);
    if block_action == BlockAction::NoReturn {
        // panic!("XXX");
    }

    state.enter_scope();
    
    // Register function parameters in the function body scope
    for (rust_name, js_name) in params {
        state.declare_variable(rust_name.clone(), js_name.clone(), false);
    }

    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(local) => {
                debug_print!("DEBUG BLOCK LOCAL: {:?}", &local);
                let js_stmt = handle_local_statement(block_action, local, state)?;
                js_stmts.push(js_stmt);
            }
            Stmt::Item(item) => match item {
                syn::Item::Fn(item_fn) => {
                    let js_stmt = handle_function_definition(item_fn, state)?;
                    js_stmts.push(js_stmt);
                }
                syn::Item::Struct(item_struct) => {
                    panic!(
                        "Struct definitions in blocks not fully supported: {:?}",
                        item
                    );
                }
                syn::Item::Enum(item_enum) => {
                    panic!("Enum definitions in blocks not fully supported: {:?}", item);
                }
                _ => {
                    panic!("Unsupported item type in block: {:?}", item);
                }
            },
            Stmt::Expr(expr, semi) => {
                match expr {
                    Expr::Return(ret) => {
                        // Handle return expressions properly
                        if let Some(return_expr) = &ret.expr {
                            let js_expr = rust_expr_to_js_with_state(return_expr, state)?;
                            js_stmts.push(state.mk_return_stmt(Some(js_expr)));
                        } else {
                            js_stmts.push(state.mk_return_stmt(None));
                        }
                    }
                    Expr::ForLoop(for_expr) => {
                        // Generate direct statement
                        let for_stmt = convert_for_to_stmt_enhanced(for_expr, state)?;
                        js_stmts.push(for_stmt);
                    }
                    Expr::While(while_expr) => {
                        // Generate direct statement
                        let while_stmt =
                            convert_while_to_stmt_legacy_compatible(while_expr, state)?;
                        js_stmts.push(while_stmt);
                    }
                    Expr::If(if_expr) => {
                        // Generate direct statement
                        let if_stmt = convert_if_to_stmt(block_action, if_expr, state)?;
                        debug_print!("DEBUG IFIN BLOCK: {:?}", if_stmt);
                        js_stmts.push(if_stmt);
                    }
                    x => {
                        debug_print!(
                            "DEBUG EXPR IN BLOCK (block action: {:?}) : {:?}, semi: {:?}",
                            &block_action, &x, &semi
                        );
                        let js_expr = rust_expr_to_js_with_state(expr, state)?;

                        if semi.is_some() {
                            // Expression with semicolon - treat as statement
                            js_stmts.push(state.mk_expr_stmt(js_expr));
                        } else {
                            // Expression without semicolon - only return if it's the last statement
                            let is_last_stmt = stmt == block.stmts.last().unwrap();
                            if is_last_stmt && block_action == BlockAction::Return {
                                debug_print!("DEBUG BLOCK: return statement: {:?}", block_action);
                                js_stmts.push(state.mk_return_stmt(Some(js_expr)));
                            } else {
                                js_stmts.push(state.mk_expr_stmt(js_expr));
                            }
                        }
                    }
                }
            }
            Stmt::Macro(mac_stmt) => {
                let js_expr = handle_macro_expr(&mac_stmt.mac, state)?;
                js_stmts.push(state.mk_expr_stmt(js_expr));
            }
            _ => {
                state.add_warning(format!("Unsupported statement type: {:?}", stmt));
                panic!("Unsupported statement type: {:?}", stmt);
            }
        }
    }

    state.exit_scope();
    Ok(js_stmts)
}

/// Handle function definitions inside blocks
fn handle_function_definition(
    item_fn: &syn::ItemFn,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    let func_name = item_fn.sig.ident.to_string();
    let js_func_name = escape_js_identifier(&func_name);

    // Convert parameters (don't register in scope yet - will be done in function body)
    let params: Vec<js::Pat> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                syn::FnArg::Receiver(_) => None, // Skip self parameters in nested functions
                syn::FnArg::Typed(pat_type) => {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let js_param_name = escape_js_identifier(&param_name);
                        Some(js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&js_param_name),
                            type_ann: None,
                        }))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Collect parameter information for scope registration
    let param_info: Vec<(String, String)> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat_type) => {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let js_param_name = escape_js_identifier(&param_name);
                        Some((param_name, js_param_name))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Convert function body with parameter registration
    let body_stmts = rust_block_to_js_with_params_and_state(BlockAction::Return, &item_fn.block, &param_info, state)?;

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
        ident: state.mk_ident(&js_func_name),
        declare: false,
        function: Box::new(function),
    };

    // Declare the function in the current scope
    state.declare_variable(func_name, js_func_name, false);

    Ok(js::Stmt::Decl(js::Decl::Fn(func_decl)))
}

pub fn rust_expr_to_js_with_state(
    expr: &Expr,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    rust_expr_to_js_with_action_and_state(BlockAction::NoReturn, expr, state)
}

pub fn rust_expr_misc(
    block_action: BlockAction,
    expr: &Expr,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    Err("test".to_string())
}

/// Convert Rust expression to JavaScript expression
pub fn rust_expr_to_js_with_action_and_state(
    block_action: BlockAction,
    expr: &Expr,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    debug_print!("DEBUG EXPR: {:?}, {:?}", block_action, &expr);
    match expr {
        // Handle literals
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => {
                let value = s.value();
                // Properly escape the string for JavaScript
                let escaped = value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t")
                    .replace('\r', "\\r");
                Ok(state.mk_str_lit(&escaped))
            }

            syn::Lit::Int(i) => {
                let value = i
                    .base10_parse::<f64>()
                    .map_err(|e| format!("Failed to parse integer: {}", e))?;
                Ok(state.mk_num_lit(value))
            }
            syn::Lit::Float(f) => {
                let value = f
                    .base10_parse::<f64>()
                    .map_err(|e| format!("Failed to parse float: {}", e))?;
                Ok(state.mk_num_lit(value))
            }
            syn::Lit::Bool(b) => Ok(state.mk_bool_lit(b.value())),
            syn::Lit::Char(c) => Ok(state.mk_str_lit(&c.value().to_string())),
            _ => panic!("Unsupported literal type: {:?}", &lit),
        },

        // Handle reference expressions
        Expr::Reference(ref_expr) => handle_reference_expr(ref_expr, state),

        // Handle paths (variables, constants)
        Expr::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
                let ident_str = last_segment.ident.to_string();

                match ident_str.as_str() {
                    "self" => Ok(state.mk_this_expr()),
                    "Self" => {
                        // Replace Self with the current struct name
                        if let Some(struct_name) = state.get_current_struct_name() {
                            Ok(js::Expr::Ident(state.mk_ident(struct_name)))
                        } else {
                            // Fallback to "Self" if no struct context
                            Ok(js::Expr::Ident(state.mk_ident("Self")))
                        }
                    },
                    "None" => Ok(state.mk_null_lit()),
                    "true" | "false" => Ok(js::Expr::Ident(state.mk_ident(&ident_str))),
                    _ => {
                        // Try to resolve variable
                        let js_name = state
                            .resolve_variable(&ident_str)
                            .unwrap_or_else(|| escape_js_identifier(&ident_str));
                        Ok(js::Expr::Ident(state.mk_ident(&js_name)))
                    }
                }
            } else {
                Err("Invalid path expression".to_string())
            }
        }

        // Handle binary operations
        Expr::Binary(bin) => {
            let left = rust_expr_to_js_with_state(&bin.left, state)?;
            let right = rust_expr_to_js_with_state(&bin.right, state)?;

            let js_op = match &bin.op {
                syn::BinOp::AddAssign(_) => Some(js::AssignOp::AddAssign),
                syn::BinOp::SubAssign(_) => Some(js::AssignOp::SubAssign),
                syn::BinOp::MulAssign(_) => Some(js::AssignOp::MulAssign),
                syn::BinOp::DivAssign(_) => Some(js::AssignOp::DivAssign),
                syn::BinOp::RemAssign(_) => Some(js::AssignOp::ModAssign),
                syn::BinOp::BitXorAssign(_) => Some(js::AssignOp::BitXorAssign),
                syn::BinOp::BitAndAssign(_) => Some(js::AssignOp::BitAndAssign),
                syn::BinOp::BitOrAssign(_) => Some(js::AssignOp::BitOrAssign),
                syn::BinOp::ShlAssign(_) => Some(js::AssignOp::LShiftAssign),
                syn::BinOp::ShrAssign(_) => Some(js::AssignOp::RShiftAssign),
                _ => None,
            };
            if let Some(js_op) = js_op {
                Ok(js::Expr::Assign(js::AssignExpr {
                    span: DUMMY_SP,
                    op: js_op,
                    left: state.expr_to_assign_target(left)?,
                    right: Box::new(right),
                }))
            } else {
                let js_op = match &bin.op {
                    syn::BinOp::Add(_) => {
                        // Check for string concatenation
                        if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                            // Use template literal for string concatenation
                            return Ok(state.mk_template_literal(
                                vec!["".to_string(), "".to_string()],
                                vec![left, right],
                            ));
                        } else {
                            js::BinaryOp::Add
                        }
                    }
                    syn::BinOp::Sub(_) => js::BinaryOp::Sub,
                    syn::BinOp::Mul(_) => js::BinaryOp::Mul,
                    syn::BinOp::Div(_) => js::BinaryOp::Div,
                    syn::BinOp::Rem(_) => js::BinaryOp::Mod,
                    syn::BinOp::And(_) => js::BinaryOp::LogicalAnd,
                    syn::BinOp::Or(_) => js::BinaryOp::LogicalOr,
                    syn::BinOp::BitXor(_) => js::BinaryOp::BitXor,
                    syn::BinOp::BitAnd(_) => js::BinaryOp::BitAnd,
                    syn::BinOp::BitOr(_) => js::BinaryOp::BitOr,
                    syn::BinOp::Shl(_) => js::BinaryOp::LShift,
                    syn::BinOp::Shr(_) => js::BinaryOp::RShift,
                    syn::BinOp::Eq(_) => js::BinaryOp::EqEqEq,
                    syn::BinOp::Lt(_) => js::BinaryOp::Lt,
                    syn::BinOp::Le(_) => js::BinaryOp::LtEq,
                    syn::BinOp::Ne(_) => js::BinaryOp::NotEqEq,
                    syn::BinOp::Ge(_) => js::BinaryOp::GtEq,
                    syn::BinOp::Gt(_) => js::BinaryOp::Gt,
                    _ => panic!("Unsupported binary operator {:?}", &bin.op),
                };

                Ok(state.mk_binary_expr(left, js_op, right))
            }
        }

        // Handle unary operations
        Expr::Unary(unary) => {
            let operand = rust_expr_to_js_with_state(&unary.expr, state)?;
            let js_op = match &unary.op {
                syn::UnOp::Not(_) => js::UnaryOp::Bang,
                syn::UnOp::Neg(_) => js::UnaryOp::Minus,
                syn::UnOp::Deref(_) => return Ok(operand), // Dereference is no-op in JS
                _ => panic!("Unsupported unary operator {:?}", &unary.op),
            };

            Ok(js::Expr::Unary(js::UnaryExpr {
                span: DUMMY_SP,
                op: js_op,
                arg: Box::new(operand),
            }))
        }

        // Handle method calls
        Expr::MethodCall(method_call) => handle_method_call(method_call, state),

        // Handle function calls
        Expr::Call(call) => handle_function_call(call, state),

        // Handle field access
        Expr::Field(field) => {
            let base = rust_expr_to_js_with_state(&field.base, state)?;

            match &field.member {
                syn::Member::Named(ident) => {
                    // Named field access: obj.field
                    // Handle raw identifiers by removing the r# prefix
                    let member_name = ident.to_string();
                    let clean_member_name = if member_name.starts_with("r#") {
                        &member_name[2..] // Remove "r#" prefix
                    } else {
                        &member_name
                    };
                    Ok(state.mk_member_expr(base, clean_member_name))
                }
                syn::Member::Unnamed(index) => {
                    // Tuple field access: obj[0]
                    let index_expr = state.mk_num_lit(index.index as f64);
                    Ok(js::Expr::Member(js::MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(base),
                        prop: js::MemberProp::Computed(js::ComputedPropName {
                            span: DUMMY_SP,
                            expr: Box::new(index_expr),
                        }),
                    }))
                }
            }
        }

        // Handle if expressions
        Expr::If(if_expr) => handle_if_expr(block_action, if_expr, state),

        // Handle block expressions
        Expr::Block(block_expr) => {
            debug_print!("DEBUG EXPR block action: {:?}", block_action);
            let stmts = rust_block_to_js_with_state(block_action, &block_expr.block, state)?;
            Ok(state.mk_iife(stmts))
        }

        // Handle array literals
        Expr::Array(array) => {
            let elements: Result<Vec<_>, _> = array
                .elems
                .iter()
                .map(|elem| rust_expr_to_js_with_state(elem, state))
                .collect();

            let js_elements: Vec<Option<js::ExprOrSpread>> = elements?
                .into_iter()
                .map(|expr| {
                    Some(js::ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    })
                })
                .collect();

            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: js_elements,
            }))
        }

        // Handle index expressions
        Expr::Index(index) => {
            let obj = rust_expr_to_js_with_state(&index.expr, state)?;
            let prop = rust_expr_to_js_with_state(&index.index, state)?;

            Ok(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(obj),
                prop: js::MemberProp::Computed(js::ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(prop),
                }),
            }))
        }

        // Handle assignments
        Expr::Assign(assign) => {
            let left = rust_expr_to_js_with_state(&assign.left, state)?;
            let right = rust_expr_to_js_with_state(&assign.right, state)?;

            Ok(js::Expr::Assign(js::AssignExpr {
                span: DUMMY_SP,
                op: js::AssignOp::Assign,
                left: state.expr_to_assign_target(left)?,
                right: Box::new(right),
            }))
        }

        // Handle return expressions
        Expr::Return(ret) => {
            if let Some(return_expr) = &ret.expr {
                let js_expr = rust_expr_to_js_with_state(return_expr, state)?;
                // Return expressions in JavaScript are statements, not expressions
                // We'll handle this at the statement level
                Ok(js_expr)
            } else {
                Ok(state.mk_undefined())
            }
        }

        // Handle macro expressions
        Expr::Macro(macro_expr) => handle_macro_expr(&macro_expr.mac, state),

        // Handle struct expressions
        Expr::Struct(struct_expr) => handle_struct_expr(struct_expr, state),

        // Handle for loops
        Expr::ForLoop(for_expr) => handle_for_expr(for_expr, state),

        // Handle match expressions
        Expr::Match(match_expr) => handle_match_expr(match_expr, state),

        Expr::Paren(paren) => handle_paren_expr(paren, state),

        Expr::Closure(closure) => handle_closure_expr(closure, state),

        // Handle async expressions
        Expr::Async(async_expr) => handle_async_expr(async_expr, state),

        // Handle await expressions
        Expr::Await(await_expr) => handle_await_expr(await_expr, state),

        // Handle if-let expressions
        Expr::Let(let_expr) => {
            // if let Some(value) = complex_option becomes a condition check
            match &*let_expr.pat {
                Pat::TupleStruct(tuple_struct) => {
                    if let Some(segment) = tuple_struct.path.segments.last() {
                        if segment.ident == "Some" {
                            let expr_js = rust_expr_to_js_with_state(&let_expr.expr, state)?;
                            // Check that value is not null/undefined
                            let not_null = state.mk_binary_expr(
                                expr_js.clone(),
                                js::BinaryOp::NotEqEq,
                                state.mk_null_lit(),
                            );
                            let not_undefined = state.mk_binary_expr(
                                expr_js,
                                js::BinaryOp::NotEqEq,
                                state.mk_undefined(),
                            );
                            Ok(state.mk_binary_expr(
                                not_null,
                                js::BinaryOp::LogicalAnd,
                                not_undefined,
                            ))
                        } else {
                            panic!(
                                "Unsupported tuple struct in let expression: {:?}",
                                segment.ident
                            );
                        }
                    } else {
                        panic!(
                            "Invalid tuple struct path in let expression: {:?}",
                            tuple_struct.path
                        );
                    }
                }
                _ => panic!("Unsupported pattern in let expression: {:?}", let_expr.pat),
            }
        }

        // Handle additional expression types
        Expr::While(while_expr) => handle_while_expr(while_expr, state),
        Expr::ForLoop(for_expr) => handle_for_expr(for_expr, state),
        Expr::Loop(loop_expr) => handle_loop_expr(loop_expr, state),

        Expr::Match(match_expr) => handle_match_expr(match_expr, state),
        Expr::Closure(closure) => handle_closure_expr(closure, state),
        Expr::Struct(struct_expr) => handle_struct_expr(struct_expr, state),
        Expr::Range(range_expr) => handle_range_expr(range_expr, state),

        Expr::Paren(paren) => handle_paren_expr(paren, state),

        Expr::Break(break_expr) => {
            if let Some(value) = &break_expr.expr {
                let value_js = rust_expr_to_js_with_state(value, state)?;
                state.add_warning("Break with value requires special handling".to_string());
                Ok(value_js)
            } else {
                state.add_warning("Break statement converted to undefined".to_string());
                Ok(state.mk_undefined())
            }
        }

        Expr::Continue(_) => {
            state.add_warning("Continue statement converted to undefined".to_string());
            Ok(state.mk_undefined())
        }

        Expr::Tuple(tuple) => {
            // Convert tuple to array
            let elements: Result<Vec<_>, _> = tuple
                .elems
                .iter()
                .map(|elem| rust_expr_to_js_with_state(elem, state))
                .collect();

            let js_elements: Vec<Option<js::ExprOrSpread>> = elements?
                .into_iter()
                .map(|expr| {
                    Some(js::ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    })
                })
                .collect();

            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: js_elements,
            }))
        }

        Expr::Paren(paren) => handle_paren_expr(paren, state),

        // Handle cast expressions
        Expr::Cast(cast_expr) => {
            let inner_expr = rust_expr_to_js_with_state(&cast_expr.expr, state)?;

            // Convert the target type to determine cast function
            let cast_fn = match &*cast_expr.ty {
                Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        match segment.ident.to_string().as_str() {
                            "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32"
                            | "u64" | "usize" | "f32" | "f64" => "Number",
                            "String" | "str" => "String",
                            "bool" => "Boolean",
                            _ => {
                                // For unknown types, add a comment
                                return Ok(js::Expr::Ident(js::Ident::new(
                                    format!(
                                        "{} /* was {} as {} in Rust */",
                                        match ast_to_code_trimmed(&[js::ModuleItem::Stmt(
                                            js::Stmt::Expr(js::ExprStmt {
                                                span: DUMMY_SP,
                                                expr: Box::new(inner_expr.clone()),
                                            })
                                        )]) {
                                            Ok(code) =>
                                                code.trim_end_matches(';').trim().to_string(),
                                            Err(_) => "expr".to_string(),
                                        },
                                        "expr",
                                        format_rust_type(&cast_expr.ty)
                                    )
                                    .into(),
                                    DUMMY_SP,
                                    SyntaxContext::empty(),
                                )));
                            }
                        }
                    } else {
                        "Object"
                    }
                }
                _ => "Object",
            };

            // Create cast function call like Number(expr)
            Ok(state.mk_call_expr(js::Expr::Ident(state.mk_ident(cast_fn)), vec![inner_expr]))
        }
        // Handle repeat expressions [value; count]
        Expr::Repeat(repeat_expr) => {
            let value = rust_expr_to_js_with_state(&repeat_expr.expr, state)?;
            let count = rust_expr_to_js_with_state(&repeat_expr.len, state)?;

            // Generate: Array.from({length: count}, () => value)
            let array_from = state.mk_member_expr(js::Expr::Ident(state.mk_ident("Array")), "from");

            // Create {length: count} object
            let length_obj = js::Expr::Object(js::ObjectLit {
                span: DUMMY_SP,
                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("length")),
                        value: Box::new(count),
                    },
                )))],
            });

            // Create () => value arrow function
            let arrow_fn = js::ArrowExpr {
                span: DUMMY_SP,
                params: vec![],
                body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(value))),
                is_async: false,
                is_generator: false,
                type_params: None,
                return_type: None,
                ctxt: SyntaxContext::empty(),
            };

            Ok(state.mk_call_expr(array_from, vec![length_obj, js::Expr::Arrow(arrow_fn)]))
        }

        // Handle verbatim expressions (unparseable token streams)
        Expr::Verbatim(verbatim) => {
            // Verbatim expressions are token streams that syn couldn't parse
            // Often these are empty or contain syntax that doesn't translate well
            let token_string = verbatim.to_string();

            if token_string.trim().is_empty() {
                // Empty verbatim expression - return undefined
                Ok(state.mk_undefined())
            } else {
                // Non-empty verbatim - add warning and return comment
                state.add_warning(format!(
                    "Verbatim expression not fully supported: {}",
                    token_string
                ));

                // Return as a comment expression
                Ok(js::Expr::Ident(js::Ident::new(
                    format!("/* verbatim: {} */", token_string.trim()).into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                )))
            }
        }

        Expr::Try(try_expr) => {
            let inner = rust_expr_to_js_with_action_and_state(block_action, &try_expr.expr, state)?;
            // Generate an IIFE that handles the try operation
            let temp_var = state.generate_temp_var();

            let stmts = vec![
                // const _temp1 = some_function();
                state.mk_var_decl(&temp_var, Some(inner), true),
                // if (_temp1 && _temp1.error) return _temp1;
                js::Stmt::If(js::IfStmt {
                    span: DUMMY_SP,
                    test: Box::new(state.mk_binary_expr(
                        js::Expr::Ident(state.mk_ident(&temp_var)),
                        js::BinaryOp::LogicalAnd,
                        js::Expr::Member(js::MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(js::Expr::Ident(state.mk_ident(&temp_var))),
                            prop: js::MemberProp::Ident(state.mk_ident_name("error")),
                        }),
                    )),
                    cons: Box::new(js::Stmt::Return(js::ReturnStmt {
                        span: DUMMY_SP,
                        arg: Some(Box::new(js::Expr::Ident(state.mk_ident(&temp_var)))),
                    })),
                    alt: None,
                }),
                // return _temp1.ok;  (unwrap the success value)
                js::Stmt::Return(js::ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(js::Expr::Member(js::MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(js::Expr::Ident(state.mk_ident(&temp_var))),
                        prop: js::MemberProp::Ident(state.mk_ident_name("ok")),
                    }))),
                }),
            ];

            Ok(state.mk_iife(stmts))
        }
        _ => {
            state.add_warning(format!("Unsupported expression type: {:?}", expr));
            panic!("Unsupported expression type: {:?}", expr);
        }
    }
}

/// Handle async expressions
fn handle_async_expr(
    async_expr: &syn::ExprAsync,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &async_expr.block, state)?;

    let async_fn = js::ArrowExpr {
        span: DUMMY_SP,
        params: vec![],
        body: Box::new(js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
            span: DUMMY_SP,
            stmts: body_stmts,
            ctxt: SyntaxContext::empty(),
        })),
        is_async: true,
        is_generator: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::Expr::Arrow(async_fn))
}

/// Handle await expressions
fn handle_await_expr(
    await_expr: &syn::ExprAwait,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let base = rust_expr_to_js_with_state(&await_expr.base, state)?;

    Ok(js::Expr::Await(js::AwaitExpr {
        span: DUMMY_SP,
        arg: Box::new(base),
    }))
}

// Parenthesized expressions
pub fn handle_paren_expr(
    paren: &syn::ExprParen,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let inner = rust_expr_to_js_with_state(&paren.expr, state)?;
    Ok(js::Expr::Paren(js::ParenExpr {
        span: DUMMY_SP,
        expr: Box::new(inner),
    }))
}

// Continuation of lib.rs - Helper functions

/// Handle method calls
fn handle_method_call(
    method_call: &syn::ExprMethodCall,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let receiver = rust_expr_to_js_with_state(&method_call.receiver, state)?;
    let method_name = method_call.method.to_string();

    // Convert arguments
    let args: Result<Vec<_>, _> = method_call
        .args
        .iter()
        .map(|arg| rust_expr_to_js_with_state(arg, state))
        .collect();
    let js_args = args?;

    // Handle special method mappings
    match method_name.as_str() {
        "len" | "count" => {
            // Use IIFE to evaluate receiver once and handle both arrays/strings and objects
            // ((obj) => obj.length !== undefined ? obj.length : Object.keys(obj).length)(receiver)
            
            // Create parameter for the IIFE
            let obj_param = js::Pat::Ident(js::BindingIdent {
                id: state.mk_ident("obj"),
                type_ann: None,
            });
            
            // Create obj.length access
            let length_access = state.mk_member_expr(js::Expr::Ident(state.mk_ident("obj")), "length");
            
            // Create undefined check: obj.length !== undefined
            let undefined_check = state.mk_binary_expr(
                length_access.clone(),
                js::BinaryOp::NotEqEq,
                state.mk_undefined()
            );
            
            // Create Object.keys(obj).length for objects
            let object_keys = state.mk_call_expr(
                state.mk_member_expr(js::Expr::Ident(state.mk_ident("Object")), "keys"),
                vec![js::Expr::Ident(state.mk_ident("obj"))]
            );
            let object_keys_length = state.mk_member_expr(object_keys, "length");
            
            // Create conditional expression
            let conditional = js::Expr::Cond(js::CondExpr {
                span: DUMMY_SP,
                test: Box::new(undefined_check),
                cons: Box::new(length_access),
                alt: Box::new(object_keys_length),
            });
            
            // Create IIFE: (obj) => conditional
            let iife = js::ArrowExpr {
                span: DUMMY_SP,
                params: vec![obj_param],
                body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(conditional))),
                is_async: false,
                is_generator: false,
                type_params: None,
                return_type: None,
                ctxt: SyntaxContext::empty(),
            };
            
            // Call the IIFE with receiver: ((obj) => ...)(receiver)
            Ok(js::Expr::Call(js::CallExpr {
                span: DUMMY_SP,
                callee: js::Callee::Expr(Box::new(js::Expr::Paren(js::ParenExpr {
                    span: DUMMY_SP,
                    expr: Box::new(js::Expr::Arrow(iife)),
                }))),
                args: vec![js::ExprOrSpread {
                    spread: None,
                    expr: Box::new(receiver),
                }],
                type_args: None,
                ctxt: SyntaxContext::empty(),
            }))
        }
        "clone" => {
            // .clone() is typically a no-op in JavaScript for primitives
            Ok(receiver)
        }
        "push" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "push"), js_args)),
        "pop" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "pop"), js_args)),
        "contains" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "includes"), js_args)),
        "to_string" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toString"), js_args)),
        "to_uppercase" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toUpperCase"), js_args))
        }
        "to_lowercase" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toLowerCase"), js_args))
        }
        "trim" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "trim"), js_args)),
        "trim_start" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "trimStart"), js_args))
        }
        "trim_end" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "trimEnd"), js_args)),
        "remove" => {
            // vec.remove(index) -> vec.splice(index, 1)[0]
            if js_args.len() == 1 {
                let splice_call = state.mk_call_expr(
                    state.mk_member_expr(receiver, "splice"),
                    vec![js_args[0].clone(), state.mk_num_lit(1.0)],
                );
                Ok(js::Expr::Member(js::MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(splice_call),
                    prop: js::MemberProp::Computed(js::ComputedPropName {
                        span: DUMMY_SP,
                        expr: Box::new(state.mk_num_lit(0.0)),
                    }),
                }))
            } else {
                Err("remove() expects exactly one argument".to_string())
            }
        }
        "insert" => {
            // Universal insert: use splice for arrays, property assignment for objects/HashMaps
            // ((obj, key, val) => obj.splice ? obj.splice(key, 0, val) : (obj[key] = val))(receiver, key, value)
            if js_args.len() == 2 {
                // Create parameters for the IIFE
                let obj_param = js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident("obj"),
                    type_ann: None,
                });
                let key_param = js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident("key"),
                    type_ann: None,
                });
                let val_param = js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident("val"),
                    type_ann: None,
                });
                
                // Check if obj.splice exists (array)
                let splice_check = state.mk_member_expr(js::Expr::Ident(state.mk_ident("obj")), "splice");
                
                // Array case: obj.splice(key, 0, val)
                let array_insert = state.mk_call_expr(
                    splice_check.clone(),
                    vec![
                        js::Expr::Ident(state.mk_ident("key")),
                        state.mk_num_lit(0.0),
                        js::Expr::Ident(state.mk_ident("val")),
                    ],
                );
                
                // Object case: obj[key] = val
                let obj_assignment = js::Expr::Assign(js::AssignExpr {
                    span: DUMMY_SP,
                    op: js::AssignOp::Assign,
                    left: js::AssignTarget::Simple(js::SimpleAssignTarget::Member(js::MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(js::Expr::Ident(state.mk_ident("obj"))),
                        prop: js::MemberProp::Computed(js::ComputedPropName {
                            span: DUMMY_SP,
                            expr: Box::new(js::Expr::Ident(state.mk_ident("key"))),
                        }),
                    })),
                    right: Box::new(js::Expr::Ident(state.mk_ident("val"))),
                });
                
                // Conditional: obj.splice ? array_insert : obj_assignment
                let conditional = js::Expr::Cond(js::CondExpr {
                    span: DUMMY_SP,
                    test: Box::new(splice_check),
                    cons: Box::new(array_insert),
                    alt: Box::new(obj_assignment),
                });
                
                // Create IIFE: (obj, key, val) => conditional
                let iife = js::ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![obj_param, key_param, val_param],
                    body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(conditional))),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                    ctxt: SyntaxContext::empty(),
                };
                
                // Call the IIFE with arguments: ((obj, key, val) => ...)(receiver, key, value)
                Ok(js::Expr::Call(js::CallExpr {
                    span: DUMMY_SP,
                    callee: js::Callee::Expr(Box::new(js::Expr::Paren(js::ParenExpr {
                        span: DUMMY_SP,
                        expr: Box::new(js::Expr::Arrow(iife)),
                    }))),
                    args: vec![
                        js::ExprOrSpread {
                            spread: None,
                            expr: Box::new(receiver),
                        },
                        js::ExprOrSpread {
                            spread: None,
                            expr: Box::new(js_args[0].clone()),
                        },
                        js::ExprOrSpread {
                            spread: None,
                            expr: Box::new(js_args[1].clone()),
                        },
                    ],
                    type_args: None,
                    ctxt: SyntaxContext::empty(),
                }))
            } else {
                Err("insert() expects exactly two arguments".to_string())
            }
        }

        "starts_with" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "startsWith"), js_args))
        }
        "ends_with" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "endsWith"), js_args)),
        "replace" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "replace"), js_args)),
        "split" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "split"), js_args)),
        "join" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "join"), js_args)),
        "map" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "map"), js_args)),
        "filter" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "filter"), js_args)),
        "find" => Ok(state.mk_call_expr(state.mk_member_expr(receiver, "find"), js_args)),
        "iter" => {
            // .iter() is typically a no-op in JavaScript
            Ok(receiver)
        }
        "collect" => {
            // .collect() is typically a no-op in JavaScript
            Ok(receiver)
        }
        "is_some" => {
            // Option::is_some() -> value !== null && value !== undefined
            let null_check =
                state.mk_binary_expr(receiver.clone(), js::BinaryOp::NotEqEq, state.mk_null_lit());
            let undefined_check =
                state.mk_binary_expr(receiver, js::BinaryOp::NotEqEq, state.mk_undefined());
            Ok(state.mk_binary_expr(null_check, js::BinaryOp::LogicalAnd, undefined_check))
        }
        "is_none" => {
            // Option::is_none() -> value === null || value === undefined
            let null_check =
                state.mk_binary_expr(receiver.clone(), js::BinaryOp::EqEqEq, state.mk_null_lit());
            let undefined_check =
                state.mk_binary_expr(receiver, js::BinaryOp::EqEqEq, state.mk_undefined());
            Ok(state.mk_binary_expr(null_check, js::BinaryOp::LogicalOr, undefined_check))
        }
        "unwrap" => {
            // .unwrap() is just the value itself in JavaScript
            Ok(receiver)
        }
        _ => {
            // Default method call
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, &method_name), js_args))
        }
    }
}

/// Handle function calls
fn handle_function_call(
    call: &syn::ExprCall,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Convert arguments first
    let args: Result<Vec<_>, _> = call
        .args
        .iter()
        .map(|arg| rust_expr_to_js_with_state(arg, state))
        .collect();
    let js_args = args?;

    match &*call.func {
        Expr::Path(path) => {
            // Check if this is a Type::method pattern
            if path.path.segments.len() >= 2 {
                let type_name = path.path.segments[path.path.segments.len() - 2]
                    .ident
                    .to_string();
                let method_name = path.path.segments.last().unwrap().ident.to_string();

                // Handle constructor calls (Type::new)
                if method_name == "new" {
                    // Handle special Rust types that should become JS equivalents
                    match type_name.as_str() {
                        "HashMap" | "BTreeMap" => {
                            // HashMap::new() or BTreeMap::new() becomes {}
                            return Ok(js::Expr::Object(js::ObjectLit {
                                span: DUMMY_SP,
                                props: vec![],
                            }));
                        }
                        "Vec" => {
                            // Vec::new() becomes []
                            return Ok(js::Expr::Array(js::ArrayLit {
                                span: DUMMY_SP,
                                elems: vec![],
                            }));
                        }
                        "Self" => {
                            // Self::new() in struct context should use object literal pattern
                            if let Some(current_struct) = state.get_current_struct_name() {
                                // Instead of new Constructor(), create object literal
                                return Ok(js::Expr::Object(js::ObjectLit {
                                    span: DUMMY_SP,
                                    props: vec![],
                                }));
                            }
                        }
                        _ => {
                            // Regular constructor call
                            return Ok(js::Expr::New(js::NewExpr {
                                span: DUMMY_SP,
                                callee: Box::new(js::Expr::Ident(state.mk_ident(&type_name))),
                                args: Some(
                                    js_args
                                        .into_iter()
                                        .map(|expr| js::ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(expr),
                                        })
                                        .collect(),
                                ),
                                type_args: None,
                                ctxt: SyntaxContext::empty(),
                            }));
                        }
                    }
                }

                // Handle other static methods (Type::method)
                let static_method =
                    state.mk_member_expr(js::Expr::Ident(state.mk_ident(&type_name)), &method_name);
                return Ok(state.mk_call_expr(static_method, js_args));
            }

            // Single segment path (regular function call)
            if let Some(last_segment) = path.path.segments.last() {
                let func_name = last_segment.ident.to_string();

                match func_name.as_str() {
                    "println" | "print" => {
                        let console_log =
                            state.mk_member_expr(js::Expr::Ident(state.mk_ident("console")), "log");
                        Ok(state.mk_call_expr(console_log, js_args))
                    }
                    "Some" => {
                        // Option::Some just returns the value in JavaScript
                        if js_args.len() == 1 {
                            Ok(js_args.into_iter().next().unwrap())
                        } else {
                            Err(format!(
                                "Some() expects exactly one argument, got {}",
                                js_args.len()
                            ))
                        }
                    }
                    "Ok" => {
                        // Ok(value) -> { ok: value }
                        if js_args.len() == 1 {
                            Ok(js::Expr::Object(js::ObjectLit {
                                span: DUMMY_SP,
                                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                                    js::KeyValueProp {
                                        key: js::PropName::Ident(state.mk_ident_name("ok")),
                                        value: Box::new(js_args.into_iter().next().unwrap()),
                                    },
                                )))],
                            }))
                        } else {
                            Err(format!(
                                "Ok() expects exactly one argument, got {}",
                                js_args.len()
                            ))
                        }
                    }
                    "Err" => {
                        // Err(error) -> { error: error }
                        if js_args.len() == 1 {
                            Ok(js::Expr::Object(js::ObjectLit {
                                span: DUMMY_SP,
                                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                                    js::KeyValueProp {
                                        key: js::PropName::Ident(state.mk_ident_name("error")),
                                        value: Box::new(js_args.into_iter().next().unwrap()),
                                    },
                                )))],
                            }))
                        } else {
                            Err(format!(
                                "Err() expects exactly one argument, got {}",
                                js_args.len()
                            ))
                        }
                    }
                    _ => {
                        // Regular function call
                        let callee = js::Expr::Ident(state.mk_ident(&func_name));
                        Ok(state.mk_call_expr(callee, js_args))
                    }
                }
            } else {
                Err("Invalid function path".to_string())
            }
        }
        _ => {
            // Complex function expression
            let callee = rust_expr_to_js_with_state(&call.func, state)?;
            Ok(state.mk_call_expr(callee, js_args))
        }
    }
}

fn contains_format_arguments(s: &str) -> bool {
    s.contains("{}") || s.contains("{:?}")
}

/// Handle macro expressions
fn handle_macro_expr(mac: &syn::Macro, state: &mut TranspilerState) -> Result<js::Expr, String> {
    let macro_name = if let Some(segment) = mac.path.segments.last() {
        segment.ident.to_string()
    } else {
        return Err("Invalid macro".to_string());
    };

    let tokens = mac.tokens.to_string();
    debug_print!("MACRO-DEBUG: {}", macro_name.as_str());

    match macro_name.as_str() {
        "println" | "print" => {
            let console_method = if macro_name == "println" {
                "log"
            } else {
                "log"
            };
            let console_expr =
                state.mk_member_expr(js::Expr::Ident(state.mk_ident("console")), console_method);

            if tokens.trim().is_empty() {
                Ok(state.mk_call_expr(console_expr, vec![]))
            } else if contains_format_arguments(&tokens) {
                // Format-style macro
                let format_result = handle_format_like_macro(&tokens, state)?;
                Ok(state.mk_call_expr(console_expr, vec![format_result]))
            } else {
                // Simple string or expression
                let arg = parse_macro_tokens(&tokens, state)?;
                Ok(state.mk_call_expr(console_expr, vec![arg]))
            }
        }
        "eprintln" | "eprint" => {
            let console_expr =
                state.mk_member_expr(js::Expr::Ident(state.mk_ident("console")), "error");

            if tokens.trim().is_empty() {
                Ok(state.mk_call_expr(console_expr, vec![]))
            } else if tokens.contains("{}") {
                let format_result = handle_format_like_macro(&tokens, state)?;
                Ok(state.mk_call_expr(console_expr, vec![format_result]))
            } else {
                let arg = parse_macro_tokens(&tokens, state)?;
                Ok(state.mk_call_expr(console_expr, vec![arg]))
            }
        }
        "format" => handle_format_like_macro(&tokens, state),
        "vec" => {
            if tokens.trim().is_empty() {
                // vec!() -> []
                Ok(js::Expr::Array(js::ArrayLit {
                    span: DUMMY_SP,
                    elems: vec![],
                }))
            } else if tokens.contains(';') {
                // vec![value; count] -> Array.from({length: count}, () => value)
                let parts: Vec<&str> = tokens.split(';').collect();
                if parts.len() == 2 {
                    let value_expr = parse_macro_tokens(parts[0].trim(), state)?;
                    let count_expr = parse_macro_tokens(parts[1].trim(), state)?;

                    // Create Array.from call
                    let array_from =
                        state.mk_member_expr(js::Expr::Ident(state.mk_ident("Array")), "from");

                    // Create {length: count} object
                    let length_obj = js::Expr::Object(js::ObjectLit {
                        span: DUMMY_SP,
                        props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name("length")),
                                value: Box::new(count_expr),
                            },
                        )))],
                    });

                    // Create () => value arrow function
                    let arrow_fn = js::ArrowExpr {
                        span: DUMMY_SP,
                        params: vec![],
                        body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(value_expr))),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                        ctxt: SyntaxContext::empty(),
                    };

                    Ok(state.mk_call_expr(array_from, vec![length_obj, js::Expr::Arrow(arrow_fn)]))
                } else {
                    Err("Invalid vec! syntax with semicolon".to_string())
                }
            } else {
                // vec![a, b, c] -> [a, b, c]
                let elements = parse_comma_separated_exprs(&tokens, state)?;
                let js_elements: Vec<Option<js::ExprOrSpread>> = elements
                    .into_iter()
                    .map(|expr| {
                        Some(js::ExprOrSpread {
                            spread: None,
                            expr: Box::new(expr),
                        })
                    })
                    .collect();

                Ok(js::Expr::Array(js::ArrayLit {
                    span: DUMMY_SP,
                    elems: js_elements,
                }))
            }
        }
        _ => panic!("Unsupported macro: {}", macro_name),
    }
}

/// Handle format-like macros (format!, println! with {}, etc.)
fn handle_format_like_macro(
    token_string: &str,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let parts = smart_comma_split(token_string);

    if parts.is_empty() {
        /* the call 'format!()' should not ever happen, but... */
        return Ok(state.mk_template_literal(vec![], vec![]));
    }

    // Get the format string
    let mut format_str = parts[0].trim();
    if format_str.starts_with('"') && format_str.ends_with('"') {
        format_str = &format_str[1..format_str.len() - 1];
    }

    // Get format arguments
    let format_args: Result<Vec<_>, _> = parts
        .iter()
        .skip(1)
        .map(|arg| parse_macro_tokens(arg.trim(), state))
        .collect();
    let js_args = format_args?;

    // Check if there are placeholders
    if !format_str.contains("{}") && !format_str.contains("{:?}") {
        return Ok(state.mk_template_literal(vec![format_str.into()], vec![]));
        // this will return just the quoted format string.
        // return Ok(state.mk_str_lit(format_str));
    }

    // Enhanced parsing to handle both {} and {:?} placeholders
    let (template_parts, template_exprs) =
        parse_format_string_with_debug(format_str, js_args, state)?;

    Ok(state.mk_template_literal(template_parts, template_exprs))
}

/// Parse format string handling both {} and {:?} placeholders
fn parse_format_string_with_debug(
    format_str: &str,
    js_args: Vec<js::Expr>,
    state: &mut TranspilerState,
) -> Result<(Vec<String>, Vec<js::Expr>), String> {
    let mut template_parts = Vec::new();
    let mut template_exprs = Vec::new();
    let mut arg_index = 0;

    let mut chars = format_str.chars().peekable();
    let mut current_part = String::new();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '{' {
                    // Escaped brace {{
                    current_part.push('{');
                    chars.next(); // consume the second {
                    continue;
                }
            }

            // Start of placeholder
            template_parts.push(current_part);
            current_part = String::new();

            // Parse the placeholder content
            let mut placeholder_content = String::new();
            let mut found_closing = false;

            while let Some(inner_ch) = chars.next() {
                if inner_ch == '}' {
                    found_closing = true;
                    break;
                } else {
                    placeholder_content.push(inner_ch);
                }
            }

            if !found_closing {
                return Err("Unclosed placeholder in format string".to_string());
            }

            // Process the placeholder
            if arg_index < js_args.len() {
                let expr = if placeholder_content == ":?" {
                    // Debug format - wrap with debug_repr
                    state.mk_call_expr(
                        js::Expr::Ident(state.mk_ident("debug_repr")),
                        vec![js_args[arg_index].clone()],
                    )
                } else if placeholder_content.is_empty() {
                    // Regular format
                    js_args[arg_index].clone()
                } else {
                    // Other format specifiers - for now treat as regular
                    // Could be extended to handle other format types like {:x}, {:02}, etc.
                    js_args[arg_index].clone()
                };

                template_exprs.push(expr);
                arg_index += 1;
            } else {
                // Not enough arguments - add empty string
                template_exprs.push(state.mk_str_lit(""));
            }
        } else if ch == '}' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    // Escaped brace }}
                    current_part.push('}');
                    chars.next(); // consume the second }
                    continue;
                }
            }
            // Unmatched closing brace - just add it
            current_part.push(ch);
        } else {
            current_part.push(ch);
        }
    }

    // Add the final part
    template_parts.push(current_part);

    // Ensure we have the right number of parts vs expressions
    while template_parts.len() < template_exprs.len() + 1 {
        template_parts.push("".to_string());
    }
    while template_parts.len() > template_exprs.len() + 1 {
        template_parts.pop();
    }

    Ok((template_parts, template_exprs))
}

/// Handle format! macro with parsed arguments and debug support
fn handle_format_macro_with_state(
    args: &Punctuated<Expr, Comma>,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    debug_print!("DEBUG-handle_format_macro_with_state");
    if args.is_empty() {
        return Ok(js::Expr::Tpl(js::Tpl {
            span: DUMMY_SP,
            exprs: vec![],
            quasis: vec![js::TplElement {
                span: DUMMY_SP,
                tail: true,
                cooked: Some("".into()),
                raw: swc_atoms::Atom::new("".to_string()),
            }],
        }));
    }

    // Get the format string
    if let Some(first_arg) = args.first() {
        if let Expr::Lit(lit) = first_arg {
            if let syn::Lit::Str(str_lit) = &lit.lit {
                let format_str = str_lit.value();

                /* This will return just the quotes:

                                if !format_str.contains("{}") {
                                    return Ok(state.mk_str_lit(&format_str));
                                }
                */

                /* this will return the empty template (backticks): */

                if !format_str.contains("{}") {
                    return Ok(js::Expr::Tpl(js::Tpl {
                        span: DUMMY_SP,
                        exprs: vec![],
                        quasis: vec![js::TplElement {
                            span: DUMMY_SP,
                            tail: true,
                            cooked: Some(format_str.clone().into()),
                            raw: swc_atoms::Atom::new(format_str.to_string()),
                        }],
                    }));
                }

                // Get format arguments
                let format_args: Result<Vec<_>, _> = args
                    .iter()
                    .skip(1)
                    .map(|arg| rust_expr_to_js_with_state(arg, state))
                    .collect();
                let js_args = format_args?;

                // Parse with debug support
                let (template_parts, template_exprs) =
                    parse_format_string_with_debug(&format_str, js_args, state)?;

                return Ok(state.mk_template_literal(template_parts, template_exprs));
            }
        }
    }

    // Fallback: concatenate arguments
    let js_args: Result<Vec<_>, _> = args
        .iter()
        .map(|arg| rust_expr_to_js_with_state(arg, state))
        .collect();

    let args_vec = js_args?;
    if args_vec.len() == 1 {
        Ok(args_vec.into_iter().next().unwrap())
    } else {
        // Join with spaces - this is a simplification
        let joined = args_vec
            .into_iter()
            .reduce(|acc, expr| {
                state.mk_binary_expr(acc.clone(), js::BinaryOp::Add, state.mk_str_lit(" "));
                state.mk_binary_expr(acc, js::BinaryOp::Add, expr)
            })
            .unwrap_or_else(|| state.mk_str_lit(""));

        Ok(joined)
    }
}

/// Handle local variable declarations
fn handle_local_statement(
    block_action: BlockAction,
    local: &syn::Local,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    if let Some(init) = &local.init {
        let init_expr = rust_expr_to_js_with_action_and_state(block_action, &init.expr, state)?;

        match &local.pat {
            Pat::Ident(pat_ident) => {
                let var_name = pat_ident.ident.to_string();
                let js_var_name = escape_js_identifier(&var_name);
                let is_mutable = pat_ident.mutability.is_some();

                let unique_js_var_name = state.declare_variable(var_name, js_var_name, is_mutable);

                Ok(state.mk_var_decl(&unique_js_var_name, Some(init_expr), !is_mutable))
            }
            Pat::Type(type_pat) => {
                // Handle typed patterns like `let x: i32 = 23;`
                // We ignore the type annotation and just handle the inner pattern
                match &*type_pat.pat {
                    Pat::Ident(pat_ident) => {
                        let var_name = pat_ident.ident.to_string();
                        let js_var_name = escape_js_identifier(&var_name);
                        let is_mutable = pat_ident.mutability.is_some();

                        let unique_js_var_name = state.declare_variable(var_name, js_var_name, is_mutable);

                        Ok(state.mk_var_decl(&unique_js_var_name, Some(init_expr), !is_mutable))
                    }
                    _ => {
                        // This is a simplified approach - you might want more sophisticated handling
                        panic!(
                            "Complex typed patterns not yet supported: {:?}",
                            type_pat.pat
                        )
                    }
                }
            }

            Pat::Tuple(tuple_pat) => {
                // Handle destructuring assignment
                let var_names: Vec<String> = tuple_pat
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

                // Create array destructuring pattern
                let destructure_pattern = js::Pat::Array(js::ArrayPat {
                    span: DUMMY_SP,
                    elems: var_names
                        .iter()
                        .map(|name| {
                            let js_name = escape_js_identifier(name);
                            let unique_js_name = state.declare_variable(name.clone(), js_name, false);
                            Some(js::Pat::Ident(js::BindingIdent {
                                id: state.mk_ident(&unique_js_name),
                                type_ann: None,
                            }))
                        })
                        .collect(),
                    optional: false,
                    type_ann: None,
                });

                Ok(js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
                    span: DUMMY_SP,
                    kind: js::VarDeclKind::Const,
                    declare: false,
                    decls: vec![js::VarDeclarator {
                        span: DUMMY_SP,
                        name: destructure_pattern,
                        init: Some(Box::new(init_expr)),
                        definite: false,
                    }],
                    ctxt: SyntaxContext::empty(),
                }))))
            }
            Pat::Struct(struct_pat) => {
                // Handle struct destructuring: let Person { name, age } = person;
                let field_names: Vec<String> = struct_pat
                    .fields
                    .iter()
                    .filter_map(|field_pat| {
                        if let syn::Member::Named(ident) = &field_pat.member {
                            Some(ident.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                // Create object destructuring pattern using shorthand syntax
                let destructure_pattern = js::Pat::Object(js::ObjectPat {
                    span: DUMMY_SP,
                    props: field_names
                        .iter()
                        .map(|name| {
                            let js_name = escape_js_identifier(name);
                            let unique_js_name = state.declare_variable(name.clone(), js_name, false);
                            // Use shorthand property syntax for destructuring
                            js::ObjectPatProp::Assign(js::AssignPatProp {
                                span: DUMMY_SP,
                                key: js::BindingIdent {
                                    id: state.mk_ident(&unique_js_name),
                                    type_ann: None,
                                },
                                value: None, // None means shorthand syntax
                            })
                        })
                        .collect(),
                    optional: false,
                    type_ann: None,
                });

                Ok(js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
                    span: DUMMY_SP,
                    kind: js::VarDeclKind::Const,
                    declare: false,
                    decls: vec![js::VarDeclarator {
                        span: DUMMY_SP,
                        name: destructure_pattern,
                        init: Some(Box::new(init_expr)),
                        definite: false,
                    }],
                    ctxt: SyntaxContext::empty(),
                }))))
            }
            _ => panic!("Unsupported destructuring pattern {:?}", &local.pat),
        }
    } else {
        // Variable declaration without initialization
        if let Pat::Ident(pat_ident) = &local.pat {
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            let is_mutable = pat_ident.mutability.is_some();

            let unique_js_var_name = state.declare_variable(var_name, js_var_name, is_mutable);

            Ok(state.mk_var_decl(&unique_js_var_name, None, false)) // Always use let for uninitialized
        } else {
            panic!("Unsupported variable pattern {:?}", &local.pat)
        }
    }
}

/// Parse macro tokens into a JavaScript expression
fn parse_macro_tokens(tokens: &str, state: &mut TranspilerState) -> Result<js::Expr, String> {
    let trimmed = tokens.trim();

    // Try to parse as a Rust expression first
    if let Ok(parsed_expr) = syn::parse_str::<syn::Expr>(trimmed) {
        rust_expr_to_js_with_state(&parsed_expr, state)
    } else {
        // Fallback to string literal
        Ok(state.mk_str_lit(trimmed))
    }
}

/// Parse comma-separated expressions from macro tokens
fn parse_comma_separated_exprs(
    tokens: &str,
    state: &mut TranspilerState,
) -> Result<Vec<js::Expr>, String> {
    let parts = smart_comma_split(tokens);
    parts
        .iter()
        .map(|part| parse_macro_tokens(part.trim(), state))
        .collect()
}

/// Smart comma splitting that respects quote boundaries
fn smart_comma_split(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_quotes = false;
    let mut paren_depth = 0;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current_part.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current_part.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current_part.push(ch);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                parts.push(current_part.trim().to_string());
                current_part.clear();
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    if !current_part.is_empty() {
        parts.push(current_part.trim().to_string());
    }

    parts
}

/// Check if an expression is likely to be a string
fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(lit) => matches!(lit.lit, syn::Lit::Str(_)),
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(segment) = path.path.segments.last() {
                    matches!(segment.ident.to_string().as_str(), "format" | "to_string")
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

/// Escape JavaScript reserved words and invalid identifiers
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

/// Generate JavaScript class for a Rust struct
pub fn generate_js_class_for_struct_with_state(
    input_struct: &ItemStruct,
) -> Result<js::ModuleItem, String> {
    let mut state = TranspilerState::new();
    let struct_name = input_struct.ident.to_string();

    let fields: Vec<(String, String)> = match &input_struct.fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                if let Some(ident) = &field.ident {
                    let field_name = ident.to_string();
                    // Handle raw identifiers by removing r# prefix
                    let clean_field_name = if field_name.starts_with("r#") {
                        field_name[2..].to_string()
                    } else {
                        field_name
                    };
                    let field_type = format_rust_type(&field.ty);
                    Some((clean_field_name, field_type))
                } else {
                    None
                }
            })
            .collect(),
        Fields::Unnamed(_) => {
            vec![("data".to_string(), "Array".to_string())]
        }
        Fields::Unit => vec![],
    };

    // Create constructor parameters
    let constructor_params: Vec<js::ParamOrTsParamProp> = fields
        .iter()
        .map(|(name, _)| {
            js::ParamOrTsParamProp::Param(js::Param {
                span: DUMMY_SP,
                decorators: vec![],
                pat: js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident(name),
                    type_ann: None,
                }),
            })
        })
        .collect();

    // Create constructor body
    let mut constructor_body = Vec::new();
    for (name, _) in &fields {
        let assignment = js::Expr::Assign(js::AssignExpr {
            span: DUMMY_SP,
            op: js::AssignOp::Assign,
            left: state.expr_to_assign_target(state.mk_member_expr(state.mk_this_expr(), name))?,
            right: Box::new(js::Expr::Ident(state.mk_ident(name))),
        });
        constructor_body.push(state.mk_expr_stmt(assignment));
    }

    let constructor = js::Constructor {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident_name("constructor")),
        params: constructor_params,
        body: Some(js::BlockStmt {
            span: DUMMY_SP,
            stmts: constructor_body,
            ctxt: SyntaxContext::empty(),
        }),
        accessibility: None,
        is_optional: false,
        ctxt: SyntaxContext::empty(),
    };

    // Create toJSON method
    let to_json_method = create_to_json_method(&fields, &mut state)?;

    // Create fromJSON static method
    let from_json_method = create_from_json_static_method(&struct_name, &fields, &mut state)?;

    // Create class with all methods
    let class = js::Class {
        span: DUMMY_SP,
        decorators: vec![],
        body: vec![
            js::ClassMember::Constructor(constructor),
            js::ClassMember::Method(to_json_method),
            js::ClassMember::Method(from_json_method),
        ],
        super_class: None,
        is_abstract: false,
        type_params: None,
        super_type_params: None,
        implements: vec![],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Class(
        js::ClassDecl {
            ident: state.mk_ident(&struct_name),
            declare: false,
            class: Box::new(class),
        },
    ))))
}

// Helper function to create toJSON method
fn create_to_json_method(
    fields: &[(String, String)],
    state: &mut TranspilerState,
) -> Result<js::ClassMethod, String> {
    // Create object properties for each field
    let mut props = Vec::new();

    for (name, _) in fields {
        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
            js::KeyValueProp {
                key: js::PropName::Ident(state.mk_ident_name(name)),
                value: Box::new(state.mk_member_expr(state.mk_this_expr(), name)),
            },
        ))));
    }

    let return_obj = js::Expr::Object(js::ObjectLit {
        span: DUMMY_SP,
        props,
    });

    let method_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: vec![state.mk_return_stmt(Some(return_obj))],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ClassMethod {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident_name("toJSON")),
        function: Box::new(js::Function {
            params: vec![],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(method_body),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
            ctxt: SyntaxContext::empty(),
        }),
        kind: js::MethodKind::Method,
        is_static: false,
        accessibility: None,
        is_abstract: false,
        is_optional: false,
        is_override: false,
    })
}

// Helper function to create fromJSON static method
fn create_from_json_static_method(
    struct_name: &str,
    fields: &[(String, String)],
    state: &mut TranspilerState,
) -> Result<js::ClassMethod, String> {
    // Create constructor arguments from json properties
    let constructor_args: Vec<js::Expr> = fields
        .iter()
        .map(|(name, _)| state.mk_member_expr(js::Expr::Ident(state.mk_ident("json")), name))
        .collect();

    // Create new StructName(json.field1, json.field2, ...)
    let new_instance = js::Expr::New(js::NewExpr {
        span: DUMMY_SP,
        callee: Box::new(js::Expr::Ident(state.mk_ident(struct_name))),
        args: Some(
            constructor_args
                .into_iter()
                .map(|expr| js::ExprOrSpread {
                    spread: None,
                    expr: Box::new(expr),
                })
                .collect(),
        ),
        type_args: None,
        ctxt: SyntaxContext::empty(),
    });

    let method_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: vec![state.mk_return_stmt(Some(new_instance))],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ClassMethod {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident_name("fromJSON")),
        function: Box::new(js::Function {
            params: vec![state.pat_to_param(js::Pat::Ident(js::BindingIdent {
                id: state.mk_ident("json"),
                type_ann: None,
            }))],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(method_body),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
            ctxt: SyntaxContext::empty(),
        }),
        kind: js::MethodKind::Method,
        is_static: true, // This is the key difference - static method
        accessibility: None,
        is_abstract: false,
        is_optional: false,
        is_override: false,
    })
}

/// Generate JavaScript enum
pub fn generate_js_enum_with_state(input_enum: &ItemEnum) -> Result<Vec<js::ModuleItem>, String> {
    let mut state = TranspilerState::new();
    let enum_name = input_enum.ident.to_string();

    let mut properties = Vec::new();

    // Add enum variants
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();

        match &variant.fields {
            Fields::Unit => {
                // Simple enum variants become string values
                properties.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name(&variant_name)),
                        value: Box::new(state.mk_str_lit_single_quote(&variant_name)),
                    },
                ))));
            }
            Fields::Unnamed(_) | Fields::Named(_) => {
                // Create factory function for complex variants
                let param_count = match &variant.fields {
                    Fields::Unnamed(f) => f.unnamed.len(),
                    Fields::Named(f) => f.named.len(),
                    _ => 0,
                };

                let field_names: Vec<String> = match &variant.fields {
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

                let params: Vec<js::Pat> = (0..param_count)
                    .map(|i| {
                        js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&field_names[i]),
                            type_ann: None,
                        })
                    })
                    .collect();

                // Create function body that returns an object
                let mut obj_props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit_single_quote(&variant_name)),
                    },
                )))];

                for i in 0..param_count {
                    obj_props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident_name(&field_names[i])),
                            value: Box::new(js::Expr::Ident(state.mk_ident(&field_names[i]))),
                        },
                    ))));
                }

                let return_obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props: obj_props,
                });

                let function_body = js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![state.mk_return_stmt(Some(return_obj))],
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

                properties.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name(&variant_name)),
                        value: Box::new(js::Expr::Fn(js::FnExpr {
                            ident: None,
                            function: Box::new(function),
                        })),
                    },
                ))));
            }
        }
    }

    // Add JSON methods directly to the enum object (before creating the enum object)
    let from_json_function = create_enum_from_json_function(input_enum, &mut state)?;
    
    // Add fromJSON as a static method to the enum object
    properties.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
        js::KeyValueProp {
            key: js::PropName::Ident(state.mk_ident_name("fromJSON")),
            value: Box::new(js::Expr::Fn(js::FnExpr {
                ident: None,
                function: Box::new(from_json_function),
            })),
        },
    ))));

    // Add toJSON method to the enum properties
    let to_json_method_body = create_enum_to_json_switch_body(input_enum, &mut state)?;
    let to_json_function = js::Function {
        params: vec![state.pat_to_param(js::Pat::Ident(js::BindingIdent {
            id: state.mk_ident("enumValue"),
            type_ann: None,
        }))],
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(to_json_method_body),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    };
    
    properties.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
        js::KeyValueProp {
            key: js::PropName::Ident(state.mk_ident_name("toJSON")),
            value: Box::new(js::Expr::Fn(js::FnExpr {
                ident: None,
                function: Box::new(to_json_function),
            })),
        },
    ))));

    // Create the enum object
    let enum_obj = js::Expr::Object(js::ObjectLit {
        span: DUMMY_SP,
        props: properties,
    });

    // Create const declaration for the enum
    let enum_var_decl = js::VarDecl {
        span: DUMMY_SP,
        kind: js::VarDeclKind::Const,
        declare: false,
        decls: vec![js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: state.mk_ident(&enum_name),
                type_ann: None,
            }),
            init: Some(Box::new(enum_obj)),
            definite: false,
        }],
        ctxt: SyntaxContext::empty(),
    };

    let mut items = vec![js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Var(
        Box::new(enum_var_decl),
    )))];

    // Create standalone isEnumName function
    let is_function_name = format!("is{}", enum_name);

    // Create function body with the same logic as the old code
    let function_body =
        js::BlockStmt {
            span: DUMMY_SP,
            stmts: vec![
                // if (typeof value === 'string') {
                js::Stmt::If(js::IfStmt {
                    span: DUMMY_SP,
                    test: Box::new(state.mk_binary_expr(
                        js::Expr::Unary(js::UnaryExpr {
                            span: DUMMY_SP,
                            op: js::UnaryOp::TypeOf,
                            arg: Box::new(js::Expr::Ident(state.mk_ident("value"))),
                        }),
                        js::BinaryOp::EqEqEq,
                        state.mk_str_lit("string"),
                    )),
                    cons: Box::new(js::Stmt::Block(js::BlockStmt {
                        span: DUMMY_SP,
                        stmts: vec![
                            // return Object.values(EnumName).includes(value);
                            state.mk_return_stmt(Some(state.mk_call_expr(
                                state.mk_member_expr(
                                    state.mk_call_expr(
                                        state.mk_member_expr(
                                            js::Expr::Ident(state.mk_ident("Object")),
                                            "values",
                                        ),
                                        vec![js::Expr::Ident(state.mk_ident(&enum_name))],
                                    ),
                                    "includes",
                                ),
                                vec![js::Expr::Ident(state.mk_ident("value"))],
                            ))),
                        ],
                        ctxt: SyntaxContext::empty(),
                    })),
                    alt: None,
                }),
                // if (value && typeof value === 'object' && value.type) {
                js::Stmt::If(js::IfStmt {
                    span: DUMMY_SP,
                    test: Box::new(state.mk_binary_expr(
                        state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident("value")),
                            js::BinaryOp::LogicalAnd,
                            state.mk_binary_expr(
                                js::Expr::Unary(js::UnaryExpr {
                                    span: DUMMY_SP,
                                    op: js::UnaryOp::TypeOf,
                                    arg: Box::new(js::Expr::Ident(state.mk_ident("value"))),
                                }),
                                js::BinaryOp::EqEqEq,
                                state.mk_str_lit("object"),
                            ),
                        ),
                        js::BinaryOp::LogicalAnd,
                        state.mk_member_expr(js::Expr::Ident(state.mk_ident("value")), "type"),
                    )),
                    cons: Box::new(js::Stmt::Block(js::BlockStmt {
                        span: DUMMY_SP,
                        stmts: vec![
                            // return Object.keys(EnumName).includes(value.type);
                            state.mk_return_stmt(Some(state.mk_call_expr(
                                state.mk_member_expr(
                                    state.mk_call_expr(
                                        state.mk_member_expr(
                                            js::Expr::Ident(state.mk_ident("Object")),
                                            "keys",
                                        ),
                                        vec![js::Expr::Ident(state.mk_ident(&enum_name))],
                                    ),
                                    "includes",
                                ),
                                vec![state.mk_member_expr(
                                    js::Expr::Ident(state.mk_ident("value")),
                                    "type",
                                )],
                            ))),
                        ],
                        ctxt: SyntaxContext::empty(),
                    })),
                    alt: None,
                }),
                // return false;
                state.mk_return_stmt(Some(state.mk_bool_lit(false))),
            ],
            ctxt: SyntaxContext::empty(),
        };

    let is_function = js::Function {
        params: vec![state.pat_to_param(js::Pat::Ident(js::BindingIdent {
            id: state.mk_ident("value"),
            type_ann: None,
        }))],
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
    let is_function_decl = js::FnDecl {
        ident: state.mk_ident(&is_function_name),
        declare: false,
        function: Box::new(is_function),
    };

    items.push(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Fn(
        is_function_decl,
    ))));

    Ok(items)
}

/// Format Rust types to JavaScript-friendly representations
pub fn format_rust_type(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                match type_name.as_str() {
                    "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64"
                    | "usize" | "f32" | "f64" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    "String" | "str" => "string".to_string(),
                    "Vec" => "Array".to_string(),
                    "HashMap" | "BTreeMap" => "Map".to_string(),
                    "HashSet" | "BTreeSet" => "Set".to_string(),
                    "Option" => "".to_string(), // Handled specially
                    "Result" => "".to_string(), // Handled specially
                    _ => "object".to_string(),
                }
            } else {
                "object".to_string()
            }
        }
        Type::Reference(type_ref) => format_rust_type(&type_ref.elem),
        Type::Array(_) => "Array".to_string(),
        Type::Tuple(_) => "Array".to_string(),
        _ => "object".to_string(),
    }
}

pub fn ast_to_code(module_items: &[js::ModuleItem]) -> Result<String, String> {
    ast_to_code_compact(module_items)
}

/// Convert JavaScript AST to code string
pub fn ast_to_code_verbose(module_items: &[js::ModuleItem]) -> Result<String, String> {
    let module = js::Module {
        span: DUMMY_SP,
        body: module_items.to_vec(),
        shebang: None,
    };

    Ok(swc_ecma_codegen::to_code(&module))
}

/// Convert JavaScript AST to compact code string (single line arrays)
pub fn ast_to_code_compact(module_items: &[js::ModuleItem]) -> Result<String, String> {
    use swc_common::SourceMap;
    use swc_common::sync::Lrc;
    use swc_ecma_codegen::{Config, Emitter};

    let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));
    let mut buf = vec![];

    let mut config = Config::default();
    config.minify = false;
    config.ascii_only = false;
    config.omit_last_semi = false;
    config.target = swc_ecma_ast::EsVersion::Es2020;
    config.emit_assert_for_import_attributes = true;
    config.inline_script = false;
    config.reduce_escaped_newline = false;

    let module = js::Module {
        span: DUMMY_SP,
        body: module_items.to_vec(),
        shebang: None,
    };

    let mut emitter = Emitter {
        cfg: config,
        cm: cm.clone(),
        comments: None,
        wr: swc_ecma_codegen::text_writer::JsWriter::new(cm, "\n", &mut buf, None),
    };

    emitter
        .emit_module(&module)
        .map_err(|e| format!("Codegen error: {}", e))?;

    let code = String::from_utf8(buf).map_err(|e| format!("UTF8 error: {}", e))?;

    // Post-process to ensure arrays are compact
    let compact_code = code
        .replace("[\n    ", "[")
        .replace(",\n    ", ", ")
        .replace("\n]", "]")
        .replace(";\n", ";\n"); // Keep statement separators

    Ok(compact_code)
}

/// Convert JavaScript AST to code string, trimmed of trailing semicolons and whitespace
pub fn ast_to_code_trimmed(module_items: &[js::ModuleItem]) -> Result<String, String> {
    let code = ast_to_code(module_items)?;

    // Properly trim trailing semicolons and whitespace
    Ok(code
        .trim_end_matches('\n')
        .trim_end_matches(';') // Remove trailing semicolon
        .trim_end() // Remove any remaining whitespace
        .to_string())
}

/// Convenience function to transpile a complete impl block to JavaScript code
pub fn transpile_impl_to_js(input_impl: &ItemImpl) -> Result<String, String> {
    let module_items = generate_js_methods_for_impl_with_state(input_impl)?;
    ast_to_code(&module_items)
}

/// Convenience function to transpile a struct to JavaScript code
pub fn transpile_struct_to_js(input_struct: &ItemStruct) -> Result<String, String> {
    let module_item = generate_js_class_for_struct_with_state(input_struct)?;
    ast_to_code(&[module_item])
}

/// Convenience function to transpile an enum to JavaScript code
pub fn transpile_enum_to_js(input_enum: &ItemEnum) -> Result<String, String> {
    let module_items = generate_js_enum_with_state(input_enum)?;
    ast_to_code(&module_items)
}

/// Core function that converts Rust while loop to JavaScript while statement
fn convert_while_to_stmt(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    let test = rust_expr_to_js_with_state(&while_expr.cond, state)?;
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &while_expr.body, state)?;

    Ok(js::Stmt::While(js::WhileStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        body: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: body_stmts,
            ctxt: SyntaxContext::empty(),
        })),
    }))
}

/// Core function that converts Rust while-let loop to JavaScript while statement
fn convert_while_let_to_stmt(
    while_expr: &syn::ExprWhile,
    let_expr: &syn::ExprLet, // The let condition
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Handle while let Some(x) = expr { ... } patterns
    let (condition, mut binding_stmts) =
        handle_pattern_binding(&let_expr.pat, "_while_temp", state)?;
    let matched_expr = rust_expr_to_js_with_state(&let_expr.expr, state)?;
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &while_expr.body, state)?;

    // Create: while (condition) { let x = temp; ...body }
    let mut while_body_stmts = vec![
        // const _while_temp = matched_expr;
        state.mk_var_decl("_while_temp", Some(matched_expr.clone()), true),
    ];

    // Add the pattern binding statements
    while_body_stmts.extend(binding_stmts.clone());

    // Add the original body
    while_body_stmts.extend(body_stmts.clone());

    // For while let, we need to re-evaluate the condition in each iteration
    // Convert to: while (true) { const temp = expr; if (!condition) break; bindings; body; }
    let mut loop_body = vec![
        state.mk_var_decl("_while_temp", Some(matched_expr), true),
        js::Stmt::If(js::IfStmt {
            span: DUMMY_SP,
            test: Box::new(js::Expr::Unary(js::UnaryExpr {
                span: DUMMY_SP,
                op: js::UnaryOp::Bang,
                arg: Box::new(condition),
            })),
            cons: Box::new(js::Stmt::Break(js::BreakStmt {
                span: DUMMY_SP,
                label: None,
            })),
            alt: None,
        }),
    ];

    // Add bindings and body
    loop_body.extend(binding_stmts);
    loop_body.extend(body_stmts);

    Ok(js::Stmt::While(js::WhileStmt {
        span: DUMMY_SP,
        test: Box::new(state.mk_bool_lit(true)),
        body: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: loop_body,
            ctxt: SyntaxContext::empty(),
        })),
    }))
}

/// Enhanced while converter that detects while-let patterns
fn convert_while_to_stmt_enhanced(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Check if this is a while-let pattern
    if let Expr::Let(let_expr) = &*while_expr.cond {
        return convert_while_let_to_stmt(while_expr, let_expr, state);
    }

    // Regular while loop
    convert_while_to_stmt(while_expr, state)
}

/// Modified handle_while_expr that reuses the core logic
fn handle_while_expr(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Reuse the statement converter and wrap in IIFE
    let while_stmt = convert_while_to_stmt_enhanced(while_expr, state)?;
    Ok(state.mk_iife(vec![while_stmt]))
}

/// Handle while-let specifically for Option patterns (like the legacy code)
fn convert_while_let_option_to_stmt(
    while_expr: &syn::ExprWhile,
    let_expr: &syn::ExprLet,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Check if this is specifically "while let Some(x) = ..." pattern
    if let Pat::TupleStruct(tuple_struct) = &*let_expr.pat {
        if let Some(segment) = tuple_struct.path.segments.last() {
            if segment.ident == "Some" {
                let matched_expr = rust_expr_to_js_with_state(&let_expr.expr, state)?;
                let body_stmts =
                    rust_block_to_js_with_state(BlockAction::NoReturn, &while_expr.body, state)?;

                // Extract variable name from Some(x) pattern
                let var_name = if let Some(inner_pat) = tuple_struct.elems.first() {
                    if let Pat::Ident(pat_ident) = inner_pat {
                        Some(pat_ident.ident.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Generate optimized JavaScript:
                // while (true) {
                //   const _temp = expr;
                //   if (_temp === null || _temp === undefined) break;
                //   const x = _temp;  // if variable binding exists
                //   // body
                // }
                let mut loop_body = vec![
                    state.mk_var_decl("_temp", Some(matched_expr), true),
                    // if (_temp === null || _temp === undefined) break;
                    js::Stmt::If(js::IfStmt {
                        span: DUMMY_SP,
                        test: Box::new({
                            let null_check = state.mk_binary_expr(
                                js::Expr::Ident(state.mk_ident("_temp")),
                                js::BinaryOp::EqEqEq,
                                state.mk_null_lit(),
                            );
                            let undefined_check = state.mk_binary_expr(
                                js::Expr::Ident(state.mk_ident("_temp")),
                                js::BinaryOp::EqEqEq,
                                state.mk_undefined(),
                            );
                            state.mk_binary_expr(
                                null_check,
                                js::BinaryOp::LogicalOr,
                                undefined_check,
                            )
                        }),
                        cons: Box::new(js::Stmt::Break(js::BreakStmt {
                            span: DUMMY_SP,
                            label: None,
                        })),
                        alt: None,
                    }),
                ];

                // Add variable binding if present
                if let Some(var_name) = var_name {
                    let js_var_name = escape_js_identifier(&var_name);
                    state.declare_variable(var_name, js_var_name.clone(), false);
                    loop_body.push(state.mk_var_decl(
                        &js_var_name,
                        Some(js::Expr::Ident(state.mk_ident("_temp"))),
                        true,
                    ));
                }

                // Add the original body
                loop_body.extend(body_stmts);

                return Ok(js::Stmt::While(js::WhileStmt {
                    span: DUMMY_SP,
                    test: Box::new(state.mk_bool_lit(true)),
                    body: Box::new(js::Stmt::Block(js::BlockStmt {
                        span: DUMMY_SP,
                        stmts: loop_body,
                        ctxt: SyntaxContext::empty(),
                    })),
                }));
            }
        }
    }

    // Fall back to generic while-let handling
    convert_while_let_to_stmt(while_expr, let_expr, state)
}

/// Most sophisticated version with all optimizations from legacy code
fn convert_while_to_stmt_legacy_compatible(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Check for while-let patterns first
    if let Expr::Let(let_expr) = &*while_expr.cond {
        return convert_while_let_option_to_stmt(while_expr, let_expr, state);
    }

    // Regular while loop - but with potential optimizations
    let test = rust_expr_to_js_with_state(&while_expr.cond, state)?;
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &while_expr.body, state)?;

    // Check for infinite loop patterns like while true
    let is_infinite = match &*while_expr.cond {
        Expr::Lit(lit) => {
            if let syn::Lit::Bool(bool_lit) = &lit.lit {
                bool_lit.value
            } else {
                false
            }
        }
        Expr::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                segment.ident == "true"
            } else {
                false
            }
        }
        _ => false,
    };

    if is_infinite {
        // Convert while true to while (true) - cleaner than while (true === true)
        Ok(js::Stmt::While(js::WhileStmt {
            span: DUMMY_SP,
            test: Box::new(state.mk_bool_lit(true)),
            body: Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: body_stmts,
                ctxt: SyntaxContext::empty(),
            })),
        }))
    } else {
        // Regular while condition
        Ok(js::Stmt::While(js::WhileStmt {
            span: DUMMY_SP,
            test: Box::new(test),
            body: Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: body_stmts,
                ctxt: SyntaxContext::empty(),
            })),
        }))
    }
}

/// Context-aware wrapper that chooses between statement and expression handling
pub fn handle_while_context_aware(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
    in_statement_context: bool,
) -> Result<Either<js::Stmt, js::Expr>, String> {
    if in_statement_context {
        convert_while_to_stmt_legacy_compatible(while_expr, state).map(Either::Left)
    } else {
        handle_while_expr(while_expr, state).map(Either::Right)
    }
}

/// Core function that converts Rust for loop to JavaScript for-of statement
fn convert_for_to_stmt(
    for_expr: &syn::ExprForLoop,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Convert the iterable expression
    let iterable = rust_expr_to_js_with_state(&for_expr.expr, state)?;

    // Convert loop body
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &for_expr.body, state)?;

    match &*for_expr.pat {
        Pat::Ident(pat_ident) => {
            // Simple case: for x in items
            let loop_var = pat_ident.ident.to_string();
            let js_loop_var = escape_js_identifier(&loop_var);

            // Declare the variable in the current scope
            state.declare_variable(loop_var, js_loop_var.clone(), false);

            // Create for...of loop
            Ok(js::Stmt::ForOf(js::ForOfStmt {
                span: DUMMY_SP,
                is_await: false,
                left: js::ForHead::VarDecl(Box::new(js::VarDecl {
                    span: DUMMY_SP,
                    kind: js::VarDeclKind::Const,
                    declare: false,
                    decls: vec![js::VarDeclarator {
                        span: DUMMY_SP,
                        name: js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&js_loop_var),
                            type_ann: None,
                        }),
                        init: None,
                        definite: false,
                    }],
                    ctxt: SyntaxContext::empty(),
                })),
                right: Box::new(iterable),
                body: Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: body_stmts,
                    ctxt: SyntaxContext::empty(),
                })),
            }))
        }

        Pat::Tuple(tuple_pat) => {
            // Tuple destructuring case: for (i, element) in items.enumerate()
            let var_names: Vec<String> = tuple_pat
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

            if var_names.is_empty() {
                return Err("No valid identifiers found in tuple pattern".to_string());
            }

            // Create array destructuring pattern for the loop variable
            let destructure_pattern = js::Pat::Array(js::ArrayPat {
                span: DUMMY_SP,
                elems: var_names
                    .iter()
                    .map(|name| {
                        let js_name = escape_js_identifier(name);
                        state.declare_variable(name.clone(), js_name.clone(), false);
                        Some(js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&js_name),
                            type_ann: None,
                        }))
                    })
                    .collect(),
                optional: false,
                type_ann: None,
            });

            // Create for...of loop with destructuring
            Ok(js::Stmt::ForOf(js::ForOfStmt {
                span: DUMMY_SP,
                is_await: false,
                left: js::ForHead::VarDecl(Box::new(js::VarDecl {
                    span: DUMMY_SP,
                    kind: js::VarDeclKind::Const,
                    declare: false,
                    decls: vec![js::VarDeclarator {
                        span: DUMMY_SP,
                        name: destructure_pattern,
                        init: None,
                        definite: false,
                    }],
                    ctxt: SyntaxContext::empty(),
                })),
                right: Box::new(iterable),
                body: Box::new(js::Stmt::Block(js::BlockStmt {
                    span: DUMMY_SP,
                    stmts: body_stmts,
                    ctxt: SyntaxContext::empty(),
                })),
            }))
        }

        Pat::Type(type_pat) => {
            // Handle typed patterns like `for x: i32 in items`
            // Recursively handle the inner pattern, ignoring the type
            let inner_for_expr = syn::ExprForLoop {
                attrs: for_expr.attrs.clone(),
                label: for_expr.label.clone(),
                for_token: for_expr.for_token,
                pat: type_pat.pat.clone(),
                in_token: for_expr.in_token,
                expr: for_expr.expr.clone(),
                body: for_expr.body.clone(),
            };
            convert_for_to_stmt(&inner_for_expr, state)
        }

        _ => panic!("Unsupported for loop pattern {:?}", &*for_expr.pat),
    }
}

fn handle_for_expr(
    for_expr: &syn::ExprForLoop,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Reuse the statement converter and wrap in IIFE
    let for_stmt = convert_for_to_stmt(for_expr, state)?;
    Ok(state.mk_iife(vec![for_stmt]))
}

/// Enhanced version that detects enumerate patterns and generates optimized JavaScript
fn convert_for_to_stmt_enhanced(
    for_expr: &syn::ExprForLoop,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    // Check for enumerate pattern first
    if let (Pat::Tuple(tuple_pat), Some(collection_expr)) =
        (&*for_expr.pat, detect_enumerate_pattern(&for_expr.expr))
    {
        if tuple_pat.elems.len() == 2 {
            let index_var = if let Pat::Ident(ref pat_ident) = tuple_pat.elems[0] {
                pat_ident.ident.to_string()
            } else {
                "i".to_string()
            };

            let item_var = if let Pat::Ident(ref pat_ident) = tuple_pat.elems[1] {
                pat_ident.ident.to_string()
            } else {
                "item".to_string()
            };

            let js_index_var = escape_js_identifier(&index_var);
            let js_item_var = escape_js_identifier(&item_var);

            // Declare variables
            state.declare_variable(index_var, js_index_var.clone(), false);
            state.declare_variable(item_var, js_item_var.clone(), false);

            let iterable = rust_expr_to_js_with_state(&collection_expr, state)?;
            let body_stmts =
                rust_block_to_js_with_state(BlockAction::NoReturn, &for_expr.body, state)?;

            // Generate: let i = 0; for (const item of collection) { body; i++; }
            let mut stmts = vec![
                // let i = 0;
                state.mk_var_decl(&js_index_var, Some(state.mk_num_lit(0.0)), false),
                // for (const item of collection) { body; i++; }
                js::Stmt::ForOf(js::ForOfStmt {
                    span: DUMMY_SP,
                    is_await: false,
                    left: js::ForHead::VarDecl(Box::new(js::VarDecl {
                        span: DUMMY_SP,
                        kind: js::VarDeclKind::Const,
                        declare: false,
                        decls: vec![js::VarDeclarator {
                            span: DUMMY_SP,
                            name: js::Pat::Ident(js::BindingIdent {
                                id: state.mk_ident(&js_item_var),
                                type_ann: None,
                            }),
                            init: None,
                            definite: false,
                        }],
                        ctxt: SyntaxContext::empty(),
                    })),
                    right: Box::new(iterable),
                    body: Box::new(js::Stmt::Block(js::BlockStmt {
                        span: DUMMY_SP,
                        stmts: {
                            let mut loop_stmts = body_stmts;
                            // Add i++ at the end
                            loop_stmts.push(state.mk_expr_stmt(js::Expr::Update(js::UpdateExpr {
                                span: DUMMY_SP,
                                op: js::UpdateOp::PlusPlus,
                                prefix: false,
                                arg: Box::new(js::Expr::Ident(state.mk_ident(&js_index_var))),
                            })));
                            loop_stmts
                        },
                        ctxt: SyntaxContext::empty(),
                    })),
                }),
            ];

            // Return a block statement containing both statements
            return Ok(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts,
                ctxt: SyntaxContext::empty(),
            }));
        }
    }

    // Fall back to regular for loop handling
    convert_for_to_stmt(for_expr, state)
}

/// Helper function to detect enumerate patterns
fn detect_enumerate_pattern(expr: &Expr) -> Option<Expr> {
    if let Expr::MethodCall(method_call) = expr {
        if method_call.method == "enumerate" {
            // Check if the receiver is a .iter() call
            if let Expr::MethodCall(iter_call) = &*method_call.receiver {
                if iter_call.method == "iter" {
                    return Some((*iter_call.receiver).clone());
                }
            }
            // Could also be direct enumerate on collection
            return Some((*method_call.receiver).clone());
        }
    }
    None
}

/// Context-aware wrapper that chooses between statement and expression handling
pub fn handle_for_context_aware(
    for_expr: &syn::ExprForLoop,
    state: &mut TranspilerState,
    in_statement_context: bool,
) -> Result<Either<js::Stmt, js::Expr>, String> {
    if in_statement_context {
        convert_for_to_stmt_enhanced(for_expr, state).map(Either::Left)
    } else {
        handle_for_expr(for_expr, state).map(Either::Right)
    }
}

// Helper enum for returning either statement or expression
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Handle pattern matching and variable binding - shared between match and if-let
fn handle_pattern_binding(
    pat: &Pat,
    match_var: &str,
    state: &mut TranspilerState,
) -> Result<(js::Expr, Vec<js::Stmt>), String> {
    let mut binding_stmts = Vec::new();

    let condition = match pat {
        Pat::Lit(lit_pat) => {
            let lit_expr = match &lit_pat.lit {
                syn::Lit::Str(s) => state.mk_str_lit(&s.value()),
                syn::Lit::Int(i) => {
                    let value = i
                        .base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse integer: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Float(f) => {
                    let value = f
                        .base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse float: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Bool(b) => state.mk_bool_lit(b.value()),
                syn::Lit::Char(c) => state.mk_str_lit(&c.value().to_string()),
                x => panic!("Unsupported literal in pattern {:?}", x),
            };

            state.mk_binary_expr(
                js::Expr::Ident(state.mk_ident(match_var)),
                js::BinaryOp::EqEqEq,
                lit_expr,
            )
        }
        Pat::Wild(_) => {
            // Wildcard pattern always matches
            state.mk_bool_lit(true)
        }
        Pat::Ident(pat_ident) => {
            // Variable binding - always matches, and we need to bind the variable
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            state.declare_variable(var_name, js_var_name.clone(), false);

            // Add: const x = _match_value;
            binding_stmts.push(state.mk_var_decl(
                &js_var_name,
                Some(js::Expr::Ident(state.mk_ident(match_var))),
                true,
            ));

            state.mk_bool_lit(true)
        }
        Pat::Path(path_pat) => {
            // Handle enum variants like None, Some
            if let Some(segment) = path_pat.path.segments.last() {
                let variant_name = segment.ident.to_string();
                match variant_name.as_str() {
                    "None" => {
                        // Check for null or undefined
                        let null_check = state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_null_lit(),
                        );
                        let undefined_check = state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_undefined(),
                        );
                        state.mk_binary_expr(null_check, js::BinaryOp::LogicalOr, undefined_check)
                    }
                    _ => {
                        // For other enum variants, compare against string
                        state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_str_lit(&variant_name),
                        )
                    }
                }
            } else {
                return Err("Invalid path pattern".to_string());
            }
        }
        Pat::TupleStruct(tuple_struct) => {
            // Handle Some(x) patterns
            if let Some(segment) = tuple_struct.path.segments.last() {
                if segment.ident == "Some" {
                    // Check that value is not null/undefined
                    let not_null = state.mk_binary_expr(
                        js::Expr::Ident(state.mk_ident(match_var)),
                        js::BinaryOp::NotEqEq,
                        state.mk_null_lit(),
                    );
                    let not_undefined = state.mk_binary_expr(
                        js::Expr::Ident(state.mk_ident(match_var)),
                        js::BinaryOp::NotEqEq,
                        state.mk_undefined(),
                    );

                    // If there's a variable binding in the pattern, handle it
                    if let Some(inner_pat) = tuple_struct.elems.first() {
                        if let Pat::Ident(pat_ident) = inner_pat {
                            let var_name = pat_ident.ident.to_string();
                            let js_var_name = escape_js_identifier(&var_name);
                            state.declare_variable(var_name, js_var_name.clone(), false);

                            // Add: const x = _match_value;
                            binding_stmts.push(state.mk_var_decl(
                                &js_var_name,
                                Some(js::Expr::Ident(state.mk_ident(match_var))),
                                true,
                            ));
                        }
                    }

                    state.mk_binary_expr(not_null, js::BinaryOp::LogicalAnd, not_undefined)
                } else {
                    // Handle generic enum variants with data - e.g., TestMessage::MessageOne(s)
                    let variant_name = segment.ident.to_string();
                    
                    // Special case for Result<T,E> patterns: Ok/Err should use legacy format
                    let (condition_field, data_field) = match variant_name.as_str() {
                        "Ok" => ("ok", "ok"),
                        "Err" => ("error", "error"), 
                        _ => ("type", "value0"), // Generic enum pattern
                    };
                    
                    // Generate condition based on the pattern type
                    let type_check = if variant_name == "Ok" || variant_name == "Err" {
                        // For Result patterns: check _match_value.ok !== undefined or _match_value.error !== undefined
                        state.mk_binary_expr(
                            state.mk_member_expr(
                                js::Expr::Ident(state.mk_ident(match_var)),
                                condition_field
                            ),
                            js::BinaryOp::NotEqEq,
                            state.mk_undefined(),
                        )
                    } else {
                        // For generic enums: _match_value.type === 'VariantName'
                        state.mk_binary_expr(
                            state.mk_member_expr(
                                js::Expr::Ident(state.mk_ident(match_var)),
                                "type"
                            ),
                            js::BinaryOp::EqEqEq,
                            state.mk_str_lit(&variant_name),
                        )
                    };
                    
                    // Handle parameter binding for enum variant data
                    for (i, inner_pat) in tuple_struct.elems.iter().enumerate() {
                        match inner_pat {
                            Pat::Ident(pat_ident) => {
                                let var_name = pat_ident.ident.to_string();
                                let js_var_name = escape_js_identifier(&var_name);
                                state.declare_variable(var_name, js_var_name.clone(), false);
                                
                                // Generate appropriate field access based on pattern type
                                let field_name = if variant_name == "Ok" || variant_name == "Err" {
                                    // For Result patterns: use data_field (ok/error)
                                    data_field.to_string()
                                } else {
                                    // For generic enums: use value0, value1, etc.
                                    format!("value{}", i)
                                };
                                
                                binding_stmts.push(state.mk_var_decl(
                                    &js_var_name,
                                    Some(state.mk_member_expr(
                                        js::Expr::Ident(state.mk_ident(match_var)),
                                        &field_name
                                    )),
                                    true,
                                ));
                            }
                            Pat::Wild(_) => {
                                // Wildcard pattern - ignore this field, no binding needed
                                // This allows patterns like SomeEnum(value, _) where the second field is ignored
                            }
                            _ => {
                                panic!("Complex patterns inside tuple struct not yet supported: {:?}", inner_pat);
                            }
                        }
                    }
                    
                    type_check
                }
            } else {
                panic!(
                    "Unsupported tuple struct last segment {:?}",
                    &tuple_struct.path.segments
                );
            }
        }
        Pat::Struct(struct_pat) => {
            // Handle struct-style enum variants: TestMessage::MessageOne { one } => { ... }
            if let Some(segment) = struct_pat.path.segments.last() {
                let variant_name = segment.ident.to_string();
                
                // Generate condition: _match_value.type === 'MessageOne'
                let type_check = state.mk_binary_expr(
                    state.mk_member_expr(
                        js::Expr::Ident(state.mk_ident(match_var)),
                        "type"
                    ),
                    js::BinaryOp::EqEqEq,
                    state.mk_str_lit(&variant_name),
                );
                
                // Handle field binding for struct-style enum variants
                for field_pat in &struct_pat.fields {
                    if let syn::Member::Named(field_name) = &field_pat.member {
                        if let Pat::Ident(pat_ident) = &*field_pat.pat {
                            let var_name = pat_ident.ident.to_string();
                            let js_var_name = escape_js_identifier(&var_name);
                            let field_name_str = field_name.to_string();
                            
                            state.declare_variable(var_name, js_var_name.clone(), false);
                            
                            // Generate: const one = _match_value.one;
                            binding_stmts.push(state.mk_var_decl(
                                &js_var_name,
                                Some(state.mk_member_expr(
                                    js::Expr::Ident(state.mk_ident(match_var)),
                                    &field_name_str
                                )),
                                true,
                            ));
                        } else {
                            panic!("Complex patterns in struct fields not yet supported: {:?}", field_pat.pat);
                        }
                    }
                }
                
                type_check
            } else {
                panic!("Invalid struct pattern path: {:?}", struct_pat.path);
            }
        }
        Pat::Tuple(tuple_pat) => {
            // Handle tuple patterns like (Some(token), Some(signature))
            // Use direct array access _match_value[i] without intermediate variables
            let mut conditions = Vec::new();
            
            for (i, elem_pat) in tuple_pat.elems.iter().enumerate() {
                // Create direct access expression: _match_value[i]
                let array_access_expr = js::Expr::Member(js::MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(js::Expr::Ident(state.mk_ident(match_var))),
                    prop: js::MemberProp::Computed(js::ComputedPropName {
                        span: DUMMY_SP,
                        expr: Box::new(state.mk_num_lit(i as f64)),
                    }),
                });
                
                // Handle different pattern types for tuple elements
                match elem_pat {
                    Pat::Ident(pat_ident) => {
                        // Simple variable binding: (a, b) => const a = _match_value[0]; const b = _match_value[1];
                        let var_name = pat_ident.ident.to_string();
                        let js_var_name = escape_js_identifier(&var_name);
                        state.declare_variable(var_name, js_var_name.clone(), false);
                        
                        binding_stmts.push(state.mk_var_decl(
                            &js_var_name,
                            Some(array_access_expr),
                            true,
                        ));
                        
                        // Always matches for variable binding
                        conditions.push(state.mk_bool_lit(true));
                    }
                    Pat::TupleStruct(tuple_struct) => {
                        // Handle Some(token), None patterns in tuples
                        if let Some(segment) = tuple_struct.path.segments.last() {
                            if segment.ident == "Some" {
                                // Check _match_value[i] !== null && _match_value[i] !== undefined
                                let not_null = state.mk_binary_expr(
                                    array_access_expr.clone(),
                                    js::BinaryOp::NotEqEq,
                                    state.mk_null_lit(),
                                );
                                let not_undefined = state.mk_binary_expr(
                                    array_access_expr.clone(),
                                    js::BinaryOp::NotEqEq,
                                    state.mk_undefined(),
                                );
                                
                                conditions.push(state.mk_binary_expr(
                                    not_null, 
                                    js::BinaryOp::LogicalAnd, 
                                    not_undefined
                                ));
                                
                                // If there's a variable binding inside Some(var)
                                if let Some(inner_pat) = tuple_struct.elems.first() {
                                    if let Pat::Ident(pat_ident) = inner_pat {
                                        let var_name = pat_ident.ident.to_string();
                                        let js_var_name = escape_js_identifier(&var_name);
                                        state.declare_variable(var_name, js_var_name.clone(), false);
                                        
                                        binding_stmts.push(state.mk_var_decl(
                                            &js_var_name,
                                            Some(array_access_expr),
                                            true,
                                        ));
                                    }
                                }
                            } else {
                                // Handle other enum variants
                                let elem_condition = state.mk_binary_expr(
                                    state.mk_member_expr(array_access_expr, "type"),
                                    js::BinaryOp::EqEqEq,
                                    state.mk_str_lit(&segment.ident.to_string()),
                                );
                                conditions.push(elem_condition);
                            }
                        }
                    }
                    Pat::Path(path_pat) => {
                        // Handle None patterns in tuples
                        if let Some(segment) = path_pat.path.segments.last() {
                            if segment.ident == "None" {
                                // Check _match_value[i] === null || _match_value[i] === undefined
                                let null_check = state.mk_binary_expr(
                                    array_access_expr.clone(),
                                    js::BinaryOp::EqEqEq,
                                    state.mk_null_lit(),
                                );
                                let undefined_check = state.mk_binary_expr(
                                    array_access_expr,
                                    js::BinaryOp::EqEqEq,
                                    state.mk_undefined(),
                                );
                                conditions.push(state.mk_binary_expr(
                                    null_check, 
                                    js::BinaryOp::LogicalOr, 
                                    undefined_check
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(format!("Unsupported pattern in tuple: {:?}", elem_pat));
                    }
                }
            }
            
            // Combine all conditions with AND
            if conditions.is_empty() {
                state.mk_bool_lit(true)
            } else {
                conditions.into_iter().reduce(|acc, cond| {
                    state.mk_binary_expr(acc, js::BinaryOp::LogicalAnd, cond)
                }).unwrap()
            }
        }
        x => panic!("Unsupported pattern {:?}", &x),
    };

    Ok((condition, binding_stmts))
}

/// Handle match expressions
fn handle_match_expr(
    match_expr: &syn::ExprMatch,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let match_value = rust_expr_to_js_with_state(&match_expr.expr, state)?;
    // Arguably the below is better but for now we keep compatible with old
    // let temp_var = state.generate_temp_var();
    let temp_var = "_match_value".to_string();

    let mut stmts = vec![state.mk_var_decl(&temp_var, Some(match_value), true)];

    let mut if_chain: Option<js::Stmt> = None;

    for (i, arm) in match_expr.arms.iter().enumerate() {
        let (condition, mut binding_stmts) = handle_pattern_binding(&arm.pat, &temp_var, state)?;
        // Use BlockAction::Return so that final expressions in match arms are properly returned
        let body_expr = rust_expr_to_js_with_action_and_state(BlockAction::Return, &arm.body, state)?;

        // Combine binding statements with return statement
        binding_stmts.push(state.mk_return_stmt(Some(body_expr)));

        let current_if = js::Stmt::If(js::IfStmt {
            span: DUMMY_SP,
            test: Box::new(condition),
            cons: Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: binding_stmts,
                ctxt: SyntaxContext::empty(),
            })),
            alt: None,
        });

        if i == 0 {
            if_chain = Some(current_if);
        } else {
            // Chain the if statements
            if let Some(ref mut chain) = if_chain {
                chain_if_statement(chain, current_if);
            }
        }
    }

    if let Some(if_stmt) = if_chain {
        stmts.push(if_stmt);
    }

    // Add default return
    stmts.push(state.mk_return_stmt(Some(state.mk_undefined())));

    Ok(state.mk_iife(stmts))
}

/// Create a condition expression for a match arm pattern
fn create_match_condition(
    pat: &Pat,
    match_var: &str,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    match pat {
        Pat::Lit(lit_pat) => {
            let lit_expr = match &lit_pat.lit {
                syn::Lit::Str(s) => state.mk_str_lit(&s.value()),
                syn::Lit::Int(i) => {
                    let value = i
                        .base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse integer: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Float(f) => {
                    let value = f
                        .base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse float: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Bool(b) => state.mk_bool_lit(b.value()),
                syn::Lit::Char(c) => state.mk_str_lit(&c.value().to_string()),
                x => panic!("Unsupported literal in match pattern: {:?}", x),
            };

            Ok(state.mk_binary_expr(
                js::Expr::Ident(state.mk_ident(match_var)),
                js::BinaryOp::EqEqEq,
                lit_expr,
            ))
        }
        Pat::Wild(_) => {
            // Wildcard pattern always matches
            Ok(state.mk_bool_lit(true))
        }
        Pat::Ident(pat_ident) => {
            // Variable binding - always matches, but we need to bind the variable
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            state.declare_variable(var_name, js_var_name, false);
            Ok(state.mk_bool_lit(true))
        }
        Pat::Path(path_pat) => {
            // Handle enum variants like None, Some
            if let Some(segment) = path_pat.path.segments.last() {
                let variant_name = segment.ident.to_string();
                match variant_name.as_str() {
                    "None" => {
                        // Check for null or undefined
                        let null_check = state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_null_lit(),
                        );
                        let undefined_check = state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_undefined(),
                        );
                        Ok(state.mk_binary_expr(
                            null_check,
                            js::BinaryOp::LogicalOr,
                            undefined_check,
                        ))
                    }
                    _ => {
                        // For other enum variants, compare against string
                        Ok(state.mk_binary_expr(
                            js::Expr::Ident(state.mk_ident(match_var)),
                            js::BinaryOp::EqEqEq,
                            state.mk_str_lit(&variant_name),
                        ))
                    }
                }
            } else {
                Err("Invalid path pattern".to_string())
            }
        }
        Pat::TupleStruct(tuple_struct) => {
            // Handle Some(x) patterns
            if let Some(segment) = tuple_struct.path.segments.last() {
                if segment.ident == "Some" {
                    // Check that value is not null/undefined
                    let not_null = state.mk_binary_expr(
                        js::Expr::Ident(state.mk_ident(match_var)),
                        js::BinaryOp::NotEqEq,
                        state.mk_null_lit(),
                    );
                    let not_undefined = state.mk_binary_expr(
                        js::Expr::Ident(state.mk_ident(match_var)),
                        js::BinaryOp::NotEqEq,
                        state.mk_undefined(),
                    );

                    // If there's a variable binding in the pattern, handle it
                    if let Some(inner_pat) = tuple_struct.elems.first() {
                        if let Pat::Ident(pat_ident) = inner_pat {
                            let var_name = pat_ident.ident.to_string();
                            let js_var_name = escape_js_identifier(&var_name);
                            state.declare_variable(var_name, js_var_name, false);
                        }
                    }

                    Ok(state.mk_binary_expr(not_null, js::BinaryOp::LogicalAnd, not_undefined))
                } else {
                    panic!("Unsupported tuple struct pattern {:?}", &segment);
                }
            } else {
                panic!(
                    "Invalid tuple struct pattern {:?}",
                    &tuple_struct.path.segments
                )
            }
        }
        x => panic!("Unsupported match pattern {:?}", &x),
    }
}

/// Helper function to chain if statements for match arms
fn chain_if_statement(current: &mut js::Stmt, next: js::Stmt) {
    if let js::Stmt::If(if_stmt) = current {
        if if_stmt.alt.is_none() {
            // Check if the next statement is "if (true)" and convert to simple else
            if let js::Stmt::If(next_if) = &next {
                if is_true_condition(&next_if.test) {
                    // Convert "else if (true)" to just "else { // Default case"
                    // We need to add the comment to the block somehow
                    if let js::Stmt::Block(block_stmt) = &*next_if.cons {
                        // Create a new block with a comment by modifying the first statement
                        let mut new_stmts = block_stmt.stmts.clone();
                        if !new_stmts.is_empty() {
                            // Add comment as an expression statement at the beginning
                            let comment_stmt = js::Stmt::Expr(js::ExprStmt {
                                span: DUMMY_SP,
                                expr: Box::new(js::Expr::Ident(js::Ident::new(
                                    "// Default case".into(),
                                    DUMMY_SP,
                                    SyntaxContext::empty(),
                                ))),
                            });
                            new_stmts.insert(0, comment_stmt);
                        }

                        let new_block = js::Stmt::Block(js::BlockStmt {
                            span: DUMMY_SP,
                            stmts: new_stmts,
                            ctxt: SyntaxContext::empty(),
                        });
                        if_stmt.alt = Some(Box::new(new_block));
                    } else {
                        if_stmt.alt = Some(next_if.cons.clone());
                    }
                } else {
                    if_stmt.alt = Some(Box::new(next));
                }
            } else {
                if_stmt.alt = Some(Box::new(next));
            }
        } else {
            // Recursively chain
            if let Some(ref mut alt) = if_stmt.alt {
                chain_if_statement(alt, next);
            }
        }
    }
}
/// Helper to check if a condition is just "true"
fn is_true_condition(expr: &js::Expr) -> bool {
    match expr {
        js::Expr::Lit(js::Lit::Bool(js::Bool { value: true, .. })) => true,
        js::Expr::Ident(ident) if ident.sym == "true" => true, // Can be removed
        _ => false,
    }
}

/// Handle loop expressions (infinite loops)
fn handle_loop_expr(
    loop_expr: &syn::ExprLoop,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let body_stmts = rust_block_to_js_with_state(BlockAction::NoReturn, &loop_expr.body, state)?;

    let while_stmt = js::Stmt::While(js::WhileStmt {
        span: DUMMY_SP,
        test: Box::new(state.mk_bool_lit(true)),
        body: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: body_stmts,
            ctxt: SyntaxContext::empty(),
        })),
    });

    Ok(state.mk_iife(vec![while_stmt]))
}

/// Handle closure expressions
fn handle_closure_expr(
    closure: &syn::ExprClosure,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Extract parameter names
    let params: Vec<js::Pat> = closure
        .inputs
        .iter()
        .map(|param| {
            match param {
                Pat::Ident(pat_ident) => js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident(&pat_ident.ident.to_string()),
                    type_ann: None,
                }),
                Pat::Reference(ref_pat) => {
                    // Handle reference patterns like &x, &&x
                    let ident = extract_ident_from_pattern(&ref_pat.pat)
                        .unwrap_or_else(|| panic!("Failed to extract identifier from reference pattern: {:?}", ref_pat));
                    js::Pat::Ident(js::BindingIdent {
                        id: state.mk_ident(&ident),
                        type_ann: None,
                    })
                }
                Pat::Type(type_pat) => {
                    // Handle typed patterns like |e: TestEvent| or |_: String|
                    match &*type_pat.pat {
                        Pat::Wild(_) => {
                            // Handle typed wildcard patterns |_: Type| -> generate placeholder name
                            let placeholder_name = format!("_unused_{}", closure.inputs.iter().position(|p| std::ptr::eq(p, param)).unwrap_or(0));
                            js::Pat::Ident(js::BindingIdent {
                                id: state.mk_ident(&placeholder_name),
                                type_ann: None,
                            })
                        }
                        _ => {
                            // Handle normal typed patterns |e: TestEvent| -> extract 'e' from the type annotation
                            let ident = extract_ident_from_pattern(&type_pat.pat)
                                .unwrap_or_else(|| panic!("Failed to extract identifier from typed pattern: {:?}", type_pat));
                            js::Pat::Ident(js::BindingIdent {
                                id: state.mk_ident(&ident),
                                type_ann: None,
                            })
                        }
                    }
                }
                Pat::Wild(_) => {
                    // Handle wildcard patterns |_| -> generate a placeholder parameter name
                    let placeholder_name = format!("_unused_{}", closure.inputs.iter().position(|p| std::ptr::eq(p, param)).unwrap_or(0));
                    js::Pat::Ident(js::BindingIdent {
                        id: state.mk_ident(&placeholder_name),
                        type_ann: None,
                    })
                }
                _ => panic!("Unsupported closure parameter pattern: {:?}", param),
            }
        })
        .collect();

    // Handle closure body
    let body = match &*closure.body {
        Expr::Block(block_expr) => {
            let stmts = rust_block_to_js_with_state(BlockAction::Return, &block_expr.block, state)?;
            js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
                span: DUMMY_SP,
                stmts,
                ctxt: SyntaxContext::empty(),
            })
        }
        _ => {
            let expr = rust_expr_to_js_with_state(&closure.body, state)?;
            js::BlockStmtOrExpr::Expr(Box::new(expr))
        }
    };

    let arrow_expr = js::Expr::Arrow(js::ArrowExpr {
        span: DUMMY_SP,
        params,
        body: Box::new(body),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    });

    // Wrap the arrow function in parentheses for better readability
    Ok(js::Expr::Paren(js::ParenExpr {
        span: DUMMY_SP,
        expr: Box::new(arrow_expr),
    }))
}

// Helper function to recursively extract identifier from nested reference patterns
fn extract_ident_from_pattern(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(pat_ident) => Some(pat_ident.ident.to_string()),
        Pat::Reference(ref_pat) => extract_ident_from_pattern(&ref_pat.pat),
        _ => None,
    }
}

/// Handle struct literal expressions
fn handle_struct_expr(
    struct_expr: &syn::ExprStruct,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let struct_name = if let Some(ident) = struct_expr.path.get_ident() {
        ident.to_string()
    } else {
        struct_expr.path.segments.last().unwrap().ident.to_string()
    };

    // Check if this is an enum variant (has multiple path segments like EnumName::VariantName)
    if struct_expr.path.segments.len() > 1 {
        // This is an enum variant like TestMessage::Hello { name: "World" }
        let enum_name = struct_expr.path.segments.first().unwrap().ident.to_string();
        let variant_name = struct_expr.path.segments.last().unwrap().ident.to_string();
        
        // Create object with named fields for enum variant
        let mut props = Vec::new();
        
        // Add the type discriminant field
        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
            js::KeyValueProp {
                key: js::PropName::Ident(state.mk_ident_name("type")),
                value: Box::new(state.mk_str_lit(&variant_name)),
            },
        ))));
        
        // Add the actual fields
        for field in &struct_expr.fields {
            if let syn::Member::Named(field_name) = &field.member {
                let field_name_str = field_name.to_string();
                let field_value = rust_expr_to_js_with_state(&field.expr, state)?;
                
                props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                        value: Box::new(field_value),
                    },
                ))));
            }
        }
        
        // Return object literal instead of constructor call
        Ok(js::Expr::Object(js::ObjectLit {
            span: DUMMY_SP,
            props,
        }))
    } else {
        // Check if this is a Self struct expression
        if struct_name == "Self" && state.get_current_struct_name().is_some() {
            // Check if we're in a static method context
            if state.is_in_static_method() {
                // In static methods, Self { ... } should use constructor + field assignment
                // We'll use an IIFE to create the object and assign fields
                let struct_name = state.get_current_struct_name().unwrap().clone();
                
                // Create constructor call: new StructName()
                let constructor_call = js::Expr::New(js::NewExpr {
                    span: DUMMY_SP,
                    callee: Box::new(js::Expr::Ident(state.mk_ident(&struct_name))),
                    args: Some(vec![]),
                    type_args: None,
                    ctxt: SyntaxContext::empty(),
                });
                
                // Create variable declaration: const obj = new StructName();
                let obj_var = state.mk_var_decl("obj", Some(constructor_call), true);
                
                // Create field assignments: obj.field = value;
                let mut assignment_stmts = vec![obj_var];
                for field in &struct_expr.fields {
                    if let syn::Member::Named(field_name) = &field.member {
                        let field_name_str = field_name.to_string();
                        let field_value = rust_expr_to_js_with_state(&field.expr, state)?;
                        
                        // Create obj.field assignment
                        let obj_access = state.mk_member_expr(
                            js::Expr::Ident(state.mk_ident("obj")),
                            &field_name_str
                        );
                        let assignment = js::Expr::Assign(js::AssignExpr {
                            span: DUMMY_SP,
                            op: js::AssignOp::Assign,
                            left: state.expr_to_assign_target(obj_access)?,
                            right: Box::new(field_value),
                        });
                        assignment_stmts.push(js::Stmt::Expr(js::ExprStmt {
                            span: DUMMY_SP,
                            expr: Box::new(assignment),
                        }));
                    }
                }
                
                // Add return statement: return obj;
                assignment_stmts.push(js::Stmt::Return(js::ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(js::Expr::Ident(state.mk_ident("obj")))),
                }));
                
                // Create IIFE: (() => { ... })()
                let iife_func = js::ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: Box::new(js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
                        span: DUMMY_SP,
                        stmts: assignment_stmts,
                        ctxt: SyntaxContext::empty(),
                    })),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                    ctxt: SyntaxContext::empty(),
                };
                
                // Call the IIFE
                Ok(js::Expr::Call(js::CallExpr {
                    span: DUMMY_SP,
                    callee: js::Callee::Expr(Box::new(js::Expr::Paren(js::ParenExpr {
                        span: DUMMY_SP,
                        expr: Box::new(js::Expr::Arrow(iife_func)),
                    }))),
                    args: vec![],
                    type_args: None,
                    ctxt: SyntaxContext::empty(),
                }))
            } else {
                // In non-static methods, Self { ... } should create an object literal with named fields
                let mut props = Vec::new();
                
                // Convert named fields to object properties
                for field in &struct_expr.fields {
                    if let syn::Member::Named(field_name) = &field.member {
                        let field_name_str = field_name.to_string();
                        let field_value = rust_expr_to_js_with_state(&field.expr, state)?;
                        
                        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                                value: Box::new(field_value),
                            },
                        ))));
                    }
                }
                
                // Return object literal for Self
                Ok(js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                }))
            }
        } else {
            // This is a regular struct, use original constructor pattern
            // Convert field initializers to constructor arguments
            let args: Result<Vec<_>, _> = struct_expr
                .fields
                .iter()
                .map(|field| rust_expr_to_js_with_state(&field.expr, state))
                .collect();

            let js_args = args?;

            // Create new constructor call
            let constructor = js::Expr::Ident(state.mk_ident(&struct_name));

            Ok(js::Expr::New(js::NewExpr {
                span: DUMMY_SP,
                callee: Box::new(constructor),
                args: Some(
                    js_args
                        .into_iter()
                        .map(|expr| js::ExprOrSpread {
                            spread: None,
                            expr: Box::new(expr),
                        })
                        .collect(),
                ),
                type_args: None,
                ctxt: SyntaxContext::empty(),
            }))
        }
    }
}

/// Handle range expressions
fn handle_range_expr(
    range_expr: &syn::ExprRange,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    match (&range_expr.start, &range_expr.end) {
        (Some(start), Some(end)) => {
            let start_js = rust_expr_to_js_with_state(start, state)?;
            let end_js = rust_expr_to_js_with_state(end, state)?;

            // Create Array.from({length: end - start}, (_, i) => i + start)
            let array_from = state.mk_member_expr(js::Expr::Ident(state.mk_ident("Array")), "from");

            let length_expr =
                state.mk_binary_expr(end_js.clone(), js::BinaryOp::Sub, start_js.clone());

            let length_obj = js::Expr::Object(js::ObjectLit {
                span: DUMMY_SP,
                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("length")),
                        value: Box::new(length_expr),
                    },
                )))],
            });

            // Create (_, i) => i + start
            let arrow_params = vec![
                js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident("_"),
                    type_ann: None,
                }),
                js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident("i"),
                    type_ann: None,
                }),
            ];

            let arrow_body = state.mk_binary_expr(
                js::Expr::Ident(state.mk_ident("i")),
                js::BinaryOp::Add,
                start_js,
            );

            let arrow_fn = js::ArrowExpr {
                span: DUMMY_SP,
                params: arrow_params,
                body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(arrow_body))),
                is_async: false,
                is_generator: false,
                type_params: None,
                return_type: None,
                ctxt: SyntaxContext::empty(),
            };

            Ok(state.mk_call_expr(array_from, vec![length_obj, js::Expr::Arrow(arrow_fn)]))
        }
        (None, Some(end)) => {
            // Range to end (..end) -> Array.from({length: end}, (_, i) => i)
            let end_js = rust_expr_to_js_with_state(end, state)?;
            let array_from = state.mk_member_expr(js::Expr::Ident(state.mk_ident("Array")), "from");

            let length_obj = js::Expr::Object(js::ObjectLit {
                span: DUMMY_SP,
                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("length")),
                        value: Box::new(end_js),
                    },
                )))],
            });

            let arrow_fn = js::ArrowExpr {
                span: DUMMY_SP,
                params: vec![
                    js::Pat::Ident(js::BindingIdent {
                        id: state.mk_ident("_"),
                        type_ann: None,
                    }),
                    js::Pat::Ident(js::BindingIdent {
                        id: state.mk_ident("i"),
                        type_ann: None,
                    }),
                ],
                body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(js::Expr::Ident(
                    state.mk_ident("i"),
                )))),
                is_async: false,
                is_generator: false,
                type_params: None,
                return_type: None,
                ctxt: SyntaxContext::empty(),
            };

            Ok(state.mk_call_expr(array_from, vec![length_obj, js::Expr::Arrow(arrow_fn)]))
        }
        x => {
            // Other range types not easily representable
            state.add_warning("Infinite or complex ranges not fully supported".to_string());
            panic!("Infinite or complex ranges not fully supported: {:?}", x);
            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: vec![],
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_method_generation() {
        let impl_block: ItemImpl = parse_quote! {
            impl MyStruct {
                fn new(value: i32) -> Self {
                    Self { value }
                }

                fn get_value(&self) -> i32 {
                    self.value
                }
            }
        };

        let result = transpile_impl_to_js(&impl_block);
        assert!(result.is_ok());
        let js_code = result.unwrap();
        assert!(js_code.contains("MyStruct.new"));
        assert!(js_code.contains("MyStruct.prototype.get_value"));
    }

    #[test]
    fn test_struct_generation() {
        let struct_def: ItemStruct = parse_quote! {
            struct Point {
                x: f64,
                y: f64,
            }
        };

        let result = transpile_struct_to_js(&struct_def);
        assert!(result.is_ok());
        let js_code = result.unwrap();
        assert!(js_code.contains("class Point"));
        assert!(js_code.contains("constructor"));
    }

    #[test]
    fn test_enum_generation() {
        let enum_def: ItemEnum = parse_quote! {
            enum Status {
                Active,
                Inactive,
                Pending(String),
            }
        };

        let result = transpile_enum_to_js(&enum_def);
        assert!(result.is_ok());
        let js_code = result.unwrap();
        assert!(js_code.contains("const Status"));
        assert!(js_code.contains("Active"));
        assert!(js_code.contains("Pending"));
    }
}

/// Generate JavaScript methods for a Rust impl block (old API)
pub fn generate_js_methods_for_impl(input_impl: &ItemImpl) -> String {
    let mut state = TranspilerState::new();

    let module_items = generate_js_methods_for_impl_with_state(input_impl)
        .expect("Failed to generate JavaScript methods for impl block");

    ast_to_code(&module_items).expect("Failed to convert AST to JavaScript code")
}

/// Handle format macro (old API)
pub fn handle_format_macro(args: &Punctuated<Expr, Comma>) -> String {
    let mut state = TranspilerState::new();

    let expr =
        handle_format_macro_with_state(args, &mut state).expect("Failed to handle format macro");

    // Convert single expression to code
    let module_items = vec![js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(expr),
    }))];

    let code = ast_to_code_trimmed(&module_items)
        .expect("Failed to convert format macro to JavaScript code");
    code
}

/// Handle reference expressions (&x, &mut y, &expr)
/// Handle reference expressions (&x, &mut y, &expr)
fn handle_reference_expr(
    ref_expr: &syn::ExprReference,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let inner_expr = rust_expr_to_js_with_state(&ref_expr.expr, state)?;

    match &*ref_expr.expr {
        // For simple path expressions (variables), handle specially
        Expr::Path(_) => {
            if ref_expr.mutability.is_some() {
                // Mutable reference - return the variable with a comment
                // We need to create a JavaScript comment, not a string literal
                Ok(js::Expr::Ident(js::Ident::new(
                    format!(
                        "{} /* was &mut in Rust */",
                        match &inner_expr {
                            js::Expr::Ident(ident) => ident.sym.to_string(),
                            _ => "unknown".to_string(),
                        }
                    )
                    .into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                )))
            } else {
                // Immutable reference to variable - just return the variable
                Ok(inner_expr)
            }
        }
        // For string literals, just return the literal (no comment needed)
        Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)) => Ok(inner_expr),
        // For other expressions, we need to create a comment expression
        _ => {
            let comment = if ref_expr.mutability.is_some() {
                " /* was &mut in Rust */"
            } else {
                " /* was & in Rust */"
            };

            // Create an identifier with the expression and comment combined
            // This is a bit of a hack, but it's the cleanest way to add comments
            Ok(js::Expr::Ident(js::Ident::new(
                format!(
                    "{}{}",
                    // Convert the inner expression back to string
                    match ast_to_code_trimmed(&[js::ModuleItem::Stmt(js::Stmt::Expr(
                        js::ExprStmt {
                            span: DUMMY_SP,
                            expr: Box::new(inner_expr.clone()),
                        }
                    ))]) {
                        Ok(code) => code.trim_end_matches(';').trim().to_string(),
                        Err(_) => "expr".to_string(),
                    },
                    comment
                )
                .into(),
                DUMMY_SP,
                SyntaxContext::empty(),
            )))
        }
    }
}

/// Convert Rust block to JavaScript (old API)
pub fn rust_block_to_js(block: &Block) -> String {
    let mut state = TranspilerState::new();

    let stmts = rust_block_to_js_with_state(BlockAction::Return, block, &mut state)
        .expect("Failed to convert Rust block to JavaScript");

    let module_items: Vec<js::ModuleItem> = stmts
        .into_iter()
        .map(|stmt| js::ModuleItem::Stmt(stmt))
        .collect();

    ast_to_code(&module_items).expect("Failed to convert block AST to JavaScript code")
}

/// Convert Rust expression to JavaScript (old API)
pub fn rust_expr_to_js(expr: &Expr) -> String {
    let mut state = TranspilerState::new();

    let js_expr = rust_expr_to_js_with_action_and_state(BlockAction::Return, expr, &mut state)
        .expect("Failed to convert Rust expression to JavaScript");

    // Convert single expression to code
    let module_items = vec![js::ModuleItem::Stmt(js::Stmt::Expr(js::ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(js_expr),
    }))];

    let code = ast_to_code_trimmed(&module_items)
        .expect("Failed to convert expression AST to JavaScript code");
    code
}

/// Generate JavaScript class for struct (old API)
pub fn generate_js_class_for_struct(input_struct: &ItemStruct) -> String {
    let module_item = generate_js_class_for_struct_with_state(input_struct)
        .expect("Failed to generate JavaScript class for struct");

    ast_to_code(&[module_item]).expect("Failed to convert struct AST to JavaScript code")
}

// Helper function to create enum toJSON method
fn create_enum_to_json_method(
    input_enum: &ItemEnum,
    state: &mut TranspilerState,
) -> Result<js::ClassMethod, String> {
    // Create switch expression that handles all enum variants
    let mut switch_cases = Vec::new();
    
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();
        
        match &variant.fields {
            Fields::Unit => {
                // Unit variants: return just the string
                let return_stmt = state.mk_return_stmt(Some(state.mk_str_lit(&variant_name)));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Unnamed(fields) => {
                // Tuple variants: return object with type and value0, value1, etc.
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for i in 0..fields.unnamed.len() {
                    let field_name = format!("value{}", i);
                    props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident_name(&field_name)),
                            value: Box::new(state.mk_member_expr(
                                js::Expr::This(js::ThisExpr { span: DUMMY_SP }),
                                &field_name
                            )),
                        },
                    ))));
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Named(fields) => {
                // Struct variants: return object with type and named fields
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for field in &fields.named {
                    if let Some(field_name) = &field.ident {
                        let field_name_str = field_name.to_string();
                        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                                value: Box::new(state.mk_member_expr(
                                    js::Expr::This(js::ThisExpr { span: DUMMY_SP }),
                                    &field_name_str
                                )),
                            },
                        ))));
                    }
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
        }
    }
    
    // Create switch statement: switch(this.type || this) { ... }
    let discriminant = js::Expr::Bin(js::BinExpr {
        span: DUMMY_SP,
        op: js::BinaryOp::LogicalOr,
        left: Box::new(state.mk_member_expr(
            js::Expr::This(js::ThisExpr { span: DUMMY_SP }),
            "type"
        )),
        right: Box::new(js::Expr::This(js::ThisExpr { span: DUMMY_SP })),
    });
    
    let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
        span: DUMMY_SP,
        discriminant: Box::new(discriminant),
        cases: switch_cases,
    });
    
    let method_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: vec![switch_stmt],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::ClassMethod {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident_name("toJSON")),
        function: Box::new(js::Function {
            params: vec![],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(method_body),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
            ctxt: SyntaxContext::empty(),
        }),
        kind: js::MethodKind::Method,
        is_static: false,
        accessibility: None,
        is_abstract: false,
        is_optional: false,
        is_override: false,
    })
}

// Helper function to create enum fromJSON function  
fn create_enum_from_json_function(
    input_enum: &ItemEnum,
    state: &mut TranspilerState,
) -> Result<js::Function, String> {
    // First parse the JSON string, then create switch cases for each variant
    let mut function_body_stmts = Vec::new();
    
    // Add try-catch block for JSON parsing
    let json_parse_stmt = js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
        span: DUMMY_SP,
        kind: js::VarDeclKind::Const,
        declare: false,
        decls: vec![js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: state.mk_ident("parsed"),
                type_ann: None,
            }),
            init: Some(Box::new(state.mk_call_expr(
                state.mk_member_expr(
                    js::Expr::Ident(state.mk_ident("JSON")),
                    "parse"
                ),
                vec![js::Expr::Ident(state.mk_ident("jsonString"))]
            ))),
            definite: false,
        }],
        ctxt: SyntaxContext::empty(),
    })));
    function_body_stmts.push(json_parse_stmt);
    
    // Create switch cases for each variant
    let mut switch_cases = Vec::new();
    
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();
        
        match &variant.fields {
            Fields::Unit => {
                // Unit variants: just return the string
                let return_stmt = state.mk_return_stmt(Some(state.mk_str_lit(&variant_name)));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Unnamed(fields) => {
                // Tuple variants: create object with type and value0, value1, etc.
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for i in 0..fields.unnamed.len() {
                    let field_name = format!("value{}", i);
                    props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident_name(&field_name)),
                            value: Box::new(state.mk_member_expr(
                                js::Expr::Ident(state.mk_ident("parsed")),
                                &field_name
                            )),
                        },
                    ))));
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Named(fields) => {
                // Struct variants: create object with type and named fields
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for field in &fields.named {
                    if let Some(field_name) = &field.ident {
                        let field_name_str = field_name.to_string();
                        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                                value: Box::new(state.mk_member_expr(
                                    js::Expr::Ident(state.mk_ident("parsed")),
                                    &field_name_str
                                )),
                            },
                        ))));
                    }
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
        }
    }
    
    // Create switch statement: switch(parsed.type || parsed) { ... }
    let discriminant = js::Expr::Bin(js::BinExpr {
        span: DUMMY_SP,
        op: js::BinaryOp::LogicalOr,
        left: Box::new(state.mk_member_expr(
            js::Expr::Ident(state.mk_ident("parsed")),
            "type"
        )),
        right: Box::new(js::Expr::Ident(state.mk_ident("parsed"))),
    });
    
    let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
        span: DUMMY_SP,
        discriminant: Box::new(discriminant),
        cases: switch_cases,
    });
    
    function_body_stmts.push(switch_stmt);
    
    let method_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: function_body_stmts,
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::Function {
        params: vec![state.pat_to_param(js::Pat::Ident(js::BindingIdent {
            id: state.mk_ident("jsonString"),
            type_ann: None,
        }))],
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(method_body),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    })
}

// Helper function to create enum toJSON standalone function
fn create_enum_to_json_standalone_function(
    input_enum: &ItemEnum,
    state: &mut TranspilerState,
) -> Result<js::Function, String> {
    // This function takes an enum instance and converts it to JSON
    let mut switch_cases = Vec::new();
    
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();
        
        match &variant.fields {
            Fields::Unit => {
                // Unit variants: return just the string
                let return_stmt = state.mk_return_stmt(Some(state.mk_str_lit(&variant_name)));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Unnamed(fields) => {
                // Tuple variants: return object with type and value0, value1, etc.
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for i in 0..fields.unnamed.len() {
                    let field_name = format!("value{}", i);
                    props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident_name(&field_name)),
                            value: Box::new(state.mk_member_expr(
                                js::Expr::Ident(state.mk_ident("enumValue")),
                                &field_name
                            )),
                        },
                    ))));
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Named(fields) => {
                // Struct variants: return object with type and named fields
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for field in &fields.named {
                    if let Some(field_name) = &field.ident {
                        let field_name_str = field_name.to_string();
                        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                                value: Box::new(state.mk_member_expr(
                                    js::Expr::Ident(state.mk_ident("enumValue")),
                                    &field_name_str
                                )),
                            },
                        ))));
                    }
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let return_stmt = state.mk_return_stmt(Some(obj));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
        }
    }
    
    // Create switch statement: switch(enumValue.type || enumValue) { ... }
    let discriminant = js::Expr::Bin(js::BinExpr {
        span: DUMMY_SP,
        op: js::BinaryOp::LogicalOr,
        left: Box::new(state.mk_member_expr(
            js::Expr::Ident(state.mk_ident("enumValue")),
            "type"
        )),
        right: Box::new(js::Expr::Ident(state.mk_ident("enumValue"))),
    });
    
    let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
        span: DUMMY_SP,
        discriminant: Box::new(discriminant),
        cases: switch_cases,
    });
    
    let method_body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: vec![switch_stmt],
        ctxt: SyntaxContext::empty(),
    };

    Ok(js::Function {
        params: vec![state.pat_to_param(js::Pat::Ident(js::BindingIdent {
            id: state.mk_ident("enumValue"),
            type_ann: None,
        }))],
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(method_body),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    })
}

// Helper function to create enum toJSON switch body (for use in both const and separate function)
fn create_enum_to_json_switch_body(
    input_enum: &ItemEnum,
    state: &mut TranspilerState,
) -> Result<js::BlockStmt, String> {
    let mut switch_cases = Vec::new();
    
    for variant in &input_enum.variants {
        let variant_name = variant.ident.to_string();
        
        match &variant.fields {
            Fields::Unit => {
                // Unit variants: return JSON.stringify of the string
                let json_stringify_call = state.mk_call_expr(
                    state.mk_member_expr(
                        js::Expr::Ident(state.mk_ident("JSON")),
                        "stringify"
                    ),
                    vec![state.mk_str_lit(&variant_name)]
                );
                let return_stmt = state.mk_return_stmt(Some(json_stringify_call));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Unnamed(fields) => {
                // Tuple variants: return JSON.stringify of object with type and value0, value1, etc.
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for i in 0..fields.unnamed.len() {
                    let field_name = format!("value{}", i);
                    props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident_name(&field_name)),
                            value: Box::new(state.mk_member_expr(
                                js::Expr::Ident(state.mk_ident("enumValue")),
                                &field_name
                            )),
                        },
                    ))));
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let json_stringify_call = state.mk_call_expr(
                    state.mk_member_expr(
                        js::Expr::Ident(state.mk_ident("JSON")),
                        "stringify"
                    ),
                    vec![obj]
                );
                let return_stmt = state.mk_return_stmt(Some(json_stringify_call));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
            Fields::Named(fields) => {
                // Struct variants: return JSON.stringify of object with type and named fields
                let mut props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident_name("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];
                
                for field in &fields.named {
                    if let Some(field_name) = &field.ident {
                        let field_name_str = field_name.to_string();
                        props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                            js::KeyValueProp {
                                key: js::PropName::Ident(state.mk_ident_name(&field_name_str)),
                                value: Box::new(state.mk_member_expr(
                                    js::Expr::Ident(state.mk_ident("enumValue")),
                                    &field_name_str
                                )),
                            },
                        ))));
                    }
                }
                
                let obj = js::Expr::Object(js::ObjectLit {
                    span: DUMMY_SP,
                    props,
                });
                
                let json_stringify_call = state.mk_call_expr(
                    state.mk_member_expr(
                        js::Expr::Ident(state.mk_ident("JSON")),
                        "stringify"
                    ),
                    vec![obj]
                );
                let return_stmt = state.mk_return_stmt(Some(json_stringify_call));
                switch_cases.push(js::SwitchCase {
                    span: DUMMY_SP,
                    test: Some(Box::new(state.mk_str_lit(&variant_name))),
                    cons: vec![return_stmt],
                });
            }
        }
    }
    
    // Create switch statement: switch(enumValue.type || enumValue) { ... }
    let discriminant = js::Expr::Bin(js::BinExpr {
        span: DUMMY_SP,
        op: js::BinaryOp::LogicalOr,
        left: Box::new(state.mk_member_expr(
            js::Expr::Ident(state.mk_ident("enumValue")),
            "type"
        )),
        right: Box::new(js::Expr::Ident(state.mk_ident("enumValue"))),
    });
    
    let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
        span: DUMMY_SP,
        discriminant: Box::new(discriminant),
        cases: switch_cases,
    });
    
    Ok(js::BlockStmt {
        span: DUMMY_SP,
        stmts: vec![switch_stmt],
        ctxt: SyntaxContext::empty(),
    })
}

/// Generate JavaScript enum (old API)

pub fn generate_js_enum(input_enum: &ItemEnum) -> String {
    let module_items =
        generate_js_enum_with_state(input_enum).expect("Failed to generate JavaScript enum");

    ast_to_code(&module_items).expect("Failed to convert enum AST to JavaScript code")
}
