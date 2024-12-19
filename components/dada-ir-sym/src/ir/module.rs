use dada_ir_ast::{
    ast::{AstItem, AstModule, AstUse, Identifier},
    diagnostic::{Diagnostic, Level},
    inputs::SourceFile,
    span::{Span, Spanned},
};
use dada_parser::prelude::SourceFileParse;
use dada_util::{FromImpls, Map};

use crate::{
    check::{
        scope::Scope,
        scope_tree::{ScopeItem, ScopeTreeNode},
    },
    ir::{
        classes::SymAggregate, functions::SymFunction, primitive::SymPrimitive,
        variables::SymVariable,
    },
    prelude::Symbol,
    well_known,
};

#[salsa::tracked]
pub struct SymModule<'db> {
    pub source: AstModule<'db>,

    // Order of fields reflects the precedence we give during name resolution.
    #[return_ref]
    pub(crate) class_map: Map<Identifier<'db>, SymAggregate<'db>>,
    #[return_ref]
    pub(crate) function_map: Map<Identifier<'db>, SymFunction<'db>>,
    #[return_ref]
    pub(crate) ast_use_map: Map<Identifier<'db>, AstUse<'db>>,
}

impl<'db> Spanned<'db> for SymModule<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        self.source(db).span(db)
    }
}

/// A "prelude" is a set of item names automatically imported into scope.
#[salsa::interned]
pub struct SymPrelude<'db> {
    pub items: Vec<SymItem<'db>>,
}

#[salsa::tracked]
impl<'db> SymModule<'db> {
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        self.source(db).name(db)
    }

    /// Name resolution scope for items in this module.
    pub fn mod_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        Scope::new(db, self.span(db)).with_link(self)
    }

    /// Returns a list of all top-level items in the module
    pub fn items(self, db: &'db dyn crate::Db) -> impl Iterator<Item = SymItem<'db>> {
        self.class_map(db)
            .values()
            .copied()
            .map(|i| SymItem::from(i))
            .chain(
                self.function_map(db)
                    .values()
                    .copied()
                    .map(|i| SymItem::from(i)),
            )
    }
}

#[salsa::tracked]
impl<'db> ScopeTreeNode<'db> for SymModule<'db> {
    fn direct_super_scope(self, _db: &'db dyn crate::Db) -> Option<ScopeItem<'db>> {
        None // FIXME
    }

    #[salsa::tracked(return_ref)]
    fn direct_generic_parameters(self, _db: &'db dyn crate::Db) -> Vec<SymVariable<'db>> {
        vec![] // FIXME: we expect to add these in the future
    }

    fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        self.mod_scope(db)
    }
}

impl<'db> Symbol<'db> for SourceFile {
    type Output = SymModule<'db>;

    fn symbol(self, db: &'db dyn crate::Db) -> Self::Output {
        self.parse(db).symbol(db)
    }
}

#[salsa::tracked]
impl<'db> Symbol<'db> for AstModule<'db> {
    type Output = SymModule<'db>;

    #[salsa::tracked]
    fn symbol(self, db: &'db dyn crate::Db) -> SymModule<'db> {
        let mut class_map = Map::default();
        let mut function_map = Map::default();
        let mut ast_use_map = Map::default();
        for item in self.items(db) {
            match *item {
                AstItem::SourceFile(_) => {}
                AstItem::Use(ast_use) => {
                    let id = match ast_use.as_id(db) {
                        Some(as_id) => as_id.id,
                        None => ast_use.path(db).last_id(db).id,
                    };

                    insert(db, &mut ast_use_map, id, ast_use.into());
                }
                AstItem::Aggregate(ast_class_item) => {
                    insert(
                        db,
                        &mut class_map,
                        ast_class_item.name(db),
                        SymAggregate::new(db, self.into(), ast_class_item),
                    );
                }
                AstItem::Function(ast_function) => {
                    insert(
                        db,
                        &mut function_map,
                        ast_function.name(db).id,
                        SymFunction::new(db, self.into(), ast_function.into()),
                    );
                }
            }
        }

        // Detect duplicates between maps. The order is significant here;
        // when resolving names, we prefer the maps that come earlier in this list.
        let canonical_map = &mut Map::default();
        insert_into_canonical_map(db, canonical_map, &class_map);
        insert_into_canonical_map(db, canonical_map, &function_map);
        insert_into_canonical_map(db, canonical_map, &ast_use_map);

        SymModule::new(db, self, class_map, function_map, ast_use_map)
    }
}

impl<'db> ScopeTreeNode<'db> for AstModule<'db> {
    fn direct_super_scope(self, db: &'db dyn crate::Db) -> Option<ScopeItem<'db>> {
        self.symbol(db).direct_super_scope(db)
    }

    fn direct_generic_parameters(self, db: &'db dyn crate::Db) -> &'db Vec<SymVariable<'db>> {
        self.symbol(db).direct_generic_parameters(db)
    }

    fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        self.symbol(db).into_scope(db)
    }
}

fn insert<'db, V: Spanned<'db>>(
    db: &'db dyn crate::Db,
    map: &mut Map<Identifier<'db>, V>,
    id: Identifier<'db>,
    value: V,
) {
    if let Some(other_value) = map.get(&id) {
        report_duplicate(db, id, value.span(db), other_value.span(db));
    } else {
        map.insert(id, value);
    }
}

fn insert_into_canonical_map<'db>(
    db: &'db dyn crate::Db,
    canonical_map: &mut Map<Identifier<'db>, Span<'db>>,
    map: &Map<Identifier<'db>, impl Spanned<'db>>,
) {
    for (id, value) in map.iter() {
        let id = *id;
        let value_span = value.span(db);
        if let Some(canonical_span) = canonical_map.get(&id) {
            report_duplicate(db, id, value_span, *canonical_span);
        } else {
            canonical_map.insert(id, value_span);
        }
    }
}

fn report_duplicate<'db>(
    db: &'db dyn crate::Db,
    id: Identifier<'db>,
    value_span: Span<'db>,
    canonical_span: Span<'db>,
) {
    Diagnostic::error(
        db,
        value_span,
        "this definition is a duplicate and will be ignored",
    )
    .label(db, Level::Error, value_span, "duplicate definition")
    .label(
        db,
        Level::Info,
        canonical_span,
        format!("we will map `{id:?}` to this other definition"),
    )
    .report(db);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromImpls)]
pub enum SymItem<'db> {
    SymClass(SymAggregate<'db>),
    SymFunction(SymFunction<'db>),
    SymPrimitive(SymPrimitive<'db>),
}

impl<'db> SymItem<'db> {
    pub fn name(self, db: &'db dyn crate::Db) -> Identifier<'db> {
        match self {
            SymItem::SymClass(sym_class) => sym_class.name(db),
            SymItem::SymFunction(sym_function) => sym_function.name(db),
            SymItem::SymPrimitive(sym_primitive) => sym_primitive.name(db),
        }
    }
}

impl<'db> Spanned<'db> for SymItem<'db> {
    fn span(&self, db: &'db dyn dada_ir_ast::Db) -> Span<'db> {
        match self {
            SymItem::SymClass(sym_class) => sym_class.span(db),
            SymItem::SymFunction(sym_function) => sym_function.span(db),
            SymItem::SymPrimitive(_) => well_known::prelude_span(db),
        }
    }
}
