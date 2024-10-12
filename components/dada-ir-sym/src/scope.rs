use std::{borrow::Cow, fmt::Display};

use dada_ir_ast::{
    ast::{AstModule, AstPath, AstUseItem, Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Errors, Level},
    inputs::CrateKind,
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymClass,
    function::{SignatureSource, SignatureSymbols, SymFunction, SymInputOutput},
    indices::{SymBinderIndex, SymBoundVarIndex, SymVarIndex},
    module::SymModule,
    prelude::IntoSymbol,
    primitive::{primitives, SymPrimitive},
    symbol::{SymGeneric, SymLocalVariable},
    ty::{Binder, GenericIndex, SymTy},
};

/// A `ScopeItem` defines a name resolution scope.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update, FromImpls)]
pub enum ScopeItem<'db> {
    Module(AstModule<'db>),
    Class(SymClass<'db>),
}

/// Name resolution scope, used when converting types/function-bodies etc into symbols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'scope, 'db> {
    chain: ScopeChain<'scope, 'db>,
}

/// A step the scope resolution chain. We first attempt to resolve an identifier
/// in the associated [`ScopeChainLink`][] and, if nothing is found, proceed to
/// the next next.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopeChain<'scope, 'db> {
    link: ScopeChainLink<'scope, 'db>,
    next: Option<Box<ScopeChain<'scope, 'db>>>,
}

/// A link the scope resolution chain.
#[derive(Clone, Debug, PartialEq, Eq, FromImpls)]
pub enum ScopeChainLink<'scope, 'db> {
    #[no_from_impl]
    Primitives,
    SymModule(SymModule<'db>),
    SignatureSymbols(Cow<'scope, SignatureSymbols<'db>>),
    LocalVariable(SymLocalVariable<'db>),

    #[no_from_impl]
    Body,
}

/// Result of name resolution.
#[derive(Clone, Debug, PartialEq, Eq, FromImpls)]
pub enum NameResolution<'db> {
    SymModule(SymModule<'db>),
    SymClass(SymClass<'db>),
    SymLocalVariable(SymLocalVariable<'db>),
    SymFunction(SymFunction<'db>),
    SymPrimitive(SymPrimitive<'db>),

    #[no_from_impl]
    SymGeneric(SymGeneric<'db>, GenericIndex),
}

/// Tracks number of binders traversed during name resolution.
/// Used to create a [`GenericIndex`][].
#[derive(Copy, Clone, Debug)]
enum BindersTraversed {
    /// Tracks the number of binders traversed.
    Bound(usize),

    /// Counts *down* from the total free (universal)
    /// generic variables in scope to 0.
    Free(usize),
}

impl<'db> ScopeItem<'db> {
    /// Convert this scope item into a scope in whatever way makes sense.
    pub fn into_scope(self, db: &'db dyn crate::Db) -> Scope<'db, 'db> {
        match self {
            ScopeItem::Module(ast_module) => ast_module.into_symbol(db).mod_scope(db),

            ScopeItem::Class(sym_class) => sym_class.class_scope(db),
        }
    }
}

impl<'scope, 'db> Scope<'scope, 'db> {
    /// A base scope containing only the primitive names.
    pub(crate) fn new(_db: &'db dyn crate::Db) -> Self {
        Scope {
            chain: ScopeChain {
                link: ScopeChainLink::Primitives,
                next: None,
            },
        }
    }

    /// Extend this scope with another link in the name resolution chain
    pub(crate) fn with_link<'scope1>(
        self,
        link: impl Into<ScopeChainLink<'scope1, 'db>>,
    ) -> Scope<'scope1, 'db>
    where
        'scope: 'scope1,
    {
        Scope {
            chain: ScopeChain {
                link: link.into(),
                next: Some(Box::new(self.chain)),
            },
        }
    }

    /// Extend this scope with another link in the name resolution chain
    pub fn push_link(&mut self, link: impl Into<ScopeChainLink<'scope, 'db>>) {
        let chain = ScopeChain {
            link: link.into(),
            next: None,
        };
        let prev_chain = std::mem::replace(&mut self.chain, chain);
        self.chain.next = Some(Box::new(prev_chain));
    }

    /// Ensures that we have at least 1 of binders, adding a dummy binding level if needed.
    pub fn ensure_binder(self) -> Scope<'scope, 'db> {
        let binders: usize = self.chain.links().map(|link| link.count_binders()).sum();
        if binders == 1 {
            self
        } else if binders == 0 {
            self.with_link(Cow::Owned(SignatureSymbols {
                source: SignatureSource::Dummy,
                generics: vec![],
                inputs: vec![],
            }))
        } else {
            panic!("ensure_binder called with {binders} level of binders, expected 0 or 1")
        }
    }

    /// Adds a "body" link to the scope chain. Used when resolving function bodies.
    pub fn with_body_link(self) -> Scope<'scope, 'db> {
        self.with_link(ScopeChainLink::Body)
    }

    /// Return the innermost class in scope along with its in-scope generic parameters.
    pub fn class(&self) -> Option<SymClass<'db>> {
        for link in self.chain.links() {
            if let ScopeChainLink::SignatureSymbols(cow) = link {
                if let SignatureSource::Class(c) = cow.source {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Resolve identifier `id` (found at `span`) in the scope.
    /// Reports errors if nothing is found and returns `Err(Reported)`.
    pub fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
        span: Span<'db>,
    ) -> Errors<NameResolution<'db>> {
        // First search the links in the scope chain
        if let Some(r) = self.chain.resolve(
            |link, binders_traversed| link.resolve_name(db, id, binders_traversed),
            BindersTraversed::Bound(0),
        ) {
            return Ok(r);
        }

        // Now search the standard prelude

        Err(
            Diagnostic::error(db, span, format!("could not find anything named `{}`", id,))
                .label(
                    db,
                    Level::Error,
                    span,
                    "I could not find anything with this name :(",
                )
                .report(db),
        )
    }

    /// Find a generic symbol in the scope and returns its name resolution.
    ///
    /// # Panics
    ///
    /// If the symbol is not in the scope.
    pub fn resolve_generic_sym(
        &self,
        db: &'db dyn crate::Db,
        sym: SymGeneric<'db>,
    ) -> NameResolution<'db> {
        match self.chain.resolve(
            |link, binders_traversed| {
                link.resolve_generic_sym(db, |sym1| *sym1 == sym, binders_traversed)
            },
            BindersTraversed::Bound(0),
        ) {
            Some(v) => v,
            None => panic!(
                "symbole `{:?}` not found in scope: {:#?}",
                sym.name(db),
                self
            ),
        }
    }

    /// Given a value of type `T`, creates a bound version like `Binder<T>`,
    /// using the binders in scope.
    ///
    /// # Panics
    ///
    /// If the result type does not match the numer of binders in scope.
    pub(crate) fn into_bound<T, B>(self, db: &'db dyn crate::Db, value: T) -> B
    where
        B: Bind<'db, T>,
    {
        let generics = self.into_bound_generics();
        B::bind(db, generics, value)
    }

    /// Convert `self` into a vec-of-vecs containing the bound generic symbols
    /// in outermost-to-innermost order. e.g. if you have `class[type A] { fn foo[type B]() }`,
    /// this will return `[[A], [B]]`.
    fn into_bound_generics(self) -> Vec<Vec<SymGeneric<'db>>> {
        let mut vec = vec![];
        for link in self.chain.into_links() {
            match link {
                ScopeChainLink::LocalVariable(_)
                | ScopeChainLink::Primitives
                | ScopeChainLink::SymModule(_) => {}
                ScopeChainLink::SignatureSymbols(cow) => {
                    vec.push(cow.into_owned().generics);
                }
                ScopeChainLink::Body => {
                    panic!("cannot create binding levels inside of body")
                }
            }
        }
        vec.reverse();
        vec
    }
}

/// Trait for creating `Binder<T>` instances.
/// Panics if the number of binders statically expected is not what we find in the scope.
pub(crate) trait Bind<'db, T> {
    /// Create `Self` from:
    ///
    /// * iterator over the remaining symbols in scope
    /// * innermost bound value `value`
    ///
    /// This either returns `value` *or* creates a `Binder<_>` around value
    /// (possibly multiple binders).
    fn bind(
        db: &'db dyn crate::Db,
        binding_levels: impl IntoIterator<Item = Vec<SymGeneric<'db>>>,
        value: T,
    ) -> Self;
}

impl<'db, T, U> Bind<'db, T> for Binder<U>
where
    U: Bind<'db, T> + Update,
    T: Update,
{
    fn bind(
        db: &'db dyn crate::Db,
        binding_levels: impl IntoIterator<Item = Vec<SymGeneric<'db>>>,
        value: T,
    ) -> Self {
        let mut binding_levels = binding_levels.into_iter();

        // Extract next level of bound symbols for use in this binder;
        // if this unwrap fails, user gave wrong number of `Binder<_>` types
        // for the scope.
        let symbols = binding_levels.next().unwrap();

        // Introduce whatever binders are needed to go from the innermost
        // value type `T` to `U`.
        let u = U::bind(db, binding_levels, value);
        Binder {
            kinds: symbols.iter().map(|s| s.kind(db)).collect(),
            bound_value: u,
        }
    }
}

impl<'db> Bind<'db, SymInputOutput<'db>> for SymInputOutput<'db> {
    fn bind(
        _db: &'db dyn crate::Db,
        binding_levels: impl IntoIterator<Item = Vec<SymGeneric<'db>>>,
        value: Self,
    ) -> Self {
        // Leaf case: symbol type is the innermost value.
        let mut binding_levels = binding_levels.into_iter();
        assert_eq!(binding_levels.next(), None);
        value
    }
}

impl<'db> Bind<'db, SymTy<'db>> for SymTy<'db> {
    fn bind(
        _db: &'db dyn crate::Db,
        binding_levels: impl IntoIterator<Item = Vec<SymGeneric<'db>>>,
        value: Self,
    ) -> Self {
        // Leaf case: symbol type is the innermost value.
        let mut binding_levels = binding_levels.into_iter();
        assert_eq!(binding_levels.next(), None);
        value
    }
}

pub trait Resolve<'db> {
    fn resolve_in(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolution<'db>>;
}

impl<'db> Resolve<'db> for AstPath<'db> {
    /// Given a path that must resolve to some kind of name resolution,
    /// resolve it if we can (reporting errors if it is invalid).
    fn resolve_in(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolution<'db>> {
        let (first_id, other_ids) = self.ids(db).split_first().unwrap();

        let resolution = first_id.resolve_in(db, scope)?;
        let (r, remaining_ids) = resolution.resolve_relative(db, other_ids)?;

        match remaining_ids.first() {
            None => Ok(r),
            Some(next_id) => Err(
                Diagnostic::error(db, next_id.span, "unexpected `.` in path")
                    .label(
                        db,
                        Level::Error,
                        next_id.span,
                        "I don't know how to interpret `.` applied to a local variable here",
                    )
                    .report(db),
            ),
        }
    }
}

impl<'db> Resolve<'db> for SpannedIdentifier<'db> {
    fn resolve_in(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolution<'db>> {
        scope.resolve_name(db, self.id, self.span)
    }
}

impl<'db> NameResolution<'db> {
    /// Returns a string describing `self` that fits the mold "an X".
    pub fn categorize(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
            NameResolution::SymModule(_) => Box::new("a module") as Box<dyn Display + 'db>,
            NameResolution::SymClass(_) => Box::new("a class"),
            NameResolution::SymLocalVariable(_) => Box::new("a local variable"),
            NameResolution::SymFunction(_) => Box::new("a function"),
            NameResolution::SymGeneric(_, _) => Box::new("a generic parameter"),
            NameResolution::SymPrimitive(p) => Box::new(format!("`{}`", p.name(db))),
        }
    }

    /// Returns a string describing `self` that fits the mold "an X named `foo`".
    pub fn describe(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
            NameResolution::SymModule(sym_module) => {
                format!("a module named `{}`", sym_module.name(db))
            }
            NameResolution::SymClass(sym_class) => {
                format!("a class named `{}`", sym_class.name(db))
            }
            NameResolution::SymLocalVariable(sym_local_variable) => {
                format!("a local variable named `{}`", sym_local_variable.name(db))
            }
            NameResolution::SymFunction(sym_function) => {
                format!("a function named `{}`", sym_function.name(db))
            }
            NameResolution::SymGeneric(sym_generic, _) => match sym_generic.name(db) {
                Some(n) => format!("a generic parameter named `{n}`"),
                None => format!("an anonymous generic parameter"),
            },
            NameResolution::SymPrimitive(sym_primitive) => {
                format!("the primitive type `{}`", sym_primitive.name(db))
            }
        }
    }

    /// Attempt to resolve `ids` relative to `self`.
    /// Continues so long as `self` is a module.
    /// Once it reaches a non-module, stops and returns the remaining entries (if any).
    /// Errors if `self` is a module but the next id in `ids` is not found.
    pub(crate) fn resolve_relative(
        self,
        db: &'db dyn crate::Db,
        ids: &'db [SpannedIdentifier<'db>],
    ) -> Errors<(NameResolution<'db>, &'db [SpannedIdentifier<'db>])> {
        let Some((next_id, other_ids)) = ids.split_first() else {
            return Ok((self, &[]));
        };

        match self {
            NameResolution::SymModule(sym_module) => {
                match sym_module.resolve_name(db, next_id.id) {
                    Some(r) => r.resolve_relative(db, other_ids),
                    None => Err(Diagnostic::error(
                        db,
                        next_id.span,
                        "nothing named `{}` found in module",
                    )
                    .label(
                        db,
                        Level::Error,
                        next_id.span,
                        format!(
                            "I could not find anything named `{}` in the module `{}`",
                            next_id.id,
                            sym_module.name(db),
                        ),
                    )
                    .report(db)),
                }
            }
            _ => Ok((self, ids)),
        }
    }
}

impl<'scope, 'db> ScopeChain<'scope, 'db> {
    /// Iterate over links starting at `self`, starting with the innermost.
    pub fn links(&self) -> impl Iterator<Item = &ScopeChainLink<'scope, 'db>> {
        let mut p = Some(self);

        std::iter::from_fn(move || match p.take() {
            Some(q) => {
                if let Some(n) = &q.next {
                    p = Some(n);
                } else {
                    p = None;
                }

                Some(&q.link)
            }
            None => None,
        })
    }

    /// Convert the chain starting at `self` into an iterator of each link,
    /// starting from the innermost.
    pub fn into_links(self) -> impl Iterator<Item = ScopeChainLink<'scope, 'db>> {
        let mut p = Some(Box::new(self));

        std::iter::from_fn(move || match p.take() {
            Some(q) => {
                p = q.next;
                Some(q.link)
            }
            None => None,
        })
    }

    /// Walk the chain to resolve an id, generic symbol, or other name lookup key.
    /// Tracks the binders that we have traversed to help in creating the [`GenericIndex`][] that identifies a generic variable.
    fn resolve(
        &self,
        resolve_link: impl Fn(&ScopeChainLink<'_, 'db>, BindersTraversed) -> Option<NameResolution<'db>>,
        binders_traversed: BindersTraversed,
    ) -> Option<NameResolution<'db>> {
        if let Some(r) = resolve_link(&self.link, binders_traversed) {
            return Some(r);
        }

        // Walk outward by one scope
        if let Some(next) = &self.next {
            // Update `binders_traversed` based on what kind of link we are stepping through...
            let next_binders_traversed = match &self.link {
                // Primitives/modules do not bind anything
                ScopeChainLink::LocalVariable(_)
                | ScopeChainLink::Primitives
                | ScopeChainLink::SymModule(_) => binders_traversed,

                // Introduce a binding level
                ScopeChainLink::SignatureSymbols(cow) => match binders_traversed {
                    BindersTraversed::Bound(binders) => BindersTraversed::Bound(binders + 1),
                    BindersTraversed::Free(variables) => {
                        BindersTraversed::Free(variables - cow.generics.len())
                    }
                },

                // Convert to free variables
                ScopeChainLink::Body => {
                    BindersTraversed::Free(self.links().map(|l| l.count_generic_variables()).sum())
                }
            };

            next.resolve(resolve_link, next_binders_traversed)
        } else {
            None
        }
    }
}

impl<'db> ScopeChainLink<'_, 'db> {
    /// Count the variables introduced by `self`
    fn count_generic_variables(&self) -> usize {
        match self {
            ScopeChainLink::LocalVariable(_)
            | ScopeChainLink::Primitives
            | ScopeChainLink::SymModule(_) => 0,
            ScopeChainLink::SignatureSymbols(cow) => cow.generics.len(),
            ScopeChainLink::Body => 0,
        }
    }

    /// Count the binders introduced by `self`
    fn count_binders(&self) -> usize {
        match self {
            ScopeChainLink::LocalVariable(_)
            | ScopeChainLink::Primitives
            | ScopeChainLink::SymModule(_) => 0,
            ScopeChainLink::SignatureSymbols(_) => 1,
            ScopeChainLink::Body => 0,
        }
    }

    fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
        binders_traversed: BindersTraversed,
    ) -> Option<NameResolution<'db>> {
        match self {
            ScopeChainLink::Primitives => primitives(db)
                .iter()
                .copied()
                .filter(|p| p.name(db) == id)
                .map(|p| p.into())
                .next(),
            ScopeChainLink::SymModule(sym_module) => sym_module.resolve_name(db, id),
            ScopeChainLink::SignatureSymbols(symbols) => {
                let SignatureSymbols {
                    source: _,
                    generics: _,
                    inputs,
                } = &**symbols;
                if let Some(resolution) =
                    self.resolve_generic_sym(db, |g| g.name(db) == Some(id), binders_traversed)
                {
                    Some(resolution)
                } else if let Some(input) = inputs.iter().find(|i| i.name(db) == id) {
                    Some(NameResolution::SymLocalVariable(*input))
                } else {
                    None
                }
            }
            ScopeChainLink::Body => None,
            &ScopeChainLink::LocalVariable(sym_local_variable) => {
                if sym_local_variable.name(db) == id {
                    Some(sym_local_variable.into())
                } else {
                    None
                }
            }
        }
    }

    fn resolve_generic_sym(
        &self,
        _db: &'db dyn crate::Db,
        test: impl Fn(&SymGeneric<'db>) -> bool,
        binders_traversed: BindersTraversed,
    ) -> Option<NameResolution<'db>> {
        match self {
            ScopeChainLink::LocalVariable(_)
            | ScopeChainLink::Primitives
            | ScopeChainLink::SymModule(_) => None,
            ScopeChainLink::SignatureSymbols(symbols) => {
                let SignatureSymbols {
                    source: _,
                    generics,
                    inputs: _,
                } = &**symbols;
                if let Some(generic) = generics.iter().position(test) {
                    let sym = generics[generic];
                    let index = binders_traversed.generic_index(generic, generics.len());
                    Some(NameResolution::SymGeneric(sym, index))
                } else {
                    None
                }
            }
            ScopeChainLink::Body => None,
        }
    }
}

impl BindersTraversed {
    fn generic_index(self, variable_index: usize, variables_in_binder: usize) -> GenericIndex {
        match self {
            BindersTraversed::Bound(binders) => GenericIndex::Bound(
                SymBinderIndex::from(binders),
                SymBoundVarIndex::from(variable_index),
            ),
            BindersTraversed::Free(variables) => GenericIndex::Universal(SymVarIndex::from(
                variables - variables_in_binder + variable_index,
            )),
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
            let (item_name, module_path) = ast_use.path(db).ids(db).split_last().unwrap();
            if let Some((file_path, dir_path)) = module_path.split_last() {
                let mut path_buf = path_buf.clone();
                for id in dir_path {
                    path_buf.push(id.id.text(db));
                }
                path_buf.push(file_path.id.text(db));
                path_buf.set_extension("dada");

                let source_file = db.source_file(&path_buf);
                let sym_module = source_file.into_symbol(db);
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
                        format!("I could find anything named `{}`", item_name.id),
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
