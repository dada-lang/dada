//! The "bir" (pronounced "beer") is the "base ir" that we use
//! for interpretation.

use crate::{
    class::Class, func::Function, in_ir_db::InIrDb, op::Op, prelude::InIrDbExt,
    storage_mode::StorageMode, word::Word,
};
use dada_id::{id, prelude::*, tables};
use salsa::DebugWithDb;

use super::syntax;

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bir {
    /// Interning tables for expressions and the like.
    tables: Tables,

    /// The starting block
    start_basic_block: BasicBlock,
}

impl DebugWithDb<dyn crate::Db + '_> for Bir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        let this = &self.tables.in_ir_db(db);

        let mut dbg = f.debug_struct("validated::Tree");
        dbg.field("start_basic_block", &self.start_basic_block);

        for basic_block in self.all_basic_blocks() {
            dbg.field(
                &format!("{basic_block:?}"),
                &basic_block.data(this).debug(this),
            );
        }

        dbg.finish()
    }
}

impl Bir {
    pub fn new(tables: Tables, start_basic_block: BasicBlock) -> Self {
        Self {
            tables,
            start_basic_block,
        }
    }

    pub fn tables(&self) -> &Tables {
        &self.tables
    }

    pub fn start_basic_block(&self) -> BasicBlock {
        self.start_basic_block
    }

    pub fn max_local_variable(&self) -> LocalVariable {
        LocalVariable::max_key(&self.tables)
    }

    pub fn max_basic_block(&self) -> BasicBlock {
        BasicBlock::max_key(&self.tables)
    }
    pub fn all_basic_blocks(&self) -> impl Iterator<Item = BasicBlock> {
        (0..usize::from(self.max_basic_block())).map(|i| BasicBlock::from(i))
    }
}

tables! {
    /// Tables that store the data for expr in the AST.
    /// You can use `tables[expr]` (etc) to access the data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tables {
        local_variables: alloc LocalVariable => LocalVariableData,
        basic_blocks: alloc BasicBlock => BasicBlockData,
        statements: alloc Statement => StatementData,
        terminators: alloc Terminator => TerminatorData,
        exprs: alloc Expr => ExprData,
        places: alloc Place => PlaceData,
        named_exprs: alloc NamedExpr => NamedExprData,
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
        /// Maps to either the declaration of the variable (if this is a named variable)
        /// or the expression this is a temporary for.
        local_variables: LocalVariable => syntax::Expr,
        basic_blocks: BasicBlock => syntax::Expr,
        statements: Statement => syntax::Expr,
        terminator: Terminator => syntax::Expr,
        expr: Expr => syntax::Expr,
        place: Place => syntax::Expr,
        named_expr: NamedExpr => syntax::NamedExpr,
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LocalVariableData {
    /// Name given to this variable by the user.
    /// If it is None, then this is a temporary
    /// introduced by the compiler.
    pub name: Option<Word>,
    pub storage_mode: StorageMode,
}

id!(pub struct BasicBlock);

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct BasicBlockData {
    statements: Vec<Statement>,
    terminator: Terminator,
}

impl DebugWithDb<InIrDb<'_, Tables>> for BasicBlockData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        let mut f = f.debug_struct("BasicBlockData");

        let statements: Vec<_> = self
            .statements
            .iter()
            .map(|s| s.data(db).debug(db))
            .collect();
        f.field("statements", &statements);

        f.finish()
    }
}

id!(pub struct Statement);

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum StatementData {
    Assign(Place, Expr),
}

impl DebugWithDb<InIrDb<'_, Tables>> for StatementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            StatementData::Assign(place, expr) => {
                write!(
                    f,
                    "{:?} = {:?}",
                    place.data(db).debug(db),
                    expr.data(db).debug(db)
                )
            }
        }
    }
}

id!(pub struct Terminator);

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum TerminatorData {
    If(LocalVariable, BasicBlock, BasicBlock),
    Await(LocalVariable, BasicBlock),
    StartAtomic(BasicBlock),
    EndAtomic(BasicBlock),
    Return(LocalVariable),
    Call {
        arguments: Vec<NamedExpr>,
        if_ok: BasicBlock,
    },
}

id!(pub struct Expr);

impl DebugWithDb<InIrDb<'_, Tables>> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        write!(f, "{:?}", self.data(db))
    }
}
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum ExprData {
    /// Reference to a local variable
    Place(Place),

    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(Word),

    /// `"foo"` with no format strings
    StringLiteral(Word),

    /// `expr.share`
    Share(Place),

    /// `expr.lease`
    Lease(Place),

    /// `expr.give`
    Give(Place),

    /// `()` or `(a, b, ...)` (i.e., expr seq cannot have length 1)
    Tuple(Vec<LocalVariable>),

    /// allocate an instance of a class
    New(Class, Vec<LocalVariable>),

    /// `a + b`
    Op(LocalVariable, Op, LocalVariable),

    /// parse or other error
    Error,
}

impl DebugWithDb<InIrDb<'_, Tables>> for ExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            ExprData::Place(p) => write!(f, "{:?}", p.debug(db)),
            ExprData::BooleanLiteral(b) => write!(f, "{}", b),
            ExprData::IntegerLiteral(w) => write!(f, "{}", w.as_str(db.db())),
            ExprData::StringLiteral(w) => write!(f, "{:?}", w.as_str(db.db())),
            ExprData::Share(p) => write!(f, "{:?}.share", p.debug(db)),
            ExprData::Lease(p) => write!(f, "{:?}.lease", p.debug(db)),
            ExprData::Give(p) => write!(f, "{:?}.give", p.debug(db)),
            ExprData::Tuple(vars) => write_parenthesized_variables(f, vars, db),
            ExprData::New(class, vars) => {
                write!(f, "{}", class.name(db.db()).as_str(db.db()))?;
                write_parenthesized_variables(f, vars, db)
            }
            ExprData::Op(lhs, op, rhs) => {
                write!(f, "{:?} {} {:?}", lhs.debug(db), op.str(), rhs.debug(db))
            }
            ExprData::Error => write!(f, "<error>"),
        }
    }
}

fn write_parenthesized_variables(
    f: &mut std::fmt::Formatter<'_>,
    vars: &Vec<LocalVariable>,
    db: &InIrDb<'_, Tables>,
) -> std::fmt::Result {
    write!(f, "(")?;
    for (v, i) in vars.iter().zip(0..) {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{:?}", v.debug(db))?;
    }
    write!(f, ")")?;
    Ok(())
}

id!(pub struct Place);

impl DebugWithDb<InIrDb<'_, Tables>> for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        write!(f, "{:?}", self.data(db))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum PlaceData {
    LocalVariable(LocalVariable),
    Function(Function),
    Class(Class),
    Dot(Place, Word),
}

impl DebugWithDb<InIrDb<'_, Tables>> for PlaceData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            PlaceData::LocalVariable(v) => write!(f, "{:?}", v.debug(db)),
            PlaceData::Function(func) => write!(f, "{:?}", func.debug(db.db())),
            PlaceData::Class(class) => write!(f, "{:?}", class.debug(db.db())),
            PlaceData::Dot(p, id) => write!(f, "{:?}.{}", p.debug(db), id.as_str(db.db())),
        }
    }
}

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: Word,
    pub expr: Expr,
}

impl DebugWithDb<InIrDb<'_, Tables>> for NamedExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:?}",
            self.name.as_str(db.db()),
            self.expr.data(db).debug(db),
        )
    }
}
