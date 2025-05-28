use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;
use swc_ecma_codegen;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Block, Expr, Fields, ItemEnum, ItemStruct, Pat, Stmt, Type};
// use syn::{FnArg, ImplItem, ItemImpl, ReturnType, Signature};
use syn::{FnArg, ImplItem, ItemImpl};

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

    pub fn declare_variable(&mut self, rust_name: String, js_name: String, is_mutable: bool) {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.insert(rust_name.clone(), js_name.clone());
        }
        self.symbol_table.insert(
            rust_name,
            SymbolInfo {
                js_name,
                rust_type: "unknown".to_string(),
                is_mutable,
            },
        );
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
    let body_stmts = rust_block_to_js_with_state(&method.block, state)?;
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

/// Convert Rust block to JavaScript statements
pub fn rust_block_to_js_with_state(
    block: &Block,
    state: &mut TranspilerState,
) -> Result<Vec<js::Stmt>, String> {
    let mut js_stmts = Vec::new();

    state.enter_scope();

    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(local) => {
                let js_stmt = handle_local_statement(local, state)?;
                js_stmts.push(js_stmt);
            }
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
                    _ => {
                        let js_expr = rust_expr_to_js_with_state(expr, state)?;

                        if semi.is_some() {
                            // Expression with semicolon - treat as statement
                            js_stmts.push(state.mk_expr_stmt(js_expr));
                        } else {
                            // Expression without semicolon - likely a return expression
                            js_stmts.push(state.mk_return_stmt(Some(js_expr)));
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

/// Convert Rust expression to JavaScript expression
pub fn rust_expr_to_js_with_state(
    expr: &Expr,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
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
            let member_name = match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };

            Ok(state.mk_member_expr(base, &member_name))
        }

        // Handle if expressions
        Expr::If(if_expr) => handle_if_expr(if_expr, state),

        // Handle block expressions
        Expr::Block(block_expr) => {
            let stmts = rust_block_to_js_with_state(&block_expr.block, state)?;
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

        _ => {
            panic!("Unsupported expression type: {:?}", expr);
        }
    }
}

/// Handle async expressions
fn handle_async_expr(
    async_expr: &syn::ExprAsync,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let body_stmts = rust_block_to_js_with_state(&async_expr.block, state)?;

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
            // .len() becomes .length property access
            Ok(state.mk_member_expr(receiver, "length"))
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
            // vec.insert(index, item) -> vec.splice(index, 0, item)
            if js_args.len() == 2 {
                Ok(state.mk_call_expr(
                    state.mk_member_expr(receiver, "splice"),
                    vec![
                        js_args[0].clone(),
                        state.mk_num_lit(0.0),
                        js_args[1].clone(),
                    ],
                ))
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
    // Convert arguments
    let args: Result<Vec<_>, _> = call
        .args
        .iter()
        .map(|arg| rust_expr_to_js_with_state(arg, state))
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
                        let console_log =
                            state.mk_member_expr(js::Expr::Ident(state.mk_ident("console")), "log");
                        Ok(state.mk_call_expr(console_log, js_args))
                    }
                    "eprintln" | "eprint" => {
                        let console_error = state
                            .mk_member_expr(js::Expr::Ident(state.mk_ident("console")), "error");
                        Ok(state.mk_call_expr(console_error, js_args))
                    }
                    "format" => {
                        // Handle format! macro as function call
                        panic!("NEVER CALLED?");
                        handle_format_macro_with_state(&call.args, state)
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

/// Handle if expressions
fn handle_if_expr(if_expr: &syn::ExprIf, state: &mut TranspilerState) -> Result<js::Expr, String> {
    let test = rust_expr_to_js_with_state(&if_expr.cond, state)?;
    let consequent_stmts = rust_block_to_js_with_state(&if_expr.then_branch, state)?;

    let mut if_stmts = vec![js::Stmt::If(js::IfStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(js::Stmt::Block(js::BlockStmt {
            span: DUMMY_SP,
            stmts: consequent_stmts,
            ctxt: SyntaxContext::empty(),
        })),
        alt: None,
    })];

    // Handle else branch
    if let Some((_, else_branch)) = &if_expr.else_branch {
        let else_stmts = match &**else_branch {
            Expr::Block(else_block) => rust_block_to_js_with_state(&else_block.block, state)?,
            Expr::If(_) => {
                // Handle else if
                let else_if_expr = rust_expr_to_js_with_state(else_branch, state)?;
                vec![state.mk_expr_stmt(else_if_expr)]
            }
            _ => {
                let else_expr = rust_expr_to_js_with_state(else_branch, state)?;
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
fn handle_macro_expr(mac: &syn::Macro, state: &mut TranspilerState) -> Result<js::Expr, String> {
    let macro_name = if let Some(segment) = mac.path.segments.last() {
        segment.ident.to_string()
    } else {
        return Err("Invalid macro".to_string());
    };

    let tokens = mac.tokens.to_string();

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
    if !format_str.contains("{}") {
        return Ok(state.mk_template_literal(vec![format_str.into()], vec![]));
        // this will return just the quoted format string.
        // return Ok(state.mk_str_lit(format_str));
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
fn handle_format_macro_with_state(
    args: &Punctuated<Expr, Comma>,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
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
    local: &syn::Local,
    state: &mut TranspilerState,
) -> Result<js::Stmt, String> {
    if let Some(init) = &local.init {
        let init_expr = rust_expr_to_js_with_state(&init.expr, state)?;

        match &local.pat {
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
                            state.declare_variable(name.clone(), js_name.clone(), false);
                            // Use shorthand property syntax for destructuring
                            js::ObjectPatProp::Assign(js::AssignPatProp {
                                span: DUMMY_SP,
                                key: js::BindingIdent {
                                    id: state.mk_ident(&js_name),
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

            state.declare_variable(var_name, js_var_name.clone(), is_mutable);

            Ok(state.mk_var_decl(&js_var_name, None, false)) // Always use let for uninitialized
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

    let constructor_params_converted: Vec<js::ParamOrTsParamProp> = constructor_params
        .into_iter()
        .map(|pat| {
            js::ParamOrTsParamProp::TsParamProp(js::TsParamProp {
                span: DUMMY_SP,
                decorators: vec![],
                accessibility: None,
                readonly: false,
                is_override: false,
                param: js::TsParamPropParam::Ident(match pat {
                    js::Pat::Ident(binding_ident) => binding_ident,
                    _ => js::BindingIdent {
                        id: state.mk_ident("param"),
                        type_ann: None,
                    },
                }),
            })
        })
        .collect();

    let constructor = js::Constructor {
        span: DUMMY_SP,
        key: js::PropName::Ident(state.mk_ident_name("constructor")),
        params: constructor_params_converted,
        body: Some(js::BlockStmt {
            span: DUMMY_SP,
            stmts: constructor_body,
            ctxt: SyntaxContext::empty(),
        }),
        accessibility: None,
        is_optional: false,
        ctxt: SyntaxContext::empty(),
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

/// Handle while loops
fn handle_while_expr(
    while_expr: &syn::ExprWhile,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    let test = rust_expr_to_js_with_state(&while_expr.cond, state)?;
    let body_stmts = rust_block_to_js_with_state(&while_expr.body, state)?;

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
        panic!("Unsupported for loop pattern {:?}", &*for_expr.pat);
    };

    let js_loop_var = escape_js_identifier(&loop_var);

    // Convert the iterable expression
    let iterable = rust_expr_to_js_with_state(&for_expr.expr, state)?;

    // Convert loop body
    let body_stmts = rust_block_to_js_with_state(&for_expr.body, state)?;

    // Create for...of loop
    let for_stmt = js::Stmt::ForOf(js::ForOfStmt {
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
    });

    Ok(state.mk_iife(vec![for_stmt]))
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
                    panic!("Unsupported tuple struct ident {:?}", &segment);
                }
            } else {
                panic!(
                    "Unsupported tuple struct last segment {:?}",
                    &tuple_struct.path.segments
                );
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
        let body_expr = rust_expr_to_js_with_state(&arm.body, state)?;

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
    let body_stmts = rust_block_to_js_with_state(&loop_expr.body, state)?;

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
            let stmts = rust_block_to_js_with_state(&block_expr.block, state)?;
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

/// Complete the rust_expr_to_js_with_state function with remaining expression types
pub fn rust_expr_to_js_with_state_complete(
    expr: &Expr,
    state: &mut TranspilerState,
) -> Result<js::Expr, String> {
    match expr {
        // Handle the patterns we already covered in the main function
        Expr::Lit(_)
        | Expr::Path(_)
        | Expr::Binary(_)
        | Expr::Unary(_)
        | Expr::MethodCall(_)
        | Expr::Call(_)
        | Expr::Field(_)
        | Expr::If(_)
        | Expr::Block(_)
        | Expr::Array(_)
        | Expr::Index(_)
        | Expr::Assign(_)
        | Expr::Return(_)
        | Expr::Macro(_) => rust_expr_to_js_with_state(expr, state),

        // Handle additional expression types
        Expr::While(while_expr) => handle_while_expr(while_expr, state),
        Expr::ForLoop(for_expr) => handle_for_expr(for_expr, state),
        Expr::Match(match_expr) => handle_match_expr(match_expr, state),
        Expr::Loop(loop_expr) => handle_loop_expr(loop_expr, state),
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

        _ => {
            state.add_warning(format!("Unsupported expression type: {:?}", expr));
            panic!("Unsupported expression type: {:?}", expr);
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

    let stmts = rust_block_to_js_with_state(block, &mut state)
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

    let js_expr = rust_expr_to_js_with_state(expr, &mut state)
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

/// Generate JavaScript enum (old API)

pub fn generate_js_enum(input_enum: &ItemEnum) -> String {
    let module_items =
        generate_js_enum_with_state(input_enum).expect("Failed to generate JavaScript enum");

    ast_to_code(&module_items).expect("Failed to convert enum AST to JavaScript code")
}
