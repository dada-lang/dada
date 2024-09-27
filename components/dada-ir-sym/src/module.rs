use dada_ir_ast::{
    ast::{AstItem, AstModule, AstUseItem, Identifier},
    diagnostic::{Diagnostic, Level},
    inputs::SourceFile,
    span::{Span, Spanned},
};
use dada_parser::prelude::SourceFileParse;
use dada_util::Map;

use crate::{class::SymClass, function::SymFunction, prelude::Symbolize};

#[salsa::tracked]
pub struct SymModule<'db> {
    // Order of fields reflects the precedence we give during name resolution.
    #[return_ref]
    pub(crate) class_map: Map<Identifier<'db>, SymClass<'db>>,
    #[return_ref]
    pub(crate) function_map: Map<Identifier<'db>, SymFunction<'db>>,
    #[return_ref]
    pub(crate) ast_use_map: Map<Identifier<'db>, AstUseItem<'db>>,
}

impl<'db> Symbolize<'db> for SourceFile {
    type Symbolic = SymModule<'db>;

    fn symbolize(self, db: &'db dyn crate::Db) -> Self::Symbolic {
        self.parse(db).symbolize(db)
    }
}

impl<'db> Symbolize<'db> for AstModule<'db> {
    type Symbolic = SymModule<'db>;

    fn symbolize(self, db: &'db dyn crate::Db) -> SymModule<'db> {
        let mut class_map = Map::default();
        let mut function_map = Map::default();
        let mut ast_use_map = Map::default();
        for item in self.items(db) {
            match *item {
                AstItem::SourceFile(_) => {}
                AstItem::Use(ast_use) => {
                    let id = match ast_use.as_id(db) {
                        Some(as_id) => as_id.id,
                        None => ast_use.path(db).last_id().id,
                    };

                    insert(db, &mut ast_use_map, id, ast_use.into());
                }
                AstItem::Class(ast_class_item) => {
                    insert(
                        db,
                        &mut class_map,
                        ast_class_item.name(db),
                        SymClass::new(db, self, ast_class_item),
                    );
                }
                AstItem::Function(ast_function) => {
                    insert(
                        db,
                        &mut function_map,
                        ast_function.name(db).id,
                        SymFunction::new(db, self.into(), ast_function),
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

        SymModule::new(db, class_map, function_map, ast_use_map)
    }
}

fn insert<'db, V: Spanned<'db>>(
    db: &'db dyn crate::Db,
    map: &mut Map<Identifier<'db>, V>,
    id: Identifier<'db>,
    value: V,
) {
    if let Some(other_value) = map.get(&id) {
        report_duplicate(
            db,
            id,
            value.span(db.as_dyn_database()),
            other_value.span(db.as_dyn_database()),
        );
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
        let value_span = value.span(db.as_dyn_database());
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
    let db: &dyn salsa::Database = db.as_dyn_database();
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
