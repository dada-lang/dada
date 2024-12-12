use std::{borrow::Cow, fmt::Display};

use dada_ir_ast::{
    ast::{AstGenericTerm, AstPath, AstPathKind, AstUse, Identifier, SpanVec, SpannedIdentifier},
    diagnostic::{Diagnostic, Errors, Level, Reported},
    inputs::Krate,
    span::{Span, Spanned},
};
use dada_util::FromImpls;

use crate::{
    ir::classes::{SymAggregate, SymClassMember},
    env::EnvLike,
    ir::binder::BoundTerm,
    ir::functions::SymFunction,
    ir::primitive::{primitives, SymPrimitive},
    ir::variables::{FromVar, SymVariable},
    ir::types::{SymGenericKind, SymGenericTerm, SymTy},
    ir::module::SymModule,
    prelude::Symbol,
    scope_tree::ScopeTreeNode,
    CheckInEnv,
};

/// Name resolution scope, used when converting types/function-bodies etc into symbols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'scope, 'db> {
    chain: ScopeChain<'scope, 'db>,
}
impl<'scope, 'db> Scope<'scope, 'db> {
    /// A base scope containing only the primitive names.
    pub(crate) fn new(db: &'db dyn crate::Db, span: Span<'db>) -> Self {
        let mut this = Scope {
            chain: ScopeChain::primitives(),
        };

        let root = db.root();
        if let Some(crate_source) = root.libdada_crate(db) {
            this = this.with_prelude(db, span, crate_source);
        }

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

    /// Extend this scope wit the prelude from a crate.
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
                Diagnostic::error(db, span, format!("prelude is not a module"))
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
    pub fn class(&self) -> Option<SymAggregate<'db>> {
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

    /// Create a global environment (meaning: one with no local variables, parameters, etc)
    /// from this scope. Used when resolving `use` statements, for example.
    pub fn into_global_env(self, db: &'db dyn crate::Db) -> GlobalEnv<'db>
    where
        'scope: 'db,
    {
        GlobalEnv { db, scope: self }
    }
}

/// A global environment that implements [`EnvLike`][].
/// Suitable when there are no local variables or parameters in scope.
/// Created by [`Scope::into_global_env`][].
pub struct GlobalEnv<'db> {
    db: &'db dyn crate::Db,
    scope: Scope<'db, 'db>,
}

impl<'db> EnvLike<'db> for GlobalEnv<'db> {
    fn db(&self) -> &'db dyn crate::Db {
        self.db
    }

    fn scope(&self) -> &Scope<'db, 'db> {
        &self.scope
    }

    fn variable_ty(&mut self, var: SymVariable<'db>) -> SymTy<'db> {
        unreachable!("global scope has no variables, `{var}` is not in scope")
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
    SymClass(SymAggregate<'db>),

    /// Introduces the given symbols into scope.
    ForAll(Cow<'scope, [SymVariable<'db>]>),
}

impl<'db> From<SymVariable<'db>> for ScopeChainKind<'_, 'db> {
    fn from(sym: SymVariable<'db>) -> Self {
        ScopeChainKind::ForAll(Cow::Owned(vec![sym]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn describe(&self, db: &'db dyn crate::Db) -> impl Display + 'db {
        self.sym.describe(db)
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

            // FIXME: When we add traits, we have to decide how we want to manage trait member lookup.
            // * Does this mean we have to merge name resolution plus type checking?
            // * Do we not support `SomeClass.TraitMember` and instead prefer `SomeTrait.Member[SomeClass]`?
            // * Do we only support `SomeClass.TraitMember` in expression contexts?
            NameResolutionSym::SymClass(sym_class) => match sym_class.inherent_member(db, id.id) {
                Some(class_member) => match class_member {
                    SymClassMember::SymFunction(sym) => Ok(Ok(NameResolution {
                        generics: self.generics,
                        sym: NameResolutionSym::SymFunction(sym),
                    })),

                    // FIXME: we should probably have a NameResolutionSym::Field?
                    SymClassMember::SymField(_) => Ok(Err(self)),
                },
                None => Ok(Err(self)),
            },

            _ => Ok(Err(self)),
        }
    }

    /// Attempts to resolve generic argments like `foo[u32]`.    
    pub(crate) fn resolve_relative_generic_args(
        mut self,
        env: &mut dyn EnvLike<'db>,
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
                format!(
                    "extra generic arguments provided",
                ),
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

        self.generics
            .extend(generics.values.iter().map(|v| v.check_in_env(env)));

        Ok(self)
    }

    /// Returns the span where the item that is being referenced was declared.
    /// Returns `None` for primitives or things that have no declaration.
    pub fn span(&self, db: &'db dyn crate::Db) -> Option<Span<'db>> {
        self.sym.span(db)
    }
}

/// Result of name resolution.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromImpls)]
pub enum NameResolutionSym<'db> {
    SymModule(SymModule<'db>),
    SymClass(SymAggregate<'db>),
    SymFunction(SymFunction<'db>),
    SymPrimitive(SymPrimitive<'db>),
    SymVariable(SymVariable<'db>),
}

impl<'db> NameResolutionSym<'db> {
    /// Returns a string describing `self` that fits the mold "an X".
    pub fn categorize(self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
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
    pub fn describe(self, db: &'db dyn crate::Db) -> impl Display + 'db {
        match self {
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

    fn expected_generic_parameters(&self, db: &'db dyn crate::Db) -> usize {
        match self {
            NameResolutionSym::SymModule(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymClass(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymFunction(sym) => sym.expected_generic_parameters(db),
            NameResolutionSym::SymPrimitive(_) => 0,
            NameResolutionSym::SymVariable(_) => 0,
        }
    }
}

impl<'db> NameResolutionSym<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Option<Span<'db>> {
        match self {
            NameResolutionSym::SymModule(sym) => Some(sym.span(db)),
            NameResolutionSym::SymClass(sym) => Some(sym.span(db)),
            NameResolutionSym::SymFunction(sym) => Some(sym.span(db)),
            NameResolutionSym::SymPrimitive(_) => None,
            NameResolutionSym::SymVariable(sym) => Some(sym.span(db)),
        }
    }
}

pub trait Resolve<'db> {
    fn resolve_in(self, env: &mut dyn EnvLike<'db>) -> Errors<NameResolution<'db>>;
}

impl<'db> Resolve<'db> for AstPath<'db> {
    /// Given a path that must resolve to some kind of name resolution,
    /// resolve it if we can (reporting errors if it is invalid).
    fn resolve_in(self, env: &mut dyn EnvLike<'db>) -> Errors<NameResolution<'db>> {
        let db = env.db();

        match self.kind(db) {
            AstPathKind::Identifier(first_id) => first_id.resolve_in(env),
            AstPathKind::GenericArgs { path, args } => {
                let base = path.resolve_in(env)?;
                base.resolve_relative_generic_args(env, args)
            }
            AstPathKind::Member { path, id } => {
                let base = path.resolve_in(env)?;
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
    fn resolve_in(self, env: &mut dyn EnvLike<'db>) -> Errors<NameResolution<'db>> {
        env.scope().resolve_name(env.db(), self.id, self.span)
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
                .map(|v| SymGenericTerm::var(db, *v))
                .collect(),
            sym: sym.into(),
        }
    }

    fn binds_symbol(&self, _db: &'db dyn crate::Db, sym: SymVariable<'db>) -> bool {
        match &self.kind {
            ScopeChainKind::SymClass(_)
            | ScopeChainKind::Primitives
            | ScopeChainKind::SymModule(_) => false,

            ScopeChainKind::ForAll(symbols) => symbols.iter().any(|&s| s == sym),
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
