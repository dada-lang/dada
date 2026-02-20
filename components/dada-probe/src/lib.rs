use std::ops::ControlFlow;

use dada_ir_ast::{
    ast::{
        AstExpr, AstExprKind, AstItem, AstMember, AstPathKind, AstStatement, LiteralKind,
        PermissionOp, SpannedIdentifier, UnaryOp,
    },
    span::{AbsoluteSpan, SourceSpanned, Spanned},
};
pub use dada_ir_sym::Db;
use dada_ir_sym::{
    ir::{
        exprs::{SymExpr, SymExprKind},
        functions::SymFunction,
        module::SymItem,
    },
    prelude::{CheckedBody, Symbol},
};
use dada_parser::prelude::{ClassItemMembers, FunctionBlock, SourceFileParse};

/// Probe for the type of an expression found in a given file at a given span.
/// Returns the type of the smallest expression that contains the given span.
pub fn probe_expression_type<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<String> {
    // First find the smallest expression containing the span
    let expr = find_smallest_containing_expr(db, span)?;

    // Return its type as a string
    Some(expr.ty(db).to_string())
}

/// Helper function to find the smallest expression that contains the given span
fn find_smallest_containing_expr<'db>(
    db: &'db dyn crate::Db,
    span: AbsoluteSpan,
) -> Option<SymExpr<'db>> {
    let mut result = None;
    let mut smallest_size = usize::MAX;

    visit_exprs(db, span, &mut |expr| {
        let expr_span = expr.source_span(db).absolute_span(db);
        if expr_span.contains(span) {
            let size = expr_span.end.as_usize() - expr_span.start.as_usize();
            if size < smallest_size {
                result = Some(expr);
                smallest_size = size;
            }
        }
        ControlFlow::<()>::Continue(())
    });

    result
}

/// Probe for the type of a variable found in a given file at a given span.
pub fn probe_variable_type<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<String> {
    // We expect `span` to be located in
    visit_exprs(db, span, &mut |expr| {
        if let SymExprKind::LetIn {
            lv,
            ty,
            initializer: _,
            body: _,
        } = expr.kind(db)
            && lv.span(db).absolute_span(db).contains(span)
        {
            ControlFlow::Break(ty.to_string())
        } else {
            ControlFlow::Continue(())
        }
    })
}

/// Probe for the compact AST representation of the expression at a given span.
///
/// # Example
/// ```dada
/// print("hello").await
/// #? ^^^^^^^ Ast: Literal(String, "hello")
/// ```
///
/// Unlike `probe_expression_type` and `probe_variable_type`, this operates on
/// the parser AST (AstExpr) rather than the type-checked IR (SymExpr), so it
/// doesn't require type-checking to succeed.
pub fn probe_ast<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<String> {
    let expr = find_smallest_containing_ast_expr(db, span)?;
    Some(compact_ast_format(db, &expr))
}

// ---- SymExpr helpers (existing) ----

/// Find the module item containing `span`
fn find_item<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<SymItem<'db>> {
    let module = span.source_file.symbol(db);
    module
        .items(db)
        .find(|item| item.source_span(db).absolute_span(db).contains(span))
}

/// Find the fn or method containing `span`
fn find_func<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<SymFunction<'db>> {
    match find_item(db, span)? {
        SymItem::SymClass(aggr) => aggr
            .methods(db)
            .find(|m| m.source_span(db).absolute_span(db).contains(span)),
        SymItem::SymFunction(func) => Some(func),
        SymItem::SymPrimitive(_) => None,
    }
}

/// Walk all expressions containing the given `span` and invoke `op`.
/// Stops if `op` returns `ControlFlow::Break`.
fn visit_exprs<'db, B>(
    db: &'db dyn crate::Db,
    span: AbsoluteSpan,
    op: &mut dyn FnMut(SymExpr<'db>) -> ControlFlow<B>,
) -> Option<B> {
    let func = find_func(db, span)?;
    let expr = func.checked_body(db)?;
    walk_expr_and_visit(db, expr, span, op)
}

fn walk_expr_and_visit<'db, B>(
    db: &'db dyn crate::Db,
    expr: SymExpr<'db>,
    span: AbsoluteSpan,
    op: &mut dyn FnMut(SymExpr<'db>) -> ControlFlow<B>,
) -> Option<B> {
    if !expr.source_span(db).absolute_span(db).contains(span) {
        return None;
    }

    match op(expr) {
        ControlFlow::Continue(()) => {}
        ControlFlow::Break(b) => return Some(b),
    }

    match expr.kind(db) {
        SymExprKind::Semi(e1, e2) => walk_expr_and_visit(db, *e1, span, op)
            .or_else(|| walk_expr_and_visit(db, *e2, span, op)),
        SymExprKind::Tuple(exprs) => {
            for &expr in exprs {
                if let Some(b) = walk_expr_and_visit(db, expr, span, op) {
                    return Some(b);
                }
            }
            None
        }
        SymExprKind::Primitive(_) => None,
        SymExprKind::ByteLiteral(_) => None,
        SymExprKind::LetIn {
            lv: _,
            ty: _,
            initializer,
            body,
        } => initializer
            .and_then(|initializer| walk_expr_and_visit(db, initializer, span, op))
            .or_else(|| walk_expr_and_visit(db, *body, span, op)),
        SymExprKind::Await {
            future,
            await_keyword: _,
        } => walk_expr_and_visit(db, *future, span, op),
        SymExprKind::Assign { place: _, value } => walk_expr_and_visit(db, *value, span, op),
        SymExprKind::PermissionOp(_, _) => None,
        SymExprKind::Call {
            function: _,
            substitution: _,
            arg_temps: _,
        } => None,
        SymExprKind::Return(sym_expr) => walk_expr_and_visit(db, *sym_expr, span, op),
        SymExprKind::Not {
            operand,
            op_span: _,
        } => walk_expr_and_visit(db, *operand, span, op),
        SymExprKind::BinaryOp(_, lhs, rhs) => walk_expr_and_visit(db, *lhs, span, op)
            .or_else(|| walk_expr_and_visit(db, *rhs, span, op)),
        SymExprKind::Aggregate { ty: _, fields } => {
            for &field in fields {
                if let Some(b) = walk_expr_and_visit(db, field, span, op) {
                    return Some(b);
                }
            }
            None
        }
        SymExprKind::Match { arms } => {
            for arm in arms {
                if let Some(b) = walk_expr_and_visit(db, arm.body, span, op) {
                    return Some(b);
                }
            }
            None
        }
        SymExprKind::Error(_) => None,
    }
}

// ---- AST probe: expression finder ----

/// Find the smallest AstExpr containing the target span by walking the parsed AST.
///
/// 💡 This walks the parser AST directly rather than the type-checked SymExpr IR,
/// allowing AST probes to work without requiring successful type-checking.
fn find_smallest_containing_ast_expr<'db>(
    db: &'db dyn crate::Db,
    target: AbsoluteSpan,
) -> Option<AstExpr<'db>> {
    let module = target.source_file.parse(db);
    let mut best: Option<AstExpr<'db>> = None;
    let mut best_size = usize::MAX;

    for item in &module.items(db).values {
        match item {
            AstItem::Function(func) => {
                if let Some(block) = func.body_block(db) {
                    for stmt in &block.statements(db).values {
                        walk_ast_statement(db, stmt, target, &mut best, &mut best_size);
                    }
                }
            }
            AstItem::MainFunction(main_fn) => {
                for stmt in &main_fn.statements(db).values {
                    walk_ast_statement(db, stmt, target, &mut best, &mut best_size);
                }
            }
            AstItem::Aggregate(aggr) => {
                for member in &aggr.members(db).values {
                    if let AstMember::Function(func) = member
                        && let Some(block) = func.body_block(db)
                    {
                        for stmt in &block.statements(db).values {
                            walk_ast_statement(db, stmt, target, &mut best, &mut best_size);
                        }
                    }
                }
            }
            AstItem::SourceFile(_) | AstItem::Use(_) => {}
        }
    }

    best
}

fn walk_ast_statement<'db>(
    db: &'db dyn crate::Db,
    stmt: &AstStatement<'db>,
    target: AbsoluteSpan,
    best: &mut Option<AstExpr<'db>>,
    best_size: &mut usize,
) {
    match stmt {
        AstStatement::Let(let_stmt) => {
            if let Some(init) = let_stmt.initializer(db) {
                walk_ast_expr(db, &init, target, best, best_size);
            }
        }
        AstStatement::Expr(expr) => {
            walk_ast_expr(db, expr, target, best, best_size);
        }
    }
}

fn walk_ast_expr<'db>(
    db: &'db dyn crate::Db,
    expr: &AstExpr<'db>,
    target: AbsoluteSpan,
    best: &mut Option<AstExpr<'db>>,
    best_size: &mut usize,
) {
    let expr_abs = expr.span.absolute_span(db);
    if !expr_abs.contains(target) {
        return;
    }

    let size = expr_abs.end.as_usize() - expr_abs.start.as_usize();
    if size < *best_size {
        *best = Some(expr.clone());
        *best_size = size;
    }

    // Recurse into children
    match &*expr.kind {
        AstExprKind::Literal(_) | AstExprKind::Id(_) => {}
        AstExprKind::Block(block) => {
            for stmt in &block.statements(db).values {
                walk_ast_statement(db, stmt, target, best, best_size);
            }
        }
        AstExprKind::DotId(sub_expr, _) => {
            walk_ast_expr(db, sub_expr, target, best, best_size);
        }
        AstExprKind::SquareBracketOp(sub_expr, _) => {
            walk_ast_expr(db, sub_expr, target, best, best_size);
        }
        AstExprKind::ParenthesisOp(callee, args) => {
            walk_ast_expr(db, callee, target, best, best_size);
            for arg in &args.values {
                walk_ast_expr(db, arg, target, best, best_size);
            }
        }
        AstExprKind::Tuple(elems) => {
            for elem in &elems.values {
                walk_ast_expr(db, elem, target, best, best_size);
            }
        }
        AstExprKind::Constructor(_, fields) => {
            for field in &fields.values {
                walk_ast_expr(db, &field.value, target, best, best_size);
            }
        }
        AstExprKind::Return(opt_expr) => {
            if let Some(sub_expr) = opt_expr {
                walk_ast_expr(db, sub_expr, target, best, best_size);
            }
        }
        AstExprKind::Await { future, .. } => {
            walk_ast_expr(db, future, target, best, best_size);
        }
        AstExprKind::PermissionOp { value, .. } => {
            walk_ast_expr(db, value, target, best, best_size);
        }
        AstExprKind::BinaryOp(_, lhs, rhs) => {
            walk_ast_expr(db, lhs, target, best, best_size);
            walk_ast_expr(db, rhs, target, best, best_size);
        }
        AstExprKind::UnaryOp(_, sub_expr) => {
            walk_ast_expr(db, sub_expr, target, best, best_size);
        }
        AstExprKind::If(arms) => {
            for arm in arms {
                if let Some(cond) = &arm.condition {
                    walk_ast_expr(db, cond, target, best, best_size);
                }
                for stmt in &arm.result.statements(db).values {
                    walk_ast_statement(db, stmt, target, best, best_size);
                }
            }
        }
    }
}

// ---- AST probe: compact formatter ----

/// Format an AstExpr as a compact single-line string.
///
/// # Example outputs
/// - `Literal(String, "hello\nworld")`
/// - `ParenthesisOp(Id(print), [Literal(String, "hello")])`
/// - `Await(ParenthesisOp(Id(print), [Literal(String, "hello")]))`
fn compact_ast_format<'db>(db: &'db dyn crate::Db, expr: &AstExpr<'db>) -> String {
    let mut buf = String::new();
    format_ast_expr(db, expr, &mut buf);
    buf
}

fn format_ast_expr<'db>(db: &'db dyn crate::Db, expr: &AstExpr<'db>, buf: &mut String) {
    match &*expr.kind {
        AstExprKind::Literal(lit) => {
            let kind = match lit.kind(db) {
                LiteralKind::Boolean => "Boolean",
                LiteralKind::Integer => "Integer",
                LiteralKind::String => "String",
            };
            buf.push_str("Literal(");
            buf.push_str(kind);
            buf.push_str(", \"");
            escape_string_into(lit.text(db), buf);
            buf.push_str("\")");
        }
        AstExprKind::Id(spanned_id) => {
            buf.push_str("Id(");
            format_identifier(db, spanned_id, buf);
            buf.push(')');
        }
        AstExprKind::Block(block) => {
            buf.push_str("Block([");
            for (i, stmt) in block.statements(db).values.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                format_ast_statement(db, stmt, buf);
            }
            buf.push_str("])");
        }
        AstExprKind::DotId(sub_expr, spanned_id) => {
            buf.push_str("DotId(");
            format_ast_expr(db, sub_expr, buf);
            buf.push_str(", ");
            format_identifier(db, spanned_id, buf);
            buf.push(')');
        }
        AstExprKind::SquareBracketOp(sub_expr, _) => {
            buf.push_str("SquareBracketOp(");
            format_ast_expr(db, sub_expr, buf);
            buf.push(')');
        }
        AstExprKind::ParenthesisOp(callee, args) => {
            buf.push_str("ParenthesisOp(");
            format_ast_expr(db, callee, buf);
            buf.push_str(", [");
            for (i, arg) in args.values.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                format_ast_expr(db, arg, buf);
            }
            buf.push_str("])");
        }
        AstExprKind::Tuple(elems) => {
            buf.push_str("Tuple([");
            for (i, elem) in elems.values.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                format_ast_expr(db, elem, buf);
            }
            buf.push_str("])");
        }
        AstExprKind::Constructor(path, fields) => {
            buf.push_str("Constructor(");
            format_ast_path(db, path, buf);
            buf.push_str(", [");
            for (i, field) in fields.values.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                format_identifier(db, &field.name, buf);
                buf.push_str(": ");
                format_ast_expr(db, &field.value, buf);
            }
            buf.push_str("])");
        }
        AstExprKind::Return(opt_expr) => {
            buf.push_str("Return");
            if let Some(sub_expr) = opt_expr {
                buf.push('(');
                format_ast_expr(db, sub_expr, buf);
                buf.push(')');
            }
        }
        AstExprKind::Await { future, .. } => {
            buf.push_str("Await(");
            format_ast_expr(db, future, buf);
            buf.push(')');
        }
        AstExprKind::PermissionOp { value, op } => {
            let op_str = match op {
                PermissionOp::Mutate => "Mutate",
                PermissionOp::Reference => "Reference",
                PermissionOp::Give => "Give",
                PermissionOp::Share => "Share",
            };
            buf.push_str("PermissionOp(");
            buf.push_str(op_str);
            buf.push_str(", ");
            format_ast_expr(db, value, buf);
            buf.push(')');
        }
        AstExprKind::BinaryOp(spanned_op, lhs, rhs) => {
            buf.push_str("BinaryOp(");
            buf.push_str(&spanned_op.op.to_string());
            buf.push_str(", ");
            format_ast_expr(db, lhs, buf);
            buf.push_str(", ");
            format_ast_expr(db, rhs, buf);
            buf.push(')');
        }
        AstExprKind::UnaryOp(spanned_op, sub_expr) => {
            let op_str = match spanned_op.op {
                UnaryOp::Not => "!",
                UnaryOp::Negate => "-",
            };
            buf.push_str("UnaryOp(");
            buf.push_str(op_str);
            buf.push_str(", ");
            format_ast_expr(db, sub_expr, buf);
            buf.push(')');
        }
        AstExprKind::If(arms) => {
            buf.push_str("If([");
            for (i, arm) in arms.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                if let Some(cond) = &arm.condition {
                    format_ast_expr(db, cond, buf);
                    buf.push_str(" => ");
                } else {
                    buf.push_str("else => ");
                }
                buf.push_str("Block([");
                for (j, stmt) in arm.result.statements(db).values.iter().enumerate() {
                    if j > 0 {
                        buf.push_str(", ");
                    }
                    format_ast_statement(db, stmt, buf);
                }
                buf.push_str("])");
            }
            buf.push_str("])");
        }
    }
}

fn format_ast_statement<'db>(db: &'db dyn crate::Db, stmt: &AstStatement<'db>, buf: &mut String) {
    match stmt {
        AstStatement::Let(let_stmt) => {
            buf.push_str("Let(");
            format_identifier(db, &let_stmt.name(db), buf);
            if let Some(init) = let_stmt.initializer(db) {
                buf.push_str(", ");
                format_ast_expr(db, &init, buf);
            }
            buf.push(')');
        }
        AstStatement::Expr(expr) => {
            format_ast_expr(db, expr, buf);
        }
    }
}

fn format_identifier(db: &dyn crate::Db, id: &SpannedIdentifier<'_>, buf: &mut String) {
    buf.push_str(id.id.text(db));
}

fn format_ast_path(db: &dyn crate::Db, path: &dada_ir_ast::ast::AstPath<'_>, buf: &mut String) {
    match path.kind(db) {
        AstPathKind::Identifier(spanned_id) => {
            format_identifier(db, spanned_id, buf);
        }
        AstPathKind::GenericArgs { path, .. } => {
            format_ast_path(db, path, buf);
            buf.push_str("[...]");
        }
        AstPathKind::Member { path, id } => {
            format_ast_path(db, path, buf);
            buf.push('.');
            format_identifier(db, id, buf);
        }
    }
}

fn escape_string_into(s: &str, buf: &mut String) {
    for ch in s.chars() {
        match ch {
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            '\\' => buf.push_str("\\\\"),
            '"' => buf.push_str("\\\""),
            c if c.is_control() => {
                buf.push_str(&format!("\\x{:02X}", c as u32));
            }
            c => buf.push(c),
        }
    }
}
