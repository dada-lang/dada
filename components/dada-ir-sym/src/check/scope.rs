use std::{borrow::Cow, fmt::Display};

use dada_ir_ast::{
    ast::{AstGenericTerm, AstPath, AstPathKind, AstUse, Identifier, SpanVec, SpannedIdentifier},
    diagnostic::{Diagnostic, Errors, Level, Reported},
    inputs::Krate,
    span::{Span, Spanned},
};
use dada_util::{FromImpls, boxed_async_fn};
use salsa::Update;
use serde::Serialize;

use crate::{
    check::{CheckTyInEnv, scope_tree::ScopeTreeNode},
    ir::{
        binder::BoundTerm,
        classes::{SymAggregate, SymAggregateStyle, SymClassMember},
        functions::SymFunction,
        module::SymModule,
        primitive::{SymPrimitive, primitives},
        types::{SymGenericKind, SymGenericTerm},
        variables::SymVariable,
    },
    prelude::Symbol,
};

use super::Env;

/// Name resolution scope, used when converting types/function-bodies etc into symbols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'scope, 'db> {
    span: Span<'db>,
    chain: ScopeChain<'scope, 'db>,
}

impl<'scope, 'db> Scope<'scope, 'db> {
    /// A base scope containing only the primitive names.
    pub(crate) fn new(db: &'db dyn crate::Db, span: Span<'db>) -> Self {
        let mut this = Scope {
            span,
            chain: ScopeChain::primitives(),
        };

        let root = db.root();
        let crate_source = root.libdada_crate(db);
        this = this.with_prelude(db, span, crate_source);

        this
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

    pub fn span(&self) -> Span<'db> {
        self.span
    }

    /// Extend this scope with the prelude from a crate.
    /// Crates can define a module named `prelude`.
    fn with_prelude(self, db: &'db dyn crate::Db, span: Span<'db>, crate_source: Krate) -> Self {
        let prelude_id = Identifier::prelude(db);
        match resolve_name_against_crate(
            db,
            crate_source,
            SpannedIdentifier {
                id: prelude_id,
                span,
            },
        ) {
            Ok(NameResolutionSym::SymModule(sym)) => self.with_link(ScopeChainKind::SymModule(sym)),
            Ok(sym) => {
                let span = sym.span(db).unwrap_or(span);
                Diagnostic::error(db, span, "prelude is not a module".to_string())
                    .label(
                        db,
                        Level::Error,
                        span,
                        format!(
                            "I expected `prelude` to be a module, but I found {category}",
                            category = sym.categorize(db)
                        ),
                    )
                    .report(db);
                self
            }
            Err(Reported(_)) => self,
        }
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
    pub fn aggregate(&self) -> Option<SymAggregate<'db>> {
        for link in self.chain.iter() {
            if let ScopeChainKind::SymAggr(aggr) = &link.kind {
                return Some(*aggr);
            }
        }
        None
    }

    /// Resolve identifier `id` (found at `span`) in the scope.
    /// Reports errors if nothing is found and returns `Err(Reported)`.
    pub(crate) fn resolve_name(
        &self,
        db: &'db dyn crate::Db,
        id: Identifier<'db>,
        span: Span<'db>,
    ) -> Errors<NameResolution<'db>> {
        if let Some(resolution) = self.chain.iter().find_map(|link| link.resolve_name(db, id)) {
            return Ok(resolution);
        }

        Err(
            Diagnostic::error(db, span, format!("could not find anything named `{id}`",))
                .label(
                    db,
                    Level::Error,
                    span,
                    "I could not find anything with this name :(",
                )
                .report(db),
        )
    }

    /// True if `sym` is in scope.
    pub fn generic_sym_in_scope(&self, db: &'db dyn crate::Db, sym: SymVariable<'db>) -> bool {
        self.chain.iter().any(|link| link.binds_symbol(db, sym))
    }

    /// Given a value of type `T` that was resolved against this scope,
    /// creates a bound version like `Binder<T>` or `Binder<Binder<T>>`
    /// where all variables defined in scope are contained in these binders.
    ///
    /// Each inner binding level in the output binds the symbols from one binding level
    /// in the scope. The outermost binding level in the output then binds all remaining
    /// symbols.
    ///
    /// Example: Given a scope that has three levels like
    ///
    /// * `Class` binding `[A, B]`
    /// * `Function` binding `[C, D]`
    /// * Local variables binding `[x, y]`
    ///
    /// if we produce a `Binder<'db, Binder<'db, T>>`, then the result would be
    ///
    /// * an outer `Binder<Binder<T>>` binds `[A, B, C, D]` that contains...
    ///     * an inner `Binder<T>` binding `[x, y]` that contains...
    ///         * the `T` value referencing `A`, `B`, `C`, `D`, `x`, and `y`
    ///
    /// # Panics
    ///
    /// If the target type `B` requires more binding levels than are present in scope.
    pub fn into_bound_value<B>(self, db: &'db dyn crate::Db, value: B::LeafTerm) -> B
    where
        B: BoundTerm<'db>,
    {
        // Compute all the bound variables in this scope.
        let binders = self.all_binders();

        // If the target type has more binder levels than we do, that is a bug in the caller.
        assert!(
            B::BINDER_LEVELS <= binders.len(),
            "target type has {} binder levels but the scope only has {}",
            B::BINDER_LEVELS,
            binders.len()
        );

        // Do we need to flatten any levels?
        let extra_binder_levels = binders.len() - B::BINDER_LEVELS;
        if extra_binder_levels == 0 {
            // Nope.
            return B::bind(db, &mut binders.into_iter(), value);
        }

        // Yep.
        let flattened_binder_levels = extra_binder_levels + 1;
        let outer_binder = binders
            .iter()
            .take(flattened_binder_levels)
            .flat_map(|v| v.iter().copied())
            .collect::<Vec<_>>();
        let remaining_binders = binders.into_iter().skip(flattened_binder_levels);
        let mut symbols_to_bind = std::iter::once(outer_binder).chain(remaining_binders);
        B::bind(db, &mut symbols_to_bind, value)
    }

    /// Convert `self` into a vec-of-vecs containing the bound generic symbols
    /// in outermost-to-innermost order. e.g. if you have `class[type A] { fn foo[type B]() }`,
    /// this will return `[[A], [B]]`.
    pub fn all_binders(&self) -> Vec<Vec<SymVariable<'db>>> {
        let mut vec = vec![];
        for link in self.chain.iter() {
            match &link.kind {
                ScopeChainKind::Primitives
                | ScopeChainKind::SymModule(_)
                | ScopeChainKind::SymAggr(_) => {}
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
/// in the associated [`ScopeChainKind`] and, if nothing is found, proceed to
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
    SymAggr(SymAggregate<'db>),

    /// Introduces the given symbols into scope.
    ForAll(Cow<'scope, [SymVariable<'db>]>),
}

impl<'db> From<SymVariable<'db>> for ScopeChainKind<'_, 'db> {
    fn from(sym: SymVariable<'db>) -> Self {
        ScopeChainKind::ForAll(Cow::Owned(vec![sym]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Update)]
pub(crate) struct NameResolution<'db> {
    pub generics: Vec<SymGenericTerm<'db>>,
    pub sym: NameResolutionSym<'db>,
}

impl<'db> NameResolution<'db> {
    /// Returns a string describing `self` that fits the mold "an X".
    pub fn categorize(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        self.sym.categorize(db)
    }

    /// Returns a string describing `self` that fits the mold "an X named `foo`".
    #[expect(dead_code)]
    pub fn describe(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        self.sym.describe(db)
    }

    /// Like [`NameResolutionSym::resolve_relative_id`] but also threads the
    /// generic arguments through.
    pub(crate) fn resolve_relative_id(
        self,
        db: &'db dyn crate::Db,
        id: SpannedIdentifier<'db>,
    ) -> Errors<Result<NameResolution<'db>, NameResolution<'db>>> {
        match self.sym.resolve_relative_id(db, id) {
            Ok(Ok(sym)) => Ok(Ok(NameResolution {
                generics: self.generics,
                sym,
            })),

            Ok(Err(s)) => {
                assert_eq!(self.sym, s);
                Ok(Err(self))
            }

            Err(e) => Err(e),
        }
    }

    /// Attempts to resolve generic argments like `foo[u32]`.    
    pub(crate) async fn resolve_relative_generic_args(
        mut self,
        env: &mut Env<'db>,
        generics: &SpanVec<'db, AstGenericTerm<'db>>,
    ) -> Errors<NameResolution<'db>> {
        let db = env.db();

        let expected_arguments = self.sym.expected_generic_parameters(db);
        let found_arguments = self.generics.len();
        assert!(found_arguments <= expected_arguments);
        let remaining_arguments = expected_arguments - found_arguments;

        if generics.len() > remaining_arguments {
            let extra_arguments = &generics.values[remaining_arguments..];
            let extra_span = extra_arguments
                .first()
                .unwrap()
                .span(db)
                .to(db, extra_arguments.last().unwrap().span(db));
            return Err(Diagnostic::error(
                db,
                extra_span,
                "extra generic arguments provided".to_string(),
            )
            .label(
                db,
                Level::Error,
                extra_span,
                format!(
                    "I expected to find at most {remaining_arguments} generic arguments, these are extra"
                ),
            )
        .report(db));
        }

        for v in generics.values.iter() {
            self.generics.push(v.check_in_env(env).await);
        }

        Ok(self)
    }

    /// Returns the span where the item that is being referenced was declared.
    /// Returns `None` for primitives or things that have no declaration.
    pub fn span(&self, db: &'db dyn crate::Db) -> Option<Span<'db>> {
        self.sym.span(db)
    }
}

/// Result of name resolution.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromImpls, Serialize, Update)]
#[allow(clippy::enum_variant_names)]
pub enum NameResolutionSym<'db> {
    SymModule(SymModule<'db>),
    SymAggregate(SymAggregate<'db>),
    SymFunction(SymFunction<'db>),
    SymPrimitive(SymPrimitive<'db>),
    SymVariable(SymVariable<'db>),
}

impl<'db> NameResolutionSym<'db> {
    /// Returns a string describing `self` that fits the mold "an X".
    pub fn categorize(self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
            NameResolutionSym::SymModule(_) => Box::new("a module") as Box<dyn Display + 'db>,
            NameResolutionSym::SymAggregate(_) => Box::new("a class"),
            NameResolutionSym::SymFunction(_) => Box::new("a function"),
            NameResolutionSym::SymVariable(var) => match var.kind(db) {
                SymGenericKind::Type => Box::new("a generic type"),
                SymGenericKind::Perm => Box::new("a generic permission"),
                SymGenericKind::Place => Box::new("a local variable"),
            },
            NameResolutionSym::SymPrimitive(p) => Box::new(format!("`{}`", p.name(db))),
        }
    }

    /// Attempt to resolve a single identifier;
    /// only works if `self` is a module or other "lexically resolved" name resolution.
    ///
    /// Returns `Ok(Ok(r))` if resolution succeeded.
    ///
    /// Returns `Ok(Err(self))` if resolution failed because this is not a lexically resolved result.
    /// Type checking will have to handle it.
    ///
    /// Returns error only if this was a lexically resolved name resolution and the identifier is not found.
    ///
    /// FIXME: Remove all error reporting from here and push it further up the chain,
    /// since the context of the lookup may matter to how we report the error.
    pub(crate) fn resolve_relative_id(
        self,
        db: &'db dyn crate::Db,
        id: SpannedIdentifier<'db>,
    ) -> Errors<Result<NameResolutionSym<'db>, NameResolutionSym<'db>>> {
        match self {
            NameResolutionSym::SymModule(sym_module) => match sym_module
                .resolve_name_against_definitions(db, id.id)
            {
                Some(sym) => Ok(Ok(sym)),
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

            // FIXME: When we add traits, we have to decide how we want to manage trait member lookup.
            // * Does this mean we have to merge name resolution plus type checking?
            // * Do we not support `SomeClass.TraitMember` and instead prefer `SomeTrait.Member[SomeClass]`?
            // * Do we only support `SomeClass.TraitMember` in expression contexts?
            NameResolutionSym::SymAggregate(sym_class) => {
                match sym_class.inherent_member(db, id.id) {
                    Some(class_member) => match class_member {
                        SymClassMember::SymFunction(sym) => Ok(Ok(sym.into())),

                        // FIXME: we should probably have a NameResolutionSym::Field?
                        SymClassMember::SymField(_) => Ok(Err(self)),
                    },
                    None => Ok(Err(self)),
                }
            }

            _ => Ok(Err(self)),
        }
    }

    /// Returns a string describing `self` that fits the mold "an X named `foo`".
    pub fn describe(self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
            NameResolutionSym::SymModule(sym_module) => {
                format!("a module named `{}`", sym_module.name(db))
            }
            NameResolutionSym::SymAggregate(sym_class) => {
                format!("a class named `{}`", sym_class.name(db))
            }
            NameResolutionSym::SymFunction(sym_function) => {
                format!("a function named `{}`", sym_function.name(db))
            }
            NameResolutionSym::SymVariable(var) => match var.name(db) {
                Some(n) => format!("{} named `{n}`", self.categorize(db)),
                None => "an anonymous generic parameter".to_string(),
            },
            NameResolutionSym::SymPrimitive(sym_primitive) => {
                format!("the primitive type `{}`", sym_primitive.name(db))
            }
        }
    }

    fn expected_generic_parameters(&self, db: &'db dyn crate::Db) -> usize {
        match self {
            NameResolutionSym::SymModule(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymAggregate(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymFunction(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymPrimitive(_) => 0,
            NameResolutionSym::SymVariable(_) => 0,
        }
    }

    fn span(&self, db: &'db dyn crate::Db) -> Option<Span<'db>> {
        match self {
            NameResolutionSym::SymModule(sym) => Some(sym.span(db)),
            NameResolutionSym::SymAggregate(sym) => Some(sym.span(db)),
            NameResolutionSym::SymFunction(sym) => Some(sym.span(db)),
            NameResolutionSym::SymPrimitive(_) => None,
            NameResolutionSym::SymVariable(sym) => Some(sym.span(db)),
        }
    }

    /// If this symbol references an aggregate (class, struct, etc) returns the
    /// aggregate style. Else returns `None`.
    pub fn style(self, db: &'db dyn crate::Db) -> Option<SymAggregateStyle> {
        match self {
            NameResolutionSym::SymModule(_) => None,
            NameResolutionSym::SymAggregate(aggr) => Some(aggr.style(db)),
            NameResolutionSym::SymFunction(_) => None,
            NameResolutionSym::SymPrimitive(_) => None,
            NameResolutionSym::SymVariable(_) => None,
        }
    }
}

/// Partial name resolution: This simply extracts what symbol has been named by the
/// user in a path. It can by synchronous and only requires a scope, not a type checking
/// environment. This is used when creating default permissions, as we want to be able
/// to do that before type checking has truly begun.
pub trait ResolveToSym<'db> {
    fn resolve_to_sym(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolutionSym<'db>>;
}

impl<'db> ResolveToSym<'db> for AstPath<'db> {
    fn resolve_to_sym(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolutionSym<'db>> {
        // This is "similar but different" to the code for `AstPath::resolve_in`.
        // This code ignores generic arguments, but they are otherwise the same.
        match self.kind(db) {
            AstPathKind::Identifier(first_id) => first_id.resolve_to_sym(db, scope),
            AstPathKind::GenericArgs { path, args: _ } => path.resolve_to_sym(db, scope),
            AstPathKind::Member { path, id } => {
                let base = path.resolve_to_sym(db, scope)?;
                match base.resolve_relative_id(db, *id)? {
                    Ok(r) => Ok(r),
                    Err(base) => Err(report_path_referencing_field(db, id, base)),
                }
            }
        }
    }
}

/// Reports an error if the user gave a path like `Foo.Bar` and `Foo` wound up being a class
/// that doesn't have nested items.
fn report_path_referencing_field<'db>(
    db: &'db dyn crate::Db,
    id: &SpannedIdentifier<'_>,
    base: NameResolutionSym<'_>,
) -> Reported {
    Diagnostic::error(db, id.span, "unexpected `.` in path")
        .label(
            db,
            Level::Error,
            id.span,
            format!(
                "I don't know how to interpret `.` applied to {} here",
                base.categorize(db),
            ),
        )
        .report(db)
}

/// Full name resolution: This requires converting generic arguments into symbols
/// which entails some amount of type checking and interacting with the environment.
/// This is therefore an `async` function.
pub trait Resolve<'db> {
    async fn resolve_in(self, env: &mut Env<'db>) -> Errors<NameResolution<'db>>;
}

impl<'db> Resolve<'db> for AstPath<'db> {
    /// Given a path that must resolve to some kind of name resolution,
    /// resolve it if we can (reporting errors if it is invalid).
    #[boxed_async_fn]
    async fn resolve_in(self, env: &mut Env<'db>) -> Errors<NameResolution<'db>> {
        // This is "similar but different" to the code for `AstPath::resolve_to_sym`.
        // That code ignores generic arguments, but they are otherwise the same.
        let db = env.db();
        match self.kind(db) {
            AstPathKind::Identifier(first_id) => first_id.resolve_in(env).await,
            AstPathKind::GenericArgs { path, args } => {
                let base = path.resolve_in(env).await?;
                base.resolve_relative_generic_args(env, args).await
            }
            AstPathKind::Member { path, id } => {
                let base = path.resolve_in(env).await?;
                match base.resolve_relative_id(db, *id)? {
                    Ok(r) => Ok(r),
                    Err(base) => Err(report_path_referencing_field(db, id, base.sym)),
                }
            }
        }
    }
}

impl<'db> ResolveToSym<'db> for SpannedIdentifier<'db> {
    fn resolve_to_sym(
        self,
        db: &'db dyn crate::Db,
        scope: &Scope<'_, 'db>,
    ) -> Errors<NameResolutionSym<'db>> {
        let NameResolution { sym, generics: _ } = scope.resolve_name(db, self.id, self.span)?;
        Ok(sym)
    }
}

impl<'db> Resolve<'db> for SpannedIdentifier<'db> {
    async fn resolve_in(self, env: &mut Env<'db>) -> Errors<NameResolution<'db>> {
        env.scope.resolve_name(env.db(), self.id, self.span)
    }
}

impl<'scope, 'db> ScopeChain<'scope, 'db> {
    /// Creates the base of the name resolution chain (primitive types).
    fn primitives() -> Self {
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

            ScopeChainKind::SymAggr(_) => None,

            ScopeChainKind::SymModule(sym) => {
                // Somewhat subtle: we give definitions precedence over uses. If the same name appears
                // in both locations, an error is reported by checking.

                if let Some(sym) = sym.resolve_name_against_definitions(db, id) {
                    match sym {
                        NameResolutionSym::SymModule(sym) => {
                            Some(self.internal_module_item(db, sym))
                        }
                        NameResolutionSym::SymAggregate(sym) => {
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
                } else {
                    sym.resolve_name_against_uses(db, id)
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

    /// Resolve an identifier like `x` that we mapped to some item in a module.
    fn internal_module_item(
        &self,
        _db: &'db dyn crate::Db,
        sym: impl ScopeTreeNode<'db> + Into<NameResolutionSym<'db>> + Copy,
    ) -> NameResolution<'db> {
        NameResolution {
            // No generic arguments have been provided yet.
            generics: vec![],
            sym: sym.into(),
        }
    }

    fn binds_symbol(&self, _db: &'db dyn crate::Db, sym: SymVariable<'db>) -> bool {
        match &self.kind {
            ScopeChainKind::SymAggr(_)
            | ScopeChainKind::Primitives
            | ScopeChainKind::SymModule(_) => false,

            ScopeChainKind::ForAll(symbols) => symbols.contains(&sym),
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
        let ast_use = self.ast_use_map(db).get(&id)?;
        resolve_ast_use(db, *ast_use)
    }
}

#[salsa::tracked]
fn resolve_ast_use<'db>(
    db: &'db dyn crate::Db,
    ast_use: AstUse<'db>,
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

    ast_use
        .path(db)
        .resolve_against(db, |id| resolve_name_against_crate(db, crate_source, id))
}

fn resolve_name_against_crate<'db>(
    db: &'db dyn crate::Db,
    krate: Krate,
    id: SpannedIdentifier<'db>,
) -> Errors<NameResolutionSym<'db>> {
    let source_file = db.source_file(krate, &[id.id]);
    match source_file.contents(db) {
        Ok(_) => {
            let sym_module = source_file.symbol(db);
            Ok(sym_module.into())
        }

        Err(message) => Err(Diagnostic::new(db, Level::Error, id.span(db), message).report(db)),
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
