use std::ops::ControlFlow;

use dada_ir_ast::span::{AbsoluteSpan, SourceSpanned};
pub use dada_ir_sym::Db;
use dada_ir_sym::{
    ir::{
        exprs::{SymExpr, SymExprKind},
        functions::SymFunction,
        module::SymItem,
    },
    prelude::{CheckedBody, Symbol},
};

/// Probe for the type of a variable found in a given file at a given span.
pub fn probe_variable_type<'db>(db: &'db dyn crate::Db, span: AbsoluteSpan) -> Option<String> {
    // We expect `span` to be located in
    visit_exprs(db, span, &|expr| {
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
        SymItem::SymFunction(func) => return Some(func),
        SymItem::SymPrimitive(_) => return None,
    }
}

/// Walk all expressions containing the given `span` and invoke `op`.
/// Stops if `op` returns `ControlFlow::Break`.
fn visit_exprs<'db, B>(
    db: &'db dyn crate::Db,
    span: AbsoluteSpan,
    op: &dyn Fn(SymExpr<'db>) -> ControlFlow<B>,
) -> Option<B> {
    let func = find_func(db, span)?;
    let expr = func.checked_body(db)?;
    walk_expr_and_visit(db, expr, span, op)
}

fn walk_expr_and_visit<'db, B>(
    db: &'db dyn crate::Db,
    expr: SymExpr<'db>,
    span: AbsoluteSpan,
    op: &dyn Fn(SymExpr<'db>) -> ControlFlow<B>,
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
