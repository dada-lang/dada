use dada_id::prelude::*;
use dada_ir::{
    code::{syntax, Code},
    filename::Filename,
    item::Item,
    origin_table::HasOriginIn,
    span::{FileSpan, LineColumn, Offset},
};
use dada_parse::prelude::*;

/// Identifies a particular expression in the syntax tree.
///
/// See [`find`].
#[derive(Copy, Clone, Debug)]
pub struct Breakpoint {
    pub item: Item,
    pub code: Code,
    pub expr: syntax::Expr,
}

impl Breakpoint {
    /// Returns the file-span of the breakpoint expression.
    pub fn span(self, db: &dyn crate::Db) -> FileSpan {
        let tree = self.code.syntax_tree(db);
        let expr_span = tree.spans(db)[self.expr];
        let filename = self.code.filename(db);
        expr_span.in_file(filename)
    }
}

/// Given a cursor position, finds the breakpoint associated with that cursor
/// (if any). This is the expression that the cursor is on.
pub fn find(db: &dyn crate::Db, filename: Filename, position: LineColumn) -> Option<Breakpoint> {
    let offset = dada_ir::lines::offset(db, filename, position);

    let item = find_item(db, filename, offset)?;
    let code = item.code(db)?;
    let syntax_tree = code.syntax_tree(db);
    let cusp_expr = find_syntax_expr(db, syntax_tree, offset);
    Some(Breakpoint {
        item,
        code,
        expr: cusp_expr,
    })
}

pub fn find_item(db: &dyn crate::Db, filename: Filename, offset: Offset) -> Option<Item> {
    filename
        .items(db)
        .iter()
        .find(|item| item.span(db).contains(offset))
        .copied()
}

/// Locates the syntax expression that the cursor is "on".
/// This is used to in the time-travelling debugger.
///
/// The idea is that executions stops at the "cusp" of the returned expression E:
/// that is, the moment when all of E's children have been
/// evaluated, but E has not yet taken effect itself.
///
/// Assumes: the offest is somewhere in this syntax tree.
///
/// Returns None if the cursor does not lie in the syntax tree at all.
fn find_syntax_expr(db: &dyn crate::Db, syntax_tree: syntax::Tree, offset: Offset) -> syntax::Expr {
    let spans = syntax_tree.spans(db);
    let data = syntax_tree.data(db);
    let traversal = TreeTraversal {
        spans,
        tables: &data.tables,
        offset,
    };
    traversal.find(data.root_expr).unwrap_or(data.root_expr)
}

struct TreeTraversal<'me> {
    spans: &'me syntax::Spans,
    tables: &'me syntax::Tables,
    offset: Offset,
}

macro_rules! search {
    ($self:expr, $expr:expr) => {
        if let Some(r) = $self.find($expr) {
            return Some(r);
        }
    };
}

impl TreeTraversal<'_> {
    fn find(&self, expr: syntax::Expr) -> Option<syntax::Expr> {
        let span = expr.origin_in(self.spans);

        // Note: we purposefully don't check against `span.start`.
        // We assume our parent has done any `span.start` checks.
        // This is helpful for cases like `return x` -- it means that
        // placing the cursor on `return` is the same as placing it
        // on `x`.
        if self.offset >= span.end {
            return None;
        }

        match expr.data(self.tables) {
            syntax::ExprData::Error
            | syntax::ExprData::Id(_)
            | syntax::ExprData::BooleanLiteral(_)
            | syntax::ExprData::IntegerLiteral(..)
            | syntax::ExprData::FloatLiteral(_, _)
            | syntax::ExprData::StringLiteral(_) => Some(expr),

            syntax::ExprData::Var(_, base_expr)
            | syntax::ExprData::Dot(base_expr, _)
            | syntax::ExprData::Share(base_expr)
            | syntax::ExprData::Lease(base_expr)
            | syntax::ExprData::Give(base_expr)
            | syntax::ExprData::Await(base_expr)
            | syntax::ExprData::Loop(base_expr)
            | syntax::ExprData::Atomic(base_expr)
            | syntax::ExprData::Parenthesized(base_expr) => {
                self.find_in_children(expr, Some(base_expr))
            }

            syntax::ExprData::Return(base_expr) => self.find_in_children(expr, base_expr),

            syntax::ExprData::Tuple(child_exprs) | syntax::ExprData::Seq(child_exprs) => {
                self.find_in_children(expr, child_exprs)
            }

            syntax::ExprData::Call(func_expr, arg_exprs) => self.find_in_children(
                expr,
                std::iter::once(func_expr).chain(
                    arg_exprs
                        .iter()
                        .map(|named_expr| &named_expr.data(self.tables).expr),
                ),
            ),

            syntax::ExprData::If(condition_expr, if_true_expr, if_false_expr) => {
                // Because `if` has alternate control flow, it's a bit different from `find_in_children`.
                // If the cursor is on the `else` keyword, for example, where do we go? We settle on "start of the if"
                // for now, but that's not obviously correct, we might want to go *into* the else block.
                search!(self, *condition_expr);

                search!(self, *if_true_expr);

                if let Some(if_false_expr) = if_false_expr {
                    let if_true_span = if_false_expr.origin_in(self.spans);
                    let if_false_span = if_false_expr.origin_in(self.spans);
                    if self.offset >= if_true_span.end && self.offset < if_false_span.start {
                        return Some(*if_false_expr);
                    }
                    search!(self, *if_false_expr);
                }

                Some(expr)
            }

            syntax::ExprData::While(condition_expr, body_expr) => {
                self.find_in_children(expr, [condition_expr, body_expr])
            }

            syntax::ExprData::Assign(lhs, rhs)
            | syntax::ExprData::Op(lhs, _, rhs)
            | syntax::ExprData::OpEq(lhs, _, rhs) => self.find_in_children(expr, [lhs, rhs]),
        }
    }

    fn find_in_children<'c>(
        &self,
        parent_expr: syntax::Expr,
        child_exprs: impl IntoIterator<Item = &'c syntax::Expr>,
    ) -> Option<syntax::Expr> {
        let mut child_exprs = child_exprs.into_iter();
        if let Some(first_child_expr) = child_exprs.next() {
            search!(self, *first_child_expr);

            let mut previous_expr = *first_child_expr;
            for child_expr in child_exprs {
                let child_span = child_expr.origin_in(self.spans);

                if self.offset < child_span.start {
                    // The cursor lies "in between" the previous expression (if any) and this one.
                    // So we want to stop execution just before the child expression begins.
                    return Some(previous_expr);
                }

                // Check if the cursor lies *inside* the child.
                search!(self, *child_expr);
                previous_expr = *child_expr;
            }
        }

        // The cursor lies after the final child expression, so the
        // parent_expr is "on the cusp" of taking effect.
        Some(parent_expr)
    }
}
