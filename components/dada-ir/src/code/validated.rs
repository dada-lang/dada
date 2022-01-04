//! The "validated" IR is the IR that we use for type checks
//! and so forth. It is still in tree form and is mildly
//! desugared and easy to work with.

use crate::{
    class::Class, func::Function, in_ir_db::InIrDb, intrinsic::Intrinsic, op::Op,
    prelude::InIrDbExt, storage_mode::StorageMode, word::Word,
};
use dada_id::{id, prelude::*, tables};
use salsa::DebugWithDb;

use super::{syntax, Code};

salsa::entity2! {
    entity Tree in crate::Jar {
        origin: Code,
        #[value ref] data: TreeData,
        #[value ref] origins: Origins,
    }
}

impl<Db: ?Sized + crate::Db> DebugWithDb<Db> for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        f.debug_struct("validated::Tree")
            .field("origin", &self.origin(db.as_dyn_ir_db()).debug(db)) // FIXME
            .finish()
    }
}

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeData {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// The root
    pub root_expr: Expr,
}

impl<Db: ?Sized + crate::Db> DebugWithDb<Db> for TreeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let in_db = self.tables.in_ir_db(db.as_dyn_ir_db());
        f.debug_struct("validated::Tree")
            .field("root_expr", &self.root_expr.debug(&in_db))
            .finish()?;
        Ok(())
    }
}

impl TreeData {
    pub fn new(tables: Tables, root_expr: Expr) -> Self {
        Self { tables, root_expr }
    }

    pub fn max_local_variable(&self) -> LocalVariable {
        LocalVariable::max_key(&self.tables)
    }
}

tables! {
    /// Tables that store the data for expr in the AST.
    /// You can use `tables[expr]` (etc) to access the data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tables {
        local_variables: alloc LocalVariable => LocalVariableData,
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        places: alloc Place => PlaceData,
    }
}

origin_table! {
    /// Side table that contains the spans for everything in a syntax tree.
    /// This isn't normally needed except for diagnostics, so it's
    /// kept separate to avoid reducing incremental reuse.
    /// You can request it by invoking the `spans`
    /// method in the `dada_parse` prelude.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Origins {
        expr_spans: Expr => syntax::Expr,
        place_spans: Place => syntax::Expr,
        named_exprs: NamedExpr => syntax::NamedExpr,
        local_variables: LocalVariable => syntax::Expr,
    }
}

id!(pub struct LocalVariable);

impl DebugWithDb<InIrDb<'_, Tables>> for LocalVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        let id = u32::from(*self);
        let data = self.data(db);
        let name = data.name.map(|n| n.as_str(db.db())).unwrap_or("temp");
        write!(f, "{name}{{{id}}}")
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct LocalVariableData {
    /// Name given to this variable by the user.
    /// If it is None, then this is a temporary
    /// introduced by the compiler.
    pub name: Option<Word>,
    pub storage_mode: StorageMode,
}

id!(pub struct Expr);

impl DebugWithDb<InIrDb<'_, Tables>> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        self.data(db).pretty_print(Some(*self), f, db)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum ExprData {
    /// Reference to a local variable
    Place(Place),

    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(u64),

    /// `"foo"` with no format strings
    StringLiteral(Word),

    /// `expr.await`
    Await(Expr),

    /// `expr(id: expr, ...)`
    Call(Expr, Vec<NamedExpr>),

    /// `expr.share`
    Share(Place),

    /// `expr.lease`
    Lease(Place),

    /// `expr.give`
    Give(Place),

    /// `()` or `(a, b, ...)` (i.e., expr seq cannot have length 1)
    Tuple(Vec<Expr>),

    /// `if condition { block } [else { block }]`
    If(Expr, Expr, Expr),

    /// `atomic { block }`
    Atomic(Expr),

    /// `loop { block }`
    Loop(Expr),

    /// `break [from expr] [with value]`
    ///
    /// * `from_expr`: Identifies the loop from which we are breaking
    /// * `with_value`: The value produced by the loop
    Break { from_expr: Expr, with_value: Expr },

    /// `continue`
    ///
    /// * `0`: identifies the loop with which we are continuing.
    Continue(Expr),

    /// `break [from expr] [with value]`
    Return(Expr),

    /// expr[0]; expr[1]; ...
    Seq(Vec<Expr>),

    /// `a + b`
    Op(Expr, Op, Expr),

    /// `a := b`
    Assign(Place, Expr),

    /// parse or other error
    Error,
}

impl DebugWithDb<InIrDb<'_, Tables>> for ExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        self.pretty_print(None, f, db)
    }
}

impl ExprData {
    fn pretty_print(
        &self,
        id: Option<Expr>,
        f: &mut std::fmt::Formatter<'_>,
        db: &InIrDb<'_, Tables>,
    ) -> std::fmt::Result {
        let id = id.map(u32::from);
        match self {
            ExprData::Place(p) => DebugWithDb::fmt(p, f, db),
            ExprData::BooleanLiteral(v) => std::fmt::Debug::fmt(v, f),
            ExprData::IntegerLiteral(v) => write!(f, "{}", v),
            ExprData::StringLiteral(v) => std::fmt::Debug::fmt(&v.as_str(db.db()), f),
            ExprData::Await(expr) => f.debug_tuple("Await").field(&expr.debug(db)).finish(),
            ExprData::Call(expr, args) => f
                .debug_tuple("Call")
                .field(&expr.debug(db))
                .field(&args.debug(db))
                .finish(),
            ExprData::Share(p) => f.debug_tuple("Share").field(p).finish(),
            ExprData::Lease(p) => f.debug_tuple("Lease").field(p).finish(),
            ExprData::Give(p) => f.debug_tuple("Give").field(p).finish(),
            ExprData::Tuple(exprs) => {
                let mut f = f.debug_tuple("");
                for expr in exprs {
                    f.field(&expr.debug(db));
                }
                f.finish()
            }
            ExprData::If(condition, if_true, if_false) => f
                .debug_tuple("If")
                .field(&condition.debug(db))
                .field(&if_true.debug(db))
                .field(&if_false.debug(db))
                .finish(),
            ExprData::Atomic(e) => f.debug_tuple("Atomic").field(&e.debug(db)).finish(),
            ExprData::Loop(e) => f
                .debug_tuple("Loop")
                .field(&id)
                .field(&e.debug(db))
                .finish(),
            ExprData::Break {
                from_expr,
                with_value,
            } => f
                .debug_tuple("Break")
                .field(&u32::from(*from_expr))
                .field(&with_value.debug(db))
                .finish(),
            ExprData::Continue(loop_expr) => f
                .debug_tuple("Continue")
                .field(&u32::from(*loop_expr))
                .finish(),
            ExprData::Return(value) => f.debug_tuple("Return").field(&value.debug(db)).finish(),
            ExprData::Seq(exprs) => f.debug_tuple("Seq").field(&exprs.debug(db)).finish(),
            ExprData::Op(lhs, op, rhs) => f
                .debug_tuple("Op")
                .field(&lhs.debug(db))
                .field(op)
                .field(&rhs.debug(db))
                .finish(),
            ExprData::Assign(place, expr) => f
                .debug_tuple("Assign")
                .field(&place.debug(db))
                .field(&expr.debug(db))
                .finish(),
            ExprData::Error => f.debug_tuple("Error").finish(),
        }
    }
}

id!(pub struct Place);

impl DebugWithDb<InIrDb<'_, Tables>> for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        DebugWithDb::fmt(&self.data(db), f, db)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum PlaceData {
    LocalVariable(LocalVariable),
    Function(Function),
    Intrinsic(Intrinsic),
    Class(Class),
    Dot(Place, Word),
}

impl DebugWithDb<InIrDb<'_, Tables>> for PlaceData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            PlaceData::LocalVariable(lv) => DebugWithDb::fmt(lv, f, db),
            PlaceData::Function(function) => DebugWithDb::fmt(function, f, db.db()),
            PlaceData::Intrinsic(intrinsic) => std::fmt::Debug::fmt(intrinsic, f),
            PlaceData::Class(class) => DebugWithDb::fmt(class, f, db.db()),
            PlaceData::Dot(place, field) => f
                .debug_tuple("Dot")
                .field(&place.debug(db))
                .field(&field.debug(db.db()))
                .finish(),
        }
    }
}

id!(pub struct NamedExpr);

impl DebugWithDb<InIrDb<'_, Tables>> for NamedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        DebugWithDb::fmt(&self.data(db), f, db)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: Word,
    pub expr: Expr,
}

impl DebugWithDb<InIrDb<'_, Tables>> for NamedExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        f.debug_tuple(self.name.as_str(db.db()))
            .field(&self.expr.debug(db))
            .finish()
    }
}
