use dada_ir_ast::ast::Identifier;
use salsa::Update;

/// Returns the standard primitives available in Dada.
#[salsa::tracked(return_ref)]
pub fn primitives<'db>(db: &'db dyn crate::Db) -> Vec<SymPrimitive<'db>> {
    vec![
        SymPrimitive::new(db, SymPrimitiveKind::Bool),
        SymPrimitive::new(db, SymPrimitiveKind::Char),
        SymPrimitive::new(db, SymPrimitiveKind::Isize),
        SymPrimitive::new(db, SymPrimitiveKind::Usize),
        SymPrimitive::new(db, SymPrimitiveKind::Float { bits: 32 }),
        SymPrimitive::new(db, SymPrimitiveKind::Float { bits: 64 }),
        SymPrimitive::new(db, SymPrimitiveKind::Int { bits: 8 }),
        SymPrimitive::new(db, SymPrimitiveKind::Int { bits: 16 }),
        SymPrimitive::new(db, SymPrimitiveKind::Int { bits: 32 }),
        SymPrimitive::new(db, SymPrimitiveKind::Int { bits: 64 }),
        SymPrimitive::new(db, SymPrimitiveKind::Uint { bits: 8 }),
        SymPrimitive::new(db, SymPrimitiveKind::Uint { bits: 16 }),
        SymPrimitive::new(db, SymPrimitiveKind::Uint { bits: 32 }),
        SymPrimitive::new(db, SymPrimitiveKind::Uint { bits: 64 }),
    ]
}

/// A "primitive" is a scalar type that is built-in to Dada and cannot be defined as an aggregate
/// type like a struct, enum, or class.
#[salsa::interned]
pub struct SymPrimitive<'db> {
    pub kind: SymPrimitiveKind,
}

impl std::fmt::Display for SymPrimitive<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            write!(f, "{}", self.name(db))
        })
        .unwrap_or_else(|| std::fmt::Debug::fmt(self, f))
    }
}

/// A "primitive" is a scalar type that is built-in to Dada and cannot be defined as a struct.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub enum SymPrimitiveKind {
    Bool,
    Char,
    Int { bits: u32 },
    Isize,
    Uint { bits: u32 },
    Usize,
    Float { bits: u32 },
}

impl<'db> SymPrimitive<'db> {
    /// Gets the name of the scalar type.
    /// We give them the same names as Rust.
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        match self.kind(db) {
            SymPrimitiveKind::Bool => Identifier::new(db, "bool".to_string()),
            SymPrimitiveKind::Char => Identifier::new(db, "char".to_string()),
            SymPrimitiveKind::Int { bits } => Identifier::new(db, format!("i{bits}")),
            SymPrimitiveKind::Isize => Identifier::new(db, "isize".to_string()),
            SymPrimitiveKind::Uint { bits } => Identifier::new(db, format!("u{bits}")),
            SymPrimitiveKind::Usize => Identifier::new(db, "usize".to_string()),
            SymPrimitiveKind::Float { bits } => Identifier::new(db, format!("f{bits}")),
        }
    }
}

impl SymPrimitiveKind {
    /// Intern `SymPrimitive<'db>`
    pub fn intern<'db>(self, db: &'db dyn crate::Db) -> SymPrimitive<'db> {
        SymPrimitive::new(db, self)
    }
}
