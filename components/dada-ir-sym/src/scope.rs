use dada_ir_ast::{
    add_from_impls,
    ast::{AstItem, AstModule, AstUseItem, Identifier},
    diagnostic::{Diagnostic, Level},
    inputs::CrateKind,
    span::Spanned,
};
use dada_parser::prelude::SourceFileParse;
use dada_util::Map;
use salsa::Update;

use crate::{
    class::SymClass, function::SymFunction, module::SymModule, prelude::Symbolize,
    symbol::SymLocalVariable,
};

add_from_impls! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update)]
    pub enum ScopeItem<'db> {
        Module(AstModule<'db>),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Update)]
pub(crate) struct Scope<'db> {
    chain: ScopeChain<'db>,
}

#[derive(Clone, Debug, PartialEq, Eq, Update)]
pub(crate) struct ScopeChain<'db> {
    link: ScopeChainLink<'db>,
    next: Option<Box<ScopeChain<'db>>>,
}

add_from_impls! {
#[derive(Clone, Debug, PartialEq, Eq, Update)]
pub(crate) enum ScopeChainLink<'db> {
    SymModule(SymModule<'db>),
    LocalVariables(LocalVariables<'db>),
}
}

#[derive(Clone, Debug, PartialEq, Eq, Update)]
pub(crate) struct LocalVariables<'db> {
    names: Map<Identifier<'db>, SymLocalVariable<'db>>,
}

add_from_impls! {
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NameResolution<'db> {
    SymModule(SymModule<'db>),
    SymClass(SymClass<'db>),
    SymLocalVariable(SymLocalVariable<'db>),
    SymFunction(SymFunction<'db>),
}
}

impl<'db> Scope<'db> {
    pub fn new(db: &'db dyn crate::Db, item: ScopeItem<'db>) -> Self {
        match item {
            ScopeItem::Module(ast_module) => {
                Scope {
                    chain: ScopeChain {
                        link: ScopeChainLink::from(ast_module.symbolize(db)),
                        next: None,
                    },
                }
            }
        }
    }

    /// Extend this scope with another link in the name resolution chain
    pub fn with_link(self, link: impl Into<ScopeChainLink<'db>>) -> Self {
        Scope {
            chain: ScopeChain {
                link: link.into(),
                next: Some(Box::new(self.chain)),
            },
        }
    }

    pub fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
        self.chain.resolve_name(db, id)
    }
}

impl<'db> ScopeChain<'db> {
    fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
        self.link.resolve_name(db, id).or_else(|| {
            self.next
                .as_ref()
                .and_then(|chain| chain.resolve_name(db, id))
        })
    }
}

impl<'db> ScopeChainLink<'db> {
    fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
        match self {
            ScopeChainLink::SymModule(sym_module) => sym_module.resolve_name(db, id),
            ScopeChainLink::LocalVariables(local_variables) => {
                Some(local_variables.names.get(&id).cloned()?.into())
            }
        }
    }
}

impl<'db> SymModule<'db> {
    fn resolve_name(
        self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
        if let Some(&v) = self.class_map(db).get(&id) {
            return Some(v.into());
        }

        if let Some(&v) = self.function_map(db).get(&id) {
            return Some(v.into());
        }

        let Some(ast_use) = self.ast_use_map(db).get(&id) else {
            return None;
        };

        resolve_ast_use(db, *ast_use)
    }
}

#[salsa::tracked]
fn resolve_ast_use<'db>(
    db: &'db dyn crate::Db,
    ast_use: AstUseItem<'db>,
) -> Option<NameResolution<'db>> {
    let crate_name = ast_use.crate_name(db);
    let Some(crate_source) = db.root().crate_source(db, crate_name.id) else {
        Diagnostic::error(
            db,
            crate_name.span,
            format!(
                "could not find a crate named `{}`",
                ast_use.crate_name(db).id
            ),
        )
        .label(
            db,
            Level::Error,
            crate_name.span,
            "could not find this crate",
        )
        .report(db);
        return None;
    };

    match crate_source.kind(db) {
        CrateKind::Directory(path_buf) => {
            let (item_name, module_path) = ast_use.path(db).ids.split_last().unwrap();
            if let Some((file_path, dir_path)) = module_path.split_last() {
                let mut path_buf = path_buf.clone();
                for id in dir_path {
                    path_buf.push(id.id.text(db));
                }
                path_buf.push(file_path.id.text(db));
                path_buf.set_extension("dada");

                let Some(source_file) = db.source_file(&path_buf) else {
                    Diagnostic::error(
                        db,
                        ast_use.path(db).span(db),
                        format!("could not find a file at `{}`", path_buf.display()),
                    )
                    .label(
                        db, 
                        Level::Error, 
                        ast_use.path(db).span(db), 
                        format!(
                            "this module is expected to be located at `{}`, but I could not find file with that name",
                            path_buf.display(),
                        )
                    )
                    .report(db);
                    return None;
                };

                let sym_module = source_file.symbolize(db);
                let Some(resolution) = sym_module.resolve_name(db, item_name.id) else {
                    Diagnostic::error(
                        db,
                        item_name.span,
                        format!("could not find an item  `{}`", path_buf.display()),
                    )
                    .label(
                        db, 
                        Level::Error, 
                        ast_use.path(db).span(db),
                        format!("I could find anything named `{}`", item_name.id)
                    )
                    .report(db);
                    return None;
                };

                Some(resolution)
            } else {
                todo!()
            }
        }
    }
}
