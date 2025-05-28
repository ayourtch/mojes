use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;
use swc_ecma_codegen;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Block, Expr, Fields, ItemEnum, ItemStruct, Pat, Stmt, Type};
use syn::{FnArg, ImplItem, ItemImpl, ReturnType, Signature};

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
            _ => Err("Unsupported assignment target expression".to_string()),
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

    pub fn declare_variable(&mut self, rust_name: String, js_name: String, is_mutable: bool) {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.insert(rust_name.clone(), js_name.clone());
        }
        self.symbol_table.insert(rust_name, SymbolInfo {
            js_name,
            rust_type: "unknown".to_string(),
            is_mutable,
        });
    }

    pub fn resolve_variable(&self, rust_name: &str) -> Option<String> {
        // Check scopes from innermost to outermost
        for scope in self.scope_stack.iter().rev() {
            if let Some(js_name) = scope.get(rust_name) {
                return Some(js_name.clone());
            }
        }
        
        // Fallback to symbol table
        self.symbol_table.get(rust_name).map(|info| info.js_name.clone())
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
        js::Expr::Lit(js::Lit::Str(js::Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: None,
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

        self.mk_call_expr(js::Expr::Arrow(arrow_fn), vec![])
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
        let quasis: Vec<js::TplElement> = parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| js::TplElement {
                span: DUMMY_SP,
                tail: i == parts.len() - 1,
                cooked: Some(part.into()),
                raw: None,
            })
            .collect();

        js::Expr::Tpl(js::Tpl {
            span: DUMMY_SP,
            exprs: exprs.into_iter().map(Box::new).collect(),
            quasis,
        })
    }
}

/// Generate JavaScript methods for a Rust impl block
pub fn generate_js_methods_for_impl(input_impl: &ItemImpl) -> Result<Vec<js::ModuleItem>, String> {
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

    let mut js_items = Vec::new();

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

    // Extract non-self parameters
    let params: Vec<js::Pat> = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Receiver(_) => None, // Skip self
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        Some(js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&pat_ident.ident.to_string()),
                            type_ann: None,
                        }))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // Convert method body to JavaScript
    let body_stmts = rust_block_to_js(&method.block, state)?;
    let body = js::BlockStmt {
        span: DUMMY_SP,
        stmts: body_stmts,
        ctxt: SyntaxContext::empty(),
    };

    let function = js::Function {
        params,
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
        state.mk_member_expr(
            js::Expr::Ident(state.mk_ident(struct_name)),
            &method_name,
        )
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

/// Convert Rust block to JavaScript statements
pub fn rust_block_to_js(block: &Block, state: &mut TranspilerState) -> Result<Vec<js::Stmt>, String> {
    let mut js_stmts = Vec::new();
    
    state.enter_scope();

    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(local) => {
                let js_stmt = handle_local_statement(local, state)?;
                js_stmts.push(js_stmt);
            }
            Stmt::Expr(expr, semi) => {
                let js_expr = rust_expr_to_js(expr, state)?;
                
                if semi.is_some() {
                    // Expression with semicolon - treat as statement
                    js_stmts.push(state.mk_expr_stmt(js_expr));
                } else {
                    // Expression without semicolon - likely a return expression
                    js_stmts.push(state.mk_return_stmt(Some(js_expr)));
                }
            }
            Stmt::Macro(mac_stmt) => {
                let js_expr = handle_macro_expr(&mac_stmt.mac, state)?;
                js_stmts.push(state.mk_expr_stmt(js_expr));
            }
            _ => {
                state.add_warning(format!("Unsupported statement type: {:?}", stmt));
            }
        }
    }

    state.exit_scope();
    Ok(js_stmts)
}

/// Convert Rust expression to JavaScript expression
pub fn rust_expr_to_js(expr: &Expr, state: &mut TranspilerState) -> Result<js::Expr, String> {
    match expr {
        // Handle literals
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Ok(state.mk_str_lit(&s.value())),
            syn::Lit::Int(i) => {
                let value = i.base10_parse::<f64>()
                    .map_err(|e| format!("Failed to parse integer: {}", e))?;
                Ok(state.mk_num_lit(value))
            }
            syn::Lit::Float(f) => {
                let value = f.base10_parse::<f64>()
                    .map_err(|e| format!("Failed to parse float: {}", e))?;
                Ok(state.mk_num_lit(value))
            }
            syn::Lit::Bool(b) => Ok(state.mk_bool_lit(b.value())),
            syn::Lit::Char(c) => Ok(state.mk_str_lit(&c.value().to_string())),
            _ => Err("Unsupported literal type".to_string()),
        },
        
        // Handle paths (variables, constants)
        Expr::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
                let ident_str = last_segment.ident.to_string();
                
                match ident_str.as_str() {
                    "self" => Ok(state.mk_this_expr()),
                    "None" => Ok(state.mk_null_lit()),
                    "true" | "false" => Ok(js::Expr::Ident(state.mk_ident(&ident_str))),
                    _ => {
                        // Try to resolve variable
                        let js_name = state.resolve_variable(&ident_str)
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
            let left = rust_expr_to_js(&bin.left, state)?;
            let right = rust_expr_to_js(&bin.right, state)?;

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
                _ => return Err("Unsupported binary operator".to_string()),
            };

            Ok(state.mk_binary_expr(left, js_op, right))
        }

        // Handle unary operations
        Expr::Unary(unary) => {
            let operand = rust_expr_to_js(&unary.expr, state)?;
            let js_op = match &unary.op {
                syn::UnOp::Not(_) => js::UnaryOp::Bang,
                syn::UnOp::Neg(_) => js::UnaryOp::Minus,
                syn::UnOp::Deref(_) => return Ok(operand), // Dereference is no-op in JS
            };

            Ok(js::Expr::Unary(js::UnaryExpr {
                span: DUMMY_SP,
                op: js_op,
                arg: Box::new(operand),
            }))
        }

        // Handle method calls
        Expr::MethodCall(method_call) => {
            handle_method_call(method_call, state)
        }

        // Handle function calls
        Expr::Call(call) => {
            handle_function_call(call, state)
        }

        // Handle field access
        Expr::Field(field) => {
            let base = rust_expr_to_js(&field.base, state)?;
            let member_name = match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };

            Ok(state.mk_member_expr(base, &member_name))
        }

        // Handle if expressions
        Expr::If(if_expr) => {
            handle_if_expr(if_expr, state)
        }

        // Handle block expressions
        Expr::Block(block_expr) => {
            let stmts = rust_block_to_js(&block_expr.block, state)?;
            Ok(state.mk_iife(stmts))
        }

        // Handle array literals
        Expr::Array(array) => {
            let elements: Result<Vec<_>, _> = array
                .elems
                .iter()
                .map(|elem| rust_expr_to_js(elem, state))
                .collect();

            let js_elements: Vec<Option<js::ExprOrSpread>> = elements?
                .into_iter()
                .map(|expr| Some(js::ExprOrSpread {
                    spread: None,
                    expr: Box::new(expr),
                }))
                .collect();

            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: js_elements,
            }))
        }

        // Handle index expressions
        Expr::Index(index) => {
            let obj = rust_expr_to_js(&index.expr, state)?;
            let prop = rust_expr_to_js(&index.index, state)?;

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
            let left = rust_expr_to_js(&assign.left, state)?;
            let right = rust_expr_to_js(&assign.right, state)?;

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
                let js_expr = rust_expr_to_js(return_expr, state)?;
                // Return expressions in JavaScript are statements, not expressions
                // We'll handle this at the statement level
                Ok(js_expr)
            } else {
                Ok(state.mk_undefined())
            }
        }

        // Handle macro expressions
        Expr::Macro(macro_expr) => {
            handle_macro_expr(&macro_expr.mac, state)
        }

        _ => {
            state.add_warning(format!("Unsupported expression type: {:?}", expr));
            Ok(state.mk_str_lit("/* Unsupported expression */"))
        }
    }
}


// Continuation of lib.rs - Helper functions

/// Handle method calls
fn handle_method_call(
    method_call: &syn::ExprMethodCall,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let receiver = rust_expr_to_js(&method_call.receiver, state)?;
    let method_name = method_call.method.to_string();

    // Convert arguments
    let args: Result<Vec<_>, _> = method_call
        .args
        .iter()
        .map(|arg| rust_expr_to_js(arg, state))
        .collect();
    let js_args = args?;

    // Handle special method mappings
    match method_name.as_str() {
        "len" | "count" => {
            // .len() becomes .length property access
            Ok(state.mk_member_expr(receiver, "length"))
        }
        "clone" => {
            // .clone() is typically a no-op in JavaScript for primitives
            Ok(receiver)
        }
        "push" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "push"), js_args))
        }
        "pop" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "pop"), js_args))
        }
        "contains" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "includes"), js_args))
        }
        "to_string" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toString"), js_args))
        }
        "to_uppercase" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toUpperCase"), js_args))
        }
        "to_lowercase" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "toLowerCase"), js_args))
        }
        "trim" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "trim"), js_args))
        }
        "starts_with" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "startsWith"), js_args))
        }
        "ends_with" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "endsWith"), js_args))
        }
        "replace" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "replace"), js_args))
        }
        "split" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "split"), js_args))
        }
        "join" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "join"), js_args))
        }
        "map" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "map"), js_args))
        }
        "filter" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "filter"), js_args))
        }
        "find" => {
            Ok(state.mk_call_expr(state.mk_member_expr(receiver, "find"), js_args))
        }
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
            let null_check = state.mk_binary_expr(
                receiver.clone(),
                js::BinaryOp::NotEqEq,
                state.mk_null_lit(),
            );
            let undefined_check = state.mk_binary_expr(
                receiver,
                js::BinaryOp::NotEqEq,
                state.mk_undefined(),
            );
            Ok(state.mk_binary_expr(null_check, js::BinaryOp::LogicalAnd, undefined_check))
        }
        "is_none" => {
            // Option::is_none() -> value === null || value === undefined
            let null_check = state.mk_binary_expr(
                receiver.clone(),
                js::BinaryOp::EqEqEq,
                state.mk_null_lit(),
            );
            let undefined_check = state.mk_binary_expr(
                receiver,
                js::BinaryOp::EqEqEq,
                state.mk_undefined(),
            );
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
    // Convert arguments
    let args: Result<Vec<_>, _> = call
        .args
        .iter()
        .map(|arg| rust_expr_to_js(arg, state))
        .collect();
    let js_args = args?;

    // Handle the function being called
    match &*call.func {
        Expr::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
// Continuation of handle_function_call and other helper functions

                let func_name = last_segment.ident.to_string();
                
                match func_name.as_str() {
                    "println" | "print" => {
                        let console_log = state.mk_member_expr(
                            js::Expr::Ident(state.mk_ident("console")),
                            "log",
                        );
                        Ok(state.mk_call_expr(console_log, js_args))
                    }
                    "eprintln" | "eprint" => {
                        let console_error = state.mk_member_expr(
                            js::Expr::Ident(state.mk_ident("console")),
                            "error",
                        );
                        Ok(state.mk_call_expr(console_error, js_args))
                    }
                    "format" => {
                        // Handle format! macro as function call
                        handle_format_macro(&call.args, state)
                    }
                    "Some" => {
                        // Option::Some just returns the value in JavaScript
                        if js_args.len() == 1 {
                            Ok(js_args.into_iter().next().unwrap())
                        } else {
                            Err(format!("Some() expects exactly one argument, got {}", js_args.len()))
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
            let callee = rust_expr_to_js(&call.func, state)?;
            Ok(state.mk_call_expr(callee, js_args))
        }
    }
}

/// Handle if expressions
fn handle_if_expr(
    if_expr: &syn::ExprIf,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let test = rust_expr_to_js(&if_expr.cond, state)?;
    let consequent_stmts = rust_block_to_js(&if_expr.then_branch, state)?;
    
    let mut if_stmts = vec![
        js::Stmt::If(js::IfStmt {
            span: DUMMY_SP,
            test: Box::new(test),
            cons: Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: consequent_stmts,
                ctxt: SyntaxContext::empty(),
            })),
            alt: None,
        })
    ];

    // Handle else branch
    if let Some((_, else_branch)) = &if_expr.else_branch {
        let else_stmts = match &**else_branch {
            Expr::Block(else_block) => rust_block_to_js(&else_block.block, state)?,
            Expr::If(_) => {
                // Handle else if
                let else_if_expr = rust_expr_to_js(else_branch, state)?;
                vec![state.mk_expr_stmt(else_if_expr)]
            }
            _ => {
                let else_expr = rust_expr_to_js(else_branch, state)?;
                vec![state.mk_return_stmt(Some(else_expr))]
            }
        };

        // Update the if statement to include the else branch
        if let js::Stmt::If(ref mut if_stmt) = if_stmts[0] {
            if_stmt.alt = Some(Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: else_stmts,
                ctxt: SyntaxContext::empty(),
            })));
        }
    }

    // Add a default return undefined
    if_stmts.push(state.mk_return_stmt(Some(state.mk_undefined())));

    Ok(state.mk_iife(if_stmts))
}

/// Handle macro expressions
fn handle_macro_expr(
    mac: &syn::Macro,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let macro_name = if let Some(segment) = mac.path.segments.last() {
        segment.ident.to_string()
    } else {
        return Err("Invalid macro".to_string());
    };

    let tokens = mac.tokens.to_string();

    match macro_name.as_str() {
        "println" | "print" => {
            let console_method = if macro_name == "println" { "log" } else { "log" };
            let console_expr = state.mk_member_expr(
                js::Expr::Ident(state.mk_ident("console")),
                console_method,
            );

            if tokens.trim().is_empty() {
                Ok(state.mk_call_expr(console_expr, vec![]))
            } else if tokens.contains("{}") {
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
            let console_expr = state.mk_member_expr(
                js::Expr::Ident(state.mk_ident("console")),
                "error",
            );

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
        "format" => {
            handle_format_like_macro(&tokens, state)
        }
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
                    let array_from = state.mk_member_expr(
                        js::Expr::Ident(state.mk_ident("Array")),
                        "from",
                    );
                    
                    // Create {length: count} object
                    let length_obj = js::Expr::Object(js::ObjectLit {
                        span: DUMMY_SP,
                        props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident("length")),
                            value: Box::new(count_expr),
                        })))],
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
                    .map(|expr| Some(js::ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    }))
                    .collect();

                Ok(js::Expr::Array(js::ArrayLit {
                    span: DUMMY_SP,
                    elems: js_elements,
                }))
            }
        }
        _ => Err(format!("Unsupported macro: {}", macro_name)),
    }
}

/// Handle format-like macros (format!, println! with {}, etc.)
fn handle_format_like_macro(
    token_string: &str,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let parts = smart_comma_split(token_string);
    
    if parts.is_empty() {
        return Ok(state.mk_str_lit(""));
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
    if !format_str.contains("{}") {
        return Ok(state.mk_str_lit(format_str));
    }

    // Split format string at placeholders
    let str_parts: Vec<&str> = format_str.split("{}").collect();
    
    // Create template literal
    let mut template_parts = Vec::new();
    let mut template_exprs = Vec::new();
    
    for (i, part) in str_parts.iter().enumerate() {
        template_parts.push(part.to_string());
        
        if i < js_args.len() {
            template_exprs.push(js_args[i].clone());
        }
    }
    
    // Handle the case where we have more parts than expressions
    if template_parts.len() > template_exprs.len() + 1 {
        // Add empty expressions for missing placeholders
        while template_exprs.len() < template_parts.len() - 1 {
            template_exprs.push(state.mk_str_lit(""));
        }
    }

    Ok(state.mk_template_literal(template_parts, template_exprs))
}

/// Handle format! macro with parsed arguments
fn handle_format_macro(
    args: &Punctuated<Expr, Comma>,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    if args.is_empty() {
        return Ok(state.mk_str_lit(""));
    }

    // Get the format string
    if let Some(first_arg) = args.first() {
        if let Expr::Lit(lit) = first_arg {
            if let syn::Lit::Str(str_lit) = &lit.lit {
                let format_str = str_lit.value();

                if !format_str.contains("{}") {
                    return Ok(state.mk_str_lit(&format_str));
                }

                // Get format arguments
                let format_args: Result<Vec<_>, _> = args
                    .iter()
                    .skip(1)
                    .map(|arg| rust_expr_to_js(arg, state))
                    .collect();
                let js_args = format_args?;

                // Split format string at placeholders
                let parts: Vec<&str> = format_str.split("{}").collect();
                
                // Create template literal
                let template_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                
                return Ok(state.mk_template_literal(template_parts, js_args));
            }
        }
    }

    // Fallback: concatenate arguments
    let js_args: Result<Vec<_>, _> = args
        .iter()
        .map(|arg| rust_expr_to_js(arg, state))
        .collect();
    
    let args_vec = js_args?;
    if args_vec.len() == 1 {
        Ok(args_vec.into_iter().next().unwrap())
    } else {
        // Join with spaces - this is a simplification
        let joined = args_vec.into_iter().reduce(|acc, expr| {
            state.mk_binary_expr(acc, js::BinaryOp::Add, state.mk_str_lit(" "));
            state.mk_binary_expr(acc, js::BinaryOp::Add, expr)
        }).unwrap_or_else(|| state.mk_str_lit(""));
        
        Ok(joined)
    }
}

/// Handle local variable declarations
fn handle_local_statement(
    local: &syn::Local,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    if let Some(init) = &local.init {
        let init_expr = rust_expr_to_js(&init.expr, state)?;

        match &*local.pat {
            Pat::Ident(pat_ident) => {
                let var_name = pat_ident.ident.to_string();
                let js_var_name = escape_js_identifier(&var_name);
                let is_mutable = pat_ident.mutability.is_some();
                
                state.declare_variable(var_name, js_var_name.clone(), is_mutable);
                
                Ok(state.mk_var_decl(&js_var_name, Some(init_expr), !is_mutable))
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
            _ => Err("Unsupported destructuring pattern".to_string()),
        }
    } else {
        // Variable declaration without initialization
        if let Pat::Ident(pat_ident) = &*local.pat {
            let var_name = pat_ident.ident.to_string();
            let js_var_name = escape_js_identifier(&var_name);
            let is_mutable = pat_ident.mutability.is_some();
            
            state.declare_variable(var_name, js_var_name.clone(), is_mutable);
            
            Ok(state.mk_var_decl(&js_var_name, None, false)) // Always use let for uninitialized
        } else {
            Err("Unsupported variable pattern".to_string())
        }
    }
}

/// Parse macro tokens into a JavaScript expression
fn parse_macro_tokens(tokens: &str, state: &mut TranspilerState) -> Result<js::Expr, String> {
    let trimmed = tokens.trim();
    
    // Try to parse as a Rust expression first
    if let Ok(parsed_expr) = syn::parse_str::<syn::Expr>(trimmed) {
        rust_expr_to_js(&parsed_expr, state)
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
        "abstract", "arguments", "await", "boolean", "break", "byte", "case", "catch",
        "char", "class", "const", "continue", "debugger", "default", "delete", "do",
        "double", "else", "enum", "eval", "export", "extends", "false", "final",
        "finally", "float", "for", "function", "goto", "if", "implements", "import",
        "in", "instanceof", "int", "interface", "let", "long", "native", "new",
        "null", "package", "private", "protected", "public", "return", "short",
        "static", "super", "switch", "synchronized", "this", "throw", "throws",
        "transient", "true", "try", "typeof", "var", "void", "volatile", "while",
        "with", "yield",
    ];

    if JS_RESERVED.contains(&rust_ident) {
        format!("{}_", rust_ident)
    } else {
        rust_ident.to_string()
    }
}

/// Generate JavaScript class for a Rust struct
pub fn generate_js_class_for_struct(input_struct: &ItemStruct) -> Result<js::ModuleItem, String> {
    let mut state = TranspilerState::new();
    let struct_name = input_struct.ident.to_string();

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
            vec![("data".to_string(), "Array".to_string())]
        }
        Fields::Unit => vec![],
    };

    // Create constructor parameters
    let constructor_params: Vec<js::Pat> = fields
        .iter()
        .map(|(name, _)| {
            js::Pat::Ident(js::BindingIdent {
                id: state.mk_ident(name),
                type_ann: None,
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

    // Create constructor method
    let constructor = js::Constructor {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident("constructor")),
        params: constructor_params,
        body: Some(js::BlockStmt {
            span: DUMMY_SP,
            stmts: constructor_body,
            ctxt: SyntaxContext::empty(),
        }),
        accessibility: None,
        is_optional: false,
    };

    // Create class
    let class = js::Class {
        span: DUMMY_SP,
        decorators: vec![],
        body: vec![js::ClassMember::Constructor(constructor)],
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

/// Generate JavaScript enum
pub fn generate_js_enum(input_enum: &ItemEnum) -> Result<js::ModuleItem, String> {
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
                        key: js::PropName::Ident(state.mk_ident(&variant_name)),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                ))));
            }
            Fields::Unnamed(fields) | Fields::Named(fields) => {
                // Create factory function for complex variants
                let param_count = match &variant.fields {
                    Fields::Unnamed(f) => f.unnamed.len(),
                    Fields::Named(f) => f.named.len(),
                    _ => 0,
                };

                let params: Vec<js::Pat> = (0..param_count)
                    .map(|i| {
                        js::Pat::Ident(js::BindingIdent {
                            id: state.mk_ident(&format!("value{}", i)),
                            type_ann: None,
                        })
                    })
                    .collect();

                // Create function body that returns an object
                let mut obj_props = vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident("type")),
                        value: Box::new(state.mk_str_lit(&variant_name)),
                    },
                )))];

                for i in 0..param_count {
                    obj_props.push(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                        js::KeyValueProp {
                            key: js::PropName::Ident(state.mk_ident(&format!("value{}", i))),
                            value: Box::new(js::Expr::Ident(state.mk_ident(&format!("value{}", i)))),
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
                    params,
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
                        key: js::PropName::Ident(state.mk_ident(&variant_name)),
                        value: Box::new(js::Expr::Fn(js::FnExpr {
                            ident: None,
                            function: Box::new(function),
                        })),
                    },
                ))));
            }
        }
    }

    // Create the enum object
    let enum_obj = js::Expr::Object(js::ObjectLit {
        span: DUMMY_SP,
        props: properties,
    });

    // Create const declaration
    let var_decl = js::VarDecl {
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

    Ok(js::ModuleItem::Stmt(js::Stmt::Decl(js::Decl::Var(Box::new(
        var_decl,
    )))))
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

/// Convert JavaScript AST to code string
pub fn ast_to_code(module_items: &[js::ModuleItem]) -> Result<String, String> {
    let module = js::Module {
        span: DUMMY_SP,
        body: module_items.to_vec(),
        shebang: None,
    };

    match swc_ecma_codegen::to_code(&module) {
        Ok(code) => Ok(code),
        Err(e) => Err(format!("Failed to generate code: {:?}", e)),
    }
}

/// Convenience function to transpile a complete impl block to JavaScript code
pub fn transpile_impl_to_js(input_impl: &ItemImpl) -> Result<String, String> {
    let module_items = generate_js_methods_for_impl(input_impl)?;
    ast_to_code(&module_items)
}

/// Convenience function to transpile a struct to JavaScript code
pub fn transpile_struct_to_js(input_struct: &ItemStruct) -> Result<String, String> {
    let module_item = generate_js_class_for_struct(input_struct)?;
    ast_to_code(&[module_item])
}

/// Convenience function to transpile an enum to JavaScript code
pub fn transpile_enum_to_js(input_enum: &ItemEnum) -> Result<String, String> {
    let module_item = generate_js_enum(input_enum)?;
    ast_to_code(&[module_item])
}

/// Handle while loops
fn handle_while_expr(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let test = rust_expr_to_js(&while_expr.cond, state)?;
    let body_stmts = rust_block_to_js(&while_expr.body, state)?;

    let while_stmt = js::Stmt::While(js::WhileStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        body: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: body_stmts,
            ctxt: SyntaxContext::empty(),
        })),
    });

    Ok(state.mk_iife(vec![while_stmt]))
}

/// Handle for loops
fn handle_for_expr(
    for_expr: &syn::ExprForLoop,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    // Extract the loop variable
    let loop_var = if let Pat::Ident(pat_ident) = &*for_expr.pat {
        pat_ident.ident.to_string()
    } else {
        return Err("Unsupported for loop pattern".to_string());
    };

    let js_loop_var = escape_js_identifier(&loop_var);
    
    // Convert the iterable expression
    let iterable = rust_expr_to_js(&for_expr.expr, state)?;
    
    // Convert loop body
    let body_stmts = rust_block_to_js(&for_expr.body, state)?;

    // Create for...of loop
    let for_stmt = js::Stmt::ForOf(js::ForOfStmt {
        span: DUMMY_SP,
        is_await: false,
        left: js::VarDeclOrExpr::VarDecl(Box::new(js::VarDecl {
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
    });

    Ok(state.mk_iife(vec![for_stmt]))
}

/// Handle match expressions
fn handle_match_expr(
    match_expr: &syn::ExprMatch,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let match_value = rust_expr_to_js(&match_expr.expr, state)?;
    let temp_var = state.generate_temp_var();
    
    let mut stmts = vec![
        state.mk_var_decl(&temp_var, Some(match_value), true)
    ];

    let mut if_chain: Option<js::Stmt> = None;

    for (i, arm) in match_expr.arms.iter().enumerate() {
        let condition = create_match_condition(&arm.pat, &temp_var, state)?;
        let body_expr = rust_expr_to_js(&arm.body, state)?;
        let body_stmt = state.mk_return_stmt(Some(body_expr));

        let current_if = js::Stmt::If(js::IfStmt {
            span: DUMMY_SP,
            test: Box::new(condition),
            cons: Box::new(js::Stmt::Block(js::BlockStmt {
                span: DUMMY_SP,
                stmts: vec![body_stmt],
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
                    let value = i.base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse integer: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Float(f) => {
                    let value = f.base10_parse::<f64>()
                        .map_err(|e| format!("Failed to parse float: {}", e))?;
                    state.mk_num_lit(value)
                }
                syn::Lit::Bool(b) => state.mk_bool_lit(b.value()),
                syn::Lit::Char(c) => state.mk_str_lit(&c.value().to_string()),
                _ => return Err("Unsupported literal in match pattern".to_string()),
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
                        Ok(state.mk_binary_expr(null_check, js::BinaryOp::LogicalOr, undefined_check))
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
                    Err("Unsupported tuple struct pattern".to_string())
                }
            } else {
                Err("Invalid tuple struct pattern".to_string())
            }
        }
        _ => Err("Unsupported match pattern".to_string()),
    }
}

/// Helper function to chain if statements for match arms
fn chain_if_statement(current: &mut js::Stmt, next: js::Stmt) {
    if let js::Stmt::If(ref mut if_stmt) = current {
        if if_stmt.alt.is_none() {
            if_stmt.alt = Some(Box::new(next));
        } else {
            // Recursively chain
            if let Some(ref mut alt) = if_stmt.alt {
                chain_if_statement(alt, next);
            }
        }
    }
}

/// Handle loop expressions (infinite loops)
fn handle_loop_expr(
    loop_expr: &syn::ExprLoop,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let body_stmts = rust_block_to_js(&loop_expr.body, state)?;

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
        .filter_map(|param| {
            if let Pat::Ident(pat_ident) = param {
                Some(js::Pat::Ident(js::BindingIdent {
                    id: state.mk_ident(&pat_ident.ident.to_string()),
                    type_ann: None,
                }))
            } else {
                None
            }
        })
        .collect();

    // Handle closure body
    let body = match &*closure.body {
        Expr::Block(block_expr) => {
            let stmts = rust_block_to_js(&block_expr.block, state)?;
            js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
                span: DUMMY_SP,
                stmts,
                ctxt: SyntaxContext::empty(),
            })
        }
        _ => {
            let expr = rust_expr_to_js(&closure.body, state)?;
            js::BlockStmtOrExpr::Expr(Box::new(expr))
        }
    };

    Ok(js::Expr::Arrow(js::ArrowExpr {
        span: DUMMY_SP,
        params,
        body: Box::new(body),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
    }))
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

    // Convert field initializers to constructor arguments
    let args: Result<Vec<_>, _> = struct_expr
        .fields
        .iter()
        .map(|field| rust_expr_to_js(&field.expr, state))
        .collect();

    let js_args = args?;

    // Create new constructor call
    let constructor = js::Expr::Ident(state.mk_ident(&struct_name));
    
    Ok(js::Expr::New(js::NewExpr {
        span: DUMMY_SP,
        callee: Box::new(constructor),
        args: Some(js_args.into_iter().map(|expr| js::ExprOrSpread {
            spread: None,
            expr: Box::new(expr),
        }).collect()),
        type_args: None,
        ctxt: SyntaxContext::empty(),
    }))
}

/// Handle range expressions
fn handle_range_expr(
    range_expr: &syn::ExprRange,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    match (&range_expr.start, &range_expr.end) {
        (Some(start), Some(end)) => {
            let start_js = rust_expr_to_js(start, state)?;
            let end_js = rust_expr_to_js(end, state)?;

            // Create Array.from({length: end - start}, (_, i) => i + start)
            let array_from = state.mk_member_expr(
                js::Expr::Ident(state.mk_ident("Array")),
                "from",
            );

            let length_expr = state.mk_binary_expr(
                end_js.clone(),
                js::BinaryOp::Sub,
                start_js.clone(),
            );

            let length_obj = js::Expr::Object(js::ObjectLit {
                span: DUMMY_SP,
                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident("length")),
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
            let end_js = rust_expr_to_js(end, state)?;
            let array_from = state.mk_member_expr(
                js::Expr::Ident(state.mk_ident("Array")),
                "from",
            );

            let length_obj = js::Expr::Object(js::ObjectLit {
                span: DUMMY_SP,
                props: vec![js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(
                    js::KeyValueProp {
                        key: js::PropName::Ident(state.mk_ident("length")),
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
        _ => {
            // Other range types not easily representable
            state.add_warning("Infinite or complex ranges not fully supported".to_string());
            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: vec![],
            }))
        }
    }
}

/// Complete the rust_expr_to_js function with remaining expression types
pub fn rust_expr_to_js_complete(expr: &Expr, state: &mut TranspilerState) -> Result<js::Expr, String> {
    match expr {
        // Handle the patterns we already covered in the main function
        Expr::Lit(_) | Expr::Path(_) | Expr::Binary(_) | Expr::Unary(_) |
        Expr::MethodCall(_) | Expr::Call(_) | Expr::Field(_) | Expr::If(_) |
        Expr::Block(_) | Expr::Array(_) | Expr::Index(_) | Expr::Assign(_) |
        Expr::Return(_) | Expr::Macro(_) => {
            rust_expr_to_js(expr, state)
        }

        // Handle additional expression types
        Expr::While(while_expr) => handle_while_expr(while_expr, state),
        Expr::ForLoop(for_expr) => handle_for_expr(for_expr, state),
        Expr::Match(match_expr) => handle_match_expr(match_expr, state),
        Expr::Loop(loop_expr) => handle_loop_expr(loop_expr, state),
        Expr::Closure(closure) => handle_closure_expr(closure, state),
        Expr::Struct(struct_expr) => handle_struct_expr(struct_expr, state),
        Expr::Range(range_expr) => handle_range_expr(range_expr, state),

        Expr::Paren(paren) => {
            // Parenthesized expressions
            let inner = rust_expr_to_js(&paren.expr, state)?;
            Ok(js::Expr::Paren(js::ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(inner),
            }))
        }

        Expr::Break(break_expr) => {
            if let Some(value) = &break_expr.expr {
                let value_js = rust_expr_to_js(value, state)?;
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
                .map(|elem| rust_expr_to_js(elem, state))
                .collect();

            let js_elements: Vec<Option<js::ExprOrSpread>> = elements?
                .into_iter()
                .map(|expr| Some(js::ExprOrSpread {
                    spread: None,
                    expr: Box::new(expr),
                }))
                .collect();

            Ok(js::Expr::Array(js::ArrayLit {
                span: DUMMY_SP,
                elems: js_elements,
            }))
        }

        _ => {
            state.add_warning(format!("Unsupported expression type: {:?}", expr));
            Ok(state.mk_str_lit("/* Unsupported expression */"))
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

