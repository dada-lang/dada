use std::{borrow::Cow, cmp::min, fmt::Display};

use dada_ir_ast::{
    ast::{AstPath, AstPathKind, AstUseItem, Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Errors, Level},
    inputs::CrateKind,
    span::{Span, Spanned},
};
use dada_util::{FromImpls, Map};
use salsa::Update;

use crate::{
    binder::Binder,
    class::SymClass,
    function::{SymFunction, SymInputOutput},
    indices::{SymBinderIndex, SymBoundVarIndex},
    module::SymModule,
    prelude::IntoSymbol,
    primitive::{primitives, SymPrimitive},
    scope_tree::ScopeTreeNode,
    subst::Subst,
    symbol::{SymGenericKind, SymVariable},
    ty::{FromVar, SymGenericTerm, SymTy, Var},
};

/// Name resolution scope, used when converting types/function-bodies etc into symbols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'scope, 'db> {
    chain: ScopeChain<'scope, 'db>,
}
impl<'scope, 'db> Scope<'scope, 'db> {
    /// A base scope containing only the primitive names.
    pub(crate) fn new(_db: &'db dyn crate::Db) -> Self {
        Scope {
            chain: ScopeChain::new(),
        }
    }

    /// Extend this scope with another link in the name resolution chain
    pub(crate) fn with_link<'scope1>(
        self,
        link: impl Into<ScopeChainKind<'scope1, 'db>>,
    ) -> Scope<'scope1, 'db>
    where
        'scope: 'scope1,
    {
        let mut this: Scope<'scope1, 'db> = self;
        this.push_link(link);
        this
    }

    /// Extend this scope with another link in the name resolution chain
    pub fn push_link(&mut self, kind: impl Into<ScopeChainKind<'scope, 'db>>) {
        let chain = ScopeChain {
            kind: kind.into(),
            next: None,
        };
        let prev_chain = std::mem::replace(&mut self.chain, chain);
        self.chain.next = Some(Box::new(prev_chain));
    }

    /// Return the innermost class in scope (if any).
    pub fn class(&self) -> Option<SymClass<'db>> {
        for link in self.chain.iter() {
            if let ScopeChainKind::SymClass(class) = &link.kind {
                return Some(*class);
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
        if let Some(resolution) = self.chain.iter().find_map(|link| link.resolve_name(db, id)) {
            return Ok(resolution);
        }

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
    pub fn resolve_generic_sym(&self, db: &'db dyn crate::Db, sym: SymVariable<'db>) -> Var<'db> {
        if let Some(r) = self
            .chain
            .iter()
            .find_map(|link| link.resolve_symbol(db, sym))
        {
            return r;
        }

        panic!(
            "symbol `{:?}` not found in scope: {:#?}",
            sym.name(db),
            self
        )
    }

    /// Given a value of type `T` that was resolved against this scope,
    /// creates a bound version like `Binder<T>`. In the process it
    /// replaces the "free variable" references within `T` bound variables.
    /// `T` must only contain free variables that arose from this scope.
    ///
    /// The number of binders created is determined by the result type `B`.
    /// This function removes binder levels from the chain
    /// corresponding to the number of binders in `B`. If `B` has more binders
    /// than are present in our chain, then the extra outermost binders in `B`
    /// are created as empty binders.
    ///
    /// Callers that expect to pop *all* binder levels should use [`into_binders`][]
    /// instead.
    pub(crate) fn pop_binders<T, B>(&mut self, db: &'db dyn crate::Db, value: T) -> B
    where
        B: Binders<'db, T>,
    {
        let mut binders = self.all_binders();

        // The number of binding levels in `B` may not match the number that are in scope.
        //
        // If `B` contains *fewer* binders than the scope, then we will leave variables from
        // the outermost binders as free variables.
        //
        // If `B` contains *more* binders than the scope, then we will pad it with extra empty
        // binders later.
        let num_skipped_binders = binders.len().saturating_sub(B::BINDER_LEVELS);
        let num_popped_binders = min(binders.len(), B::BINDER_LEVELS);
        let num_extra_binders = B::BINDER_LEVELS - binders.len();

        // Pad `binders` with extra binders if needed.
        if num_extra_binders > 0 {
            assert_eq!(num_skipped_binders, 0);
            binders = (0..num_extra_binders)
                .map(|_| vec![])
                .chain(binders)
                .collect();
        }

        // Compute a vector that contains the substitution (if any) for each
        // free variable that could appear in `value` (all of which are assumed
        // to have come from this scope).
        let free_var_substitution: Map<SymVariable<'db>, SymGenericTerm<'db>> = {
            // Variables in the binders to be popped will be replaced by a bound var.
            // Given `[[A, B], [C, D, E]]`, we will create variables like
            // `[^1.0, ^1.1, ^0.0, ^0.1, ^0.2]`, where `^0` and `^1` indicate
            // binder indices, with `^0` representing the innermost binder.
            binders
                .iter()
                .zip(0..)
                .skip(num_skipped_binders)
                .flat_map(|(binder_vars, binder_index)| {
                    let binder_index = SymBinderIndex::from(binders.len() - binder_index - 1);
                    binder_vars.iter().copied().zip(0..).map(move |(v, i)| {
                        let bound_index = SymBoundVarIndex::from(i);
                        let generic_index = Var::Bound(binder_index, bound_index);
                        (v, SymGenericTerm::var(db, v.kind(db), generic_index))
                    })
                })
                .collect()
        };

        let result = B::bind(
            db,
            binders.into_iter().skip(num_skipped_binders),
            &free_var_substitution,
            value,
        );

        // Pop off the binders we need to pop.
        let chain = std::mem::replace(&mut self.chain, ScopeChain::new());
        self.chain = chain.pop_binders(num_popped_binders);

        result
    }

    /// Version of [`Self::pop_binders`][] that asserts that all binder links have been popped.
    pub(crate) fn into_bound_value<T, B>(mut self, db: &'db dyn crate::Db, value: T) -> B
    where
        B: Binders<'db, T>,
    {
        let value = self.pop_binders(db, value);
        let binder_link = self.chain.iter().find(|link| link.is_binder());
        assert!(binder_link.is_none(), "failed to pop binder link");
        value
    }

    /// Convert `self` into a vec-of-vecs containing the bound generic symbols
    /// in outermost-to-innermost order. e.g. if you have `class[type A] { fn foo[type B]() }`,
    /// this will return `[[A], [B]]`.
    fn all_binders(&self) -> Vec<Vec<SymVariable<'db>>> {
        let mut vec = vec![];
        for link in self.chain.iter() {
            match &link.kind {
                ScopeChainKind::Primitives
                | ScopeChainKind::SymModule(_)
                | ScopeChainKind::SymClass(_) => {}
                ScopeChainKind::ForAll(cow) => {
                    vec.push(cow.iter().copied().collect());
                }
            }
        }
        vec.reverse();
        vec
    }
}

/// A link in the scope resolution chain. We first attempt to resolve an identifier
/// in the associated [`ScopeChainLink`][] and, if nothing is found, proceed to
/// the next link.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopeChain<'scope, 'db> {
    /// Kind of this link.
    kind: ScopeChainKind<'scope, 'db>,

    /// Next link in the chain. Earlier links shadow later links.
    next: Option<Box<ScopeChain<'scope, 'db>>>,
}

/// A link the scope resolution chain.
#[derive(Clone, Debug, PartialEq, Eq, FromImpls)]
pub enum ScopeChainKind<'scope, 'db> {
    /// Introduces the primitives into scope (always present).
    #[no_from_impl]
    Primitives,

    /// Records that we are in the scope of a module.
    SymModule(SymModule<'db>),

    /// Records that we are in the scope of a class
    SymClass(SymClass<'db>),

    /// Introduces the given symbols into scope.
    ForAll(Cow<'scope, [SymVariable<'db>]>),
}

impl<'db> From<SymVariable<'db>> for ScopeChainKind<'_, 'db> {
    fn from(sym: SymVariable<'db>) -> Self {
        ScopeChainKind::ForAll(Cow::Owned(vec![sym]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameResolution<'db> {
    pub generics: Vec<SymGenericTerm<'db>>,
    pub sym: NameResolutionSym<'db>,
}

impl<'db> NameResolution<'db> {
    /// Returns a string describing `self` that fits the mold "an X".
    pub fn categorize(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self.sym {
            NameResolutionSym::SymModule(_) => Box::new("a module") as Box<dyn Display + 'db>,
            NameResolutionSym::SymClass(_) => Box::new("a class"),
            NameResolutionSym::SymFunction(_) => Box::new("a function"),
            NameResolutionSym::SymVariable(var) => match var.kind(db) {
                SymGenericKind::Type => Box::new("a generic type"),
                SymGenericKind::Perm => Box::new("a generic permission"),
                SymGenericKind::Place => Box::new("a local variable"),
            },
            NameResolutionSym::SymPrimitive(p) => Box::new(format!("`{}`", p.name(db))),
        }
    }

    /// Returns a string describing `self` that fits the mold "an X named `foo`".
    pub fn describe(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self.sym {
            NameResolutionSym::SymModule(sym_module) => {
                format!("a module named `{}`", sym_module.name(db))
            }
            NameResolutionSym::SymClass(sym_class) => {
                format!("a class named `{}`", sym_class.name(db))
            }
            NameResolutionSym::SymFunction(sym_function) => {
                format!("a function named `{}`", sym_function.name(db))
            }
            NameResolutionSym::SymVariable(var) => match var.name(db) {
                Some(n) => format!("{} named `{n}`", self.categorize(db)),
                None => format!("an anonymous generic parameter"),
            },
            NameResolutionSym::SymPrimitive(sym_primitive) => {
                format!("the primitive type `{}`", sym_primitive.name(db))
            }
        }
    }

    /// Attempt to resolve a singe identifier;
    /// only works if `self` is a module or other "lexically resolved" name resolution.
    ///
    /// Returns `Ok(Ok(r))` if resolution succeeded.
    ///
    /// Returns `Ok(Err(self))` if resolution failed because this is not a lexically resolved result.
    /// Type checking will have to handle it.
    ///
    /// Returns error only if this was a lexically resolved name resolution and the identifier is not found.
    pub fn resolve_relative_id(
        self,
        db: &'db dyn crate::Db,
        id: SpannedIdentifier<'db>,
    ) -> Errors<Result<NameResolution<'db>, NameResolution<'db>>> {
        match self.sym {
            NameResolutionSym::SymModule(sym_module) => match sym_module
                .resolve_name_against_definitions(db, id.id)
            {
                Some(sym) => Ok(Ok(NameResolution {
                    generics: self.generics,
                    sym,
                })),
                None => Err(
                    Diagnostic::error(db, id.span, "nothing named `{}` found in module")
                        .label(
                            db,
                            Level::Error,
                            id.span,
                            format!(
                                "I could not find anything named `{}` in the module `{}`",
                                id.id,
                                sym_module.name(db),
                            ),
                        )
                        .report(db),
                ),
            },
            _ => Ok(Err(self)),
        }
    }
}

/// Result of name resolution.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromImpls)]
pub enum NameResolutionSym<'db> {
    SymModule(SymModule<'db>),
    SymClass(SymClass<'db>),
    SymFunction(SymFunction<'db>),
    SymPrimitive(SymPrimitive<'db>),
    SymVariable(SymVariable<'db>),
}

/// Trait for creating `Binder<T>` instances.
/// Panics if the number of binders statically expected is not what we find in the scope.
pub(crate) trait Binders<'db, T> {
    const BINDER_LEVELS: usize;

    /// Create `Self` from:
    ///
    /// * iterator over the remaining symbols in scope
    /// * innermost bound value `value`
    ///
    /// This either returns `value` *or* creates a `Binder<_>` around value
    /// (possibly multiple binders).
    fn bind(
        db: &'db dyn crate::Db,
        symbols_to_bind: impl Iterator<Item = Vec<SymVariable<'db>>>,
        free_var_substitution: &Map<SymVariable<'db>, SymGenericTerm<'db>>,
        value: T,
    ) -> Self;
}

impl<'db, T, U> Binders<'db, T> for Binder<U>
where
    U: Binders<'db, T> + Update,
    T: Update,
{
    fn bind(
        db: &'db dyn crate::Db,
        mut symbols_to_bind: impl Iterator<Item = Vec<SymVariable<'db>>>,
        free_var_substitution: &Map<SymVariable<'db>, SymGenericTerm<'db>>,
        value: T,
    ) -> Self {
        // Extract next level of bound symbols for use in this binder;
        // if this unwrap fails, user gave wrong number of `Binder<_>` types
        // for the scope.
        let symbols = symbols_to_bind.next().unwrap();

        // Introduce whatever binders are needed to go from the innermost
        // value type `T` to `U`.
        let u = U::bind(db, symbols_to_bind, free_var_substitution, value);
        Binder {
            kinds: symbols.iter().map(|s| s.kind(db)).collect(),
            bound_value: u,
        }
    }

    const BINDER_LEVELS: usize = U::BINDER_LEVELS + 1;
}

impl<'db> Binders<'db, SymInputOutput<'db>> for SymInputOutput<'db> {
    fn bind(
        db: &'db dyn crate::Db,
        symbols_to_bind: impl Iterator<Item = Vec<SymVariable<'db>>>,
        free_var_substitution: &Map<SymVariable<'db>, SymGenericTerm<'db>>,
        value: Self,
    ) -> Self {
        bind_leaf(db, symbols_to_bind, free_var_substitution, value)
    }

    const BINDER_LEVELS: usize = 0;
}

impl<'db> Binders<'db, SymTy<'db>> for SymTy<'db> {
    fn bind(
        db: &'db dyn crate::Db,
        symbols_to_bind: impl Iterator<Item = Vec<SymVariable<'db>>>,
        free_var_substitution: &Map<SymVariable<'db>, SymGenericTerm<'db>>,
        value: Self,
    ) -> Self {
        bind_leaf(db, symbols_to_bind, free_var_substitution, value)
    }

    const BINDER_LEVELS: usize = 0;
}

fn bind_leaf<'db, L: Subst<'db, Output = L>>(
    db: &'db dyn crate::Db,
    mut symbols_to_bind: impl Iterator<Item = Vec<SymVariable<'db>>>,
    free_var_substitution: &Map<SymVariable<'db>, L::GenericTerm>,
    value: L,
) -> L {
    // Leaf case: symbol type is the innermost value.
    assert_eq!(symbols_to_bind.next(), None);
    value.subst_vars(db, free_var_substitution)
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
        match self.kind(db) {
            AstPathKind::Identifier(first_id) => first_id.resolve_in(db, scope),
            AstPathKind::GenericArgs { path: _, args: _ } => todo!(),
            AstPathKind::Member { path, id } => {
                let base = path.resolve_in(db, scope)?;
                match base.resolve_relative_id(db, *id)? {
                    Ok(r) => Ok(r),
                    Err(base) => Err(Diagnostic::error(db, id.span, "unexpected `.` in path")
                        .label(
                            db,
                            Level::Error,
                            id.span,
                            format!(
                                "I don't know how to interpret `.` applied to {} here",
                                base.categorize(db),
                            ),
                        )
                        .report(db)),
                }
            }
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

impl<'scope, 'db> ScopeChain<'scope, 'db> {
    /// Creates the base of the name resolution chain (primitive types).
    fn new() -> Self {
        ScopeChain {
            kind: ScopeChainKind::Primitives,
            next: None,
        }
    }

    /// Walks the chain, starting with the innermost links.
    pub fn iter(&self) -> impl Iterator<Item = &ScopeChain<'scope, 'db>> {
        let mut p = Some(self);

        std::iter::from_fn(move || match p.take() {
            Some(q) => {
                if let Some(n) = &q.next {
                    p = Some(n);
                } else {
                    p = None;
                }

                Some(q)
            }
            None => None,
        })
    }

    fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
        match &self.kind {
            ScopeChainKind::Primitives => primitives(db)
                .iter()
                .copied()
                .filter(|p| p.name(db) == id)
                .map(|p| NameResolution {
                    generics: vec![],
                    sym: p.into(),
                })
                .next(),

            ScopeChainKind::SymClass(_) => None,

            ScopeChainKind::SymModule(sym) => {
                // Somewhat subtle: we give definitions precedence over uses. If the same name appears
                // in both locations, an error is reported by checking.

                if let Some(sym) = sym.resolve_name_against_definitions(db, id) {
                    match sym {
                        NameResolutionSym::SymModule(sym) => {
                            Some(self.internal_module_item(db, sym))
                        }
                        NameResolutionSym::SymClass(sym) => {
                            Some(self.internal_module_item(db, sym))
                        }
                        NameResolutionSym::SymFunction(sym) => {
                            Some(self.internal_module_item(db, sym))
                        }
                        NameResolutionSym::SymPrimitive(_) | NameResolutionSym::SymVariable(_) => {
                            // cannot be members of a module
                            unreachable!()
                        }
                    }
                } else if let Some(resolution) = sym.resolve_name_against_uses(db, id) {
                    Some(resolution)
                } else {
                    None
                }
            }

            ScopeChainKind::ForAll(symbols) => {
                if let Some(index) = symbols.iter().position(|&s| s.name(db) == Some(id)) {
                    let sym = symbols[index];
                    Some(NameResolution {
                        generics: vec![],
                        sym: NameResolutionSym::SymVariable(sym),
                    })
                } else {
                    None
                }
            }
        }
    }

    fn internal_module_item(
        &self,
        db: &'db dyn crate::Db,
        sym: impl ScopeTreeNode<'db> + Into<NameResolutionSym<'db>> + Copy,
    ) -> NameResolution<'db> {
        NameResolution {
            generics: sym
                .transitive_generic_parameters(db)
                .iter()
                .map(|v| SymGenericTerm::var(db, v.kind(db), Var::Universal(*v)))
                .collect(),
            sym: sym.into(),
        }
    }

    fn resolve_symbol(&self, _db: &'db dyn crate::Db, sym: SymVariable<'db>) -> Option<Var<'db>> {
        match &self.kind {
            ScopeChainKind::SymClass(_)
            | ScopeChainKind::Primitives
            | ScopeChainKind::SymModule(_) => None,

            ScopeChainKind::ForAll(symbols) => {
                if symbols.iter().any(|&s| s == sym) {
                    Some(Var::Universal(sym))
                } else {
                    None
                }
            }
        }
    }

    fn is_binder(&self) -> bool {
        match &self.kind {
            ScopeChainKind::Primitives
            | ScopeChainKind::SymModule(_)
            | ScopeChainKind::SymClass(_) => false,
            ScopeChainKind::ForAll(_) => true,
        }
    }

    // Pop off binders from chain until `num_popped_binders` have been popped.
    fn pop_binders(self, num_popped_binders: usize) -> Self {
        if num_popped_binders == 0 {
            return self;
        }

        let is_binder = self.is_binder();
        let next = self.next.unwrap();
        if is_binder {
            next.pop_binders(num_popped_binders - 1)
        } else {
            next.pop_binders(num_popped_binders)
        }
    }
}

impl<'db> SymModule<'db> {
    fn resolve_name_against_definitions(
        self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolutionSym<'db>> {
        if let Some(&v) = self.class_map(db).get(&id) {
            return Some(v.into());
        }

        if let Some(&v) = self.function_map(db).get(&id) {
            return Some(v.into());
        }

        None
    }

    fn resolve_name_against_uses(
        self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
    ) -> Option<NameResolution<'db>> {
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
        CrateKind::Directory(dir_path) => ast_use.path(db).resolve_against(db, |id| {
            let module_path = dir_path.join(id.id.text(db)).with_extension("dada");
            let source_file = db.source_file(&module_path);
            match source_file.contents(db) {
                Ok(_) => {
                    let sym_module = source_file.into_symbol(db);
                    Ok(sym_module.into())
                }

                Err(message) => {
                    Err(Diagnostic::new(db, Level::Error, id.span(db), message).report(db))
                }
            }
        }),
    }
}

trait ResolveAgainst<'db> {
    fn resolve_against(
        self,
        db: &'db dyn crate::Db,
        op: impl FnOnce(SpannedIdentifier<'db>) -> Errors<NameResolutionSym<'db>>,
    ) -> Option<NameResolution<'db>>;
}

impl<'db> ResolveAgainst<'db> for AstPath<'db> {
    fn resolve_against(
        self,
        _db: &'db dyn crate::Db,
        _op: impl FnOnce(SpannedIdentifier<'db>) -> Errors<NameResolutionSym<'db>>,
    ) -> Option<NameResolution<'db>> {
        todo!()
    }
}
