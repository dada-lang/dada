//! The "bir" (pronounced "beer") is the "base ir" that we use
//! for interpretation.

use crate::{
    class::Class,
    func::Function,
    in_ir_db::InIrDb,
    intrinsic::Intrinsic,
    op::Op,
    prelude::InIrDbExt,
    storage_mode::StorageMode,
    word::{SpannedOptionalWord, Word},
};
use dada_id::{id, prelude::*, tables};
use salsa::DebugWithDb;

use super::{syntax, Code};

salsa::entity2! {
    entity Bir in crate::Jar {
        origin: Code,
        #[value ref] data: BirData,
        #[value ref] origins: Origins,
    }
}

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BirData {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// The starting block
    pub start_basic_block: BasicBlock,
}

impl<Db: ?Sized + crate::Db> DebugWithDb<Db> for BirData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let this = &self.tables.in_ir_db(db.as_dyn_ir_db());

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

impl BirData {
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
        (0..usize::from(self.max_basic_block())).map(BasicBlock::from)
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

impl<Db: ?Sized> DebugWithDb<Db> for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct BasicBlockData {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

impl DebugWithDb<InIrDb<'_, Tables>> for BasicBlockData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        f.debug_tuple("BasicBlockData")
            .field(&self.statements.debug(db))
            .field(&self.terminator.debug(db))
            .finish()
    }
}

id!(pub struct Statement);

impl DebugWithDb<InIrDb<'_, Tables>> for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        DebugWithDb::fmt(&self.data(db), f, db)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum StatementData {
    Assign(Place, Expr),
}

impl DebugWithDb<InIrDb<'_, Tables>> for StatementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            StatementData::Assign(place, expr) => f
                .debug_tuple("Assign")
                .field(&place.debug(db))
                .field(&expr.debug(db))
                .finish(),
        }
    }
}

id!(pub struct Terminator);

impl DebugWithDb<InIrDb<'_, Tables>> for Terminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        DebugWithDb::fmt(self.data(db), f, db)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum TerminatorData {
    Goto(BasicBlock),
    If(Place, BasicBlock, BasicBlock),
    StartAtomic(BasicBlock),
    EndAtomic(BasicBlock),
    Return(Place),
    Assign(Place, TerminatorExpr, BasicBlock),
    Error,
    Panic,
}

impl DebugWithDb<InIrDb<'_, Tables>> for TerminatorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            TerminatorData::Goto(block) => f.debug_tuple("Goto").field(block).finish(),
            TerminatorData::If(condition, if_true, if_false) => f
                .debug_tuple("If")
                .field(&condition.debug(db))
                .field(&if_true.debug(db))
                .field(&if_false.debug(db))
                .finish(),
            TerminatorData::StartAtomic(block) => {
                f.debug_tuple("StartAomic").field(&block.debug(db)).finish()
            }
            TerminatorData::EndAtomic(block) => {
                f.debug_tuple("EndAtomic").field(&block.debug(db)).finish()
            }
            TerminatorData::Return(value) => {
                f.debug_tuple("Return").field(&value.debug(db)).finish()
            }
            TerminatorData::Assign(target, expr, next) => f
                .debug_tuple("Assign")
                .field(&target.debug(db))
                .field(&expr.debug(db))
                .field(&next.debug(db))
                .finish(),
            TerminatorData::Error => f.debug_tuple("Error").finish(),
            TerminatorData::Panic => f.debug_tuple("Panic").finish(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum TerminatorExpr {
    Await(Place),

    /// Call `function(arguments...)`. The `labels` for each
    /// argument are present as well.
    Call {
        function: Place,
        arguments: Vec<Place>,
        labels: Vec<SpannedOptionalWord>,
    },
}

impl DebugWithDb<InIrDb<'_, Tables>> for TerminatorExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            TerminatorExpr::Await(place) => f.debug_tuple("Await").field(&place.debug(db)).finish(),
            TerminatorExpr::Call {
                function,
                arguments,
                labels,
            } => f
                .debug_tuple("Call")
                .field(&function.debug(db))
                .field(&arguments.debug(db))
                .field(&labels.debug(db.db()))
                .finish(),
        }
    }
}

id!(pub struct Expr);

impl DebugWithDb<InIrDb<'_, Tables>> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        write!(f, "{:?}", self.data(db).debug(db))
    }
}
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum ExprData {
    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(u64),

    /// `"foo"` with no format strings
    StringLiteral(Word),

    /// `expr.share`
    Share(Place),

    /// `expr.give.share`
    ShareValue(Expr),

    /// `expr.lease`
    Lease(Place),

    /// `expr.give`
    Give(Place),

    /// `()`
    Unit,

    /// `(a, b, ...)` (i.e., at least 2)
    Tuple(Vec<Place>),

    /// `a + b`
    Op(Place, Op, Place),

    /// parse or other error
    Error,
}

impl DebugWithDb<InIrDb<'_, Tables>> for ExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            ExprData::BooleanLiteral(b) => write!(f, "{}", b),
            ExprData::IntegerLiteral(w) => write!(f, "{}", w),
            ExprData::StringLiteral(w) => write!(f, "{:?}", w.as_str(db.db())),
            ExprData::Share(p) => write!(f, "{:?}.share", p.debug(db)),
            ExprData::ShareValue(e) => write!(f, "{:?}.share", e.debug(db)),
            ExprData::Lease(p) => write!(f, "{:?}.lease", p.debug(db)),
            ExprData::Give(p) => write!(f, "{:?}.give", p.debug(db)),
            ExprData::Unit => write!(f, "()"),
            ExprData::Tuple(vars) => write_parenthesized_places(f, vars, db),
            ExprData::Op(lhs, op, rhs) => {
                write!(f, "{:?} {} {:?}", lhs.debug(db), op.str(), rhs.debug(db))
            }
            ExprData::Error => write!(f, "<error>"),
        }
    }
}

fn write_parenthesized_places(
    f: &mut std::fmt::Formatter<'_>,
    vars: &[Place],
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
        write!(f, "{:?}", self.data(db).debug(db))
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum PlaceData {
    LocalVariable(LocalVariable),
    Function(Function),
    Class(Class),
    Intrinsic(Intrinsic),
    Dot(Place, Word),
}

impl DebugWithDb<InIrDb<'_, Tables>> for PlaceData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tables>) -> std::fmt::Result {
        match self {
            PlaceData::LocalVariable(v) => write!(f, "{:?}", v.debug(db)),
            PlaceData::Function(func) => write!(f, "{:?}", func.debug(db.db())),
            PlaceData::Class(class) => write!(f, "{:?}", class.debug(db.db())),
            PlaceData::Intrinsic(intrinsic) => write!(f, "{:?}", intrinsic),
            PlaceData::Dot(p, id) => write!(f, "{:?}.{}", p.debug(db), id.as_str(db.db())),
        }
    }
}
