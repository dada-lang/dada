use std::sync::Arc;

use dada_ir_ast::{
    Db,
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};
use dada_util::vecset::VecSet;

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::{
        exprs::SymExpr,
        indices::InferVarIndex,
        types::{SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyName},
        variables::SymVariable,
    },
};

use super::chains::{Chain, RedTerm, RedTy};

/// The `OrElse` trait captures error reporting context.
/// Primitive type operations like subtyping are given an `&dyn OrElse<'db>`
/// as argument. If the subtyping operation fails, it invokes the [`OrElse::report`][]
/// method to report the error.
///
/// `OrElse` objects can be converted, using the [`OrElse::to_arc`][] method,
/// into an [`ArcOrElse<'db>`][], which allows the or-else to be preserved
/// for longer than the current stack frame. This is used to store an or-else in
/// inference variable data so that, if a conflict is later generated, we can
/// extract the reason for that original constraint.
pub trait OrElse<'db> {
    /// Report the diagnostic created by [`OrElse::or_else`][].
    fn report(&self, db: &'db dyn Db, because: Because<'db>) -> Reported {
        self.or_else(db, because).report(db)
    }

    /// Create a diagnostic representing the error.
    ///
    /// The error would typically be expressed in high-level terms, like
    /// "cannot assign from `a` to `b`" or "incorrect type of function argument".
    ///
    /// The `because` argument signals the reason the low-level operation failed
    /// and will be used to provide additional details, like "`our` is not assignable to `my`".
    fn or_else(&self, db: &'db dyn Db, because: Because<'db>) -> Diagnostic;

    /// Convert a `&dyn OrElse<'db>` into an `ArcOrElse<'db>` so that it can be
    /// stored in an [`InferenceVarData`](`crate::check::inference::InferenceVarData`)
    /// or otherwise preserved beyond the current stack frame.
    /// See the trait comment for more details.
    fn to_arc(&self) -> ArcOrElse<'db>;
}

/// See [`OrElse::to_arc`][].
pub type ArcOrElse<'db> = Arc<dyn OrElse<'db> + 'db>;

impl<'db> OrElse<'db> for ArcOrElse<'db> {
    fn or_else(&self, db: &'db dyn Db, because: Because<'db>) -> Diagnostic {
        <dyn OrElse<'db>>::or_else(self, db, because)
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::clone(self)
    }
}

pub trait OrElseHelper<'db>: Sized {
    /// Create a new [`OrElse`][] that just maps the [`Because`][]
    /// value and propagates to an underlying or-else. This is used
    /// when an operating like subtyping delegates to sub-operations:
    /// the suboperation will provide a [`Because`][] value and
    /// then the subtyping can provide additional context, but the
    /// high-level 'or-else' is left unchanged.
    fn map_because(
        self,
        f: impl 'db + Clone + Fn(Because<'db>) -> Because<'db>,
    ) -> impl OrElse<'db>;
}

impl<'db> OrElseHelper<'db> for &dyn OrElse<'db> {
    /// See [`OrElseHelper::map_because`][].
    fn map_because(
        self,
        f: impl 'db + Clone + Fn(Because<'db>) -> Because<'db>,
    ) -> impl OrElse<'db> {
        struct MapBecause<F, G>(F, G);

        impl<'db, F, G> OrElse<'db> for MapBecause<F, G>
        where
            F: 'db + Clone + Fn(Because<'db>) -> Because<'db>,
            G: std::ops::Deref<Target: OrElse<'db>>,
        {
            fn or_else(&self, db: &'db dyn Db, because: Because<'db>) -> Diagnostic {
                self.1.or_else(db, (self.0)(because))
            }

            fn to_arc(&self) -> Arc<dyn OrElse<'db> + 'db> {
                Arc::new(MapBecause(self.0.clone(), self.1.to_arc()))
            }
        }

        MapBecause(f, self)
    }
}

/// Reason that a low-level typing operation failed.
pub enum Because<'db> {
    /// This type is not numeric
    NotNumeric(RedTy<'db>),

    /// `shared[place]` was required
    NotSubOfShared(SymPlace<'db>),

    /// `leased[place]` was required
    NotSubOfLeased(SymPlace<'db>),

    /// Universal variable was not declared to be `predicate` (and it must be)
    VarNotDeclaredToBe(SymVariable<'db>, Predicate),

    /// Universal variable was declared to be `predicate` (but ought not to be)
    VarDeclaredToBe(SymVariable<'db>, Predicate),

    /// The never type is not copy
    NeverIsNotCopy,

    /// The never type is not copy
    NeverIsNotLent,

    /// Classes are not copy
    ClassIsNotCopy(SymTyName<'db>),

    /// Classes are not lent
    ClassIsNotLent(SymTyName<'db>),

    /// We cannot infer otherwise because of a previous requirement
    PreviousRequirement(ArcOrElse<'db>),

    /// In an inference failure, this indicates the conflict with previously recorded requirement.
    BaseRequirement,

    /// `A B` is not lent
    ApplicationNotLent(SymGenericTerm<'db>, SymGenericTerm<'db>),

    /// Struct is not copy because some component of the given struct type is not copy
    StructComponentNotCopy(SymTyName<'db>, SymGenericTerm<'db>, Box<Because<'db>>),

    /// Struct is copy
    StructIsCopy(SymTy<'db>),

    /// Type does not have a lent component
    NoLentComponent(SymGenericTerm<'db>),

    /// Shared is copy
    SharedIsCopy(SymPerm<'db>),

    /// Leasing from a copy place yields a copy permission (which is not desired here)
    LeasedFromCopyIsCopy(SymPerm<'db>),

    /// My is move
    MyIsMove,

    /// Our is copy
    OurIsCopy,

    /// My is owned
    MyIsOwned,

    /// Our is owned
    OurIsOwned,

    /// Term could be leased
    TermCouldBeLeased(SymGenericTerm<'db>),

    /// Universal mismatch
    UniversalMismatch(SymVariable<'db>, SymVariable<'db>),

    /// Name mismatch
    NameMismatch(SymTyName<'db>, SymTyName<'db>),

    /// Not a subtype
    NotSubRedTys(RedTerm<'db>, RedTerm<'db>),

    /// The given chain was not a sub-chain of any of the upper bounds in the set
    NotSubChain(Chain<'db>, VecSet<Chain<'db>>),

    /// The given chain was not a sub-chain of any of the upper bounds
    /// that were found for this inference variable.
    NotSubChainInfer(Chain<'db>, Vec<(Chain<'db>, ArcOrElse<'db>)>),
}

impl<'db> Because<'db> {
    /// Convenience function to create a [`Because::StructComponentNotCopy`][].
    pub fn struct_component_not_copy(
        self,
        sym_ty_name: SymTyName<'db>,
        term: impl Into<SymGenericTerm<'db>>,
    ) -> Because<'db> {
        Because::StructComponentNotCopy(sym_ty_name, term.into(), Box::new(self))
    }
}

pub(super) fn report_infer_is_contradictory<'db>(
    env: &Env<'db>,
    var_index: InferVarIndex,
    predicate: Predicate,
    predicate_span: Span<'db>,
    isnt_span: Span<'db>,
) -> Reported {
    let db = env.db();
    let (var_span, var_kind) = env
        .runtime()
        .with_inference_var_data(var_index, |data| (data.span(), data.kind()));

    Diagnostic::error(
        db,
        var_span,
        format!("contradictory requirements for `{var_kind}`"),
    )
    .label(
        db,
        Level::Error,
        var_span,
        format!("I could not infer a `{var_kind}` here because it would have to be both `{predicate}` and not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        predicate_span,
        format!("required to be `{predicate}` here"),
    )
    .label(
        db,
        Level::Error,
        isnt_span,
        format!("required not to be `{predicate}` here"),
    )
    .report(db)
}

pub(super) fn report_never_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the never type (`!`) is not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("the never type (`!`) is considered `my` and therefore is not `{predicate}`"),
    )
    .report(db)
}

pub(super) fn report_term_must_not_be_leased_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(db, span, format!("the {kind} `{term}` cannot be leased"))
        .label(
            db,
            Level::Error,
            span,
            format!("the {kind} `{term}` is considered leased"),
        )
        .report(db)
}

pub(super) fn report_term_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the {kind} `{term}` is not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("I expected a `{predicate}` {kind} but I found `{term}`"),
    )
    .report(db)
}

pub(super) fn report_term_must_not_be_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the {kind} `{term}` must not be `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("I did not expect a `{predicate}` {kind} but I found `{term}`"),
    )
    .report(db)
}

pub(super) fn report_var_must_be_but_is_not_declared_to_be<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("variable `{var}` must be `{predicate}` but is not declared to be"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("variable `{var}` is not declared to be `{predicate}`"),
    )
    .report(db)
}

pub(super) fn report_var_must_not_be_declared_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("variable `{var}` must not be declared as `{predicate}` (but it is)"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("variable `{var}` must not be declared to be `{predicate}`"),
    )
    .report(db)
}

#[derive(Copy, Clone, Debug)]
pub struct UnassignableType<'db> {
    pub variable: SymVariable<'db>,
    pub variable_span: Span<'db>,
    pub variable_ty: SymTy<'db>,
    pub initializer: SymExpr<'db>,
}

impl<'db> OrElse<'db> for UnassignableType<'db> {
    fn or_else(&self, db: &'db dyn Db, _because: Because<'db>) -> Diagnostic {
        let initializer_ty = self.initializer.ty(db);
        Diagnostic::error(
            db,
            self.initializer.span(db),
            format!("variable initialized with value of wrong type"),
        )
        .label(
            db,
            Level::Error,
            self.initializer.span(db),
            format!("initializer has type `{initializer_ty}`"),
        )
        .label(
            db,
            Level::Info,
            self.variable_span,
            format!(
                "variable has type `{variable_ty}`",
                variable_ty = self.variable_ty
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BooleanTypeRequired<'db> {
    pub expr: SymExpr<'db>,
}

impl<'db> OrElse<'db> for BooleanTypeRequired<'db> {
    fn or_else(&self, db: &'db dyn Db, _because: Because<'db>) -> Diagnostic {
        Diagnostic::error(
            db,
            self.expr.span(db),
            format!("boolean expression required"),
        )
        .label(
            db,
            Level::Error,
            self.expr.span(db),
            format!(
                "I expected this expression to have a boolean type, but it has the type `{}`",
                self.expr.ty(db)
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NumericTypeExpected<'db> {
    pub expr: SymExpr<'db>,
    pub ty: SymTy<'db>,
}

impl<'db> OrElse<'db> for NumericTypeExpected<'db> {
    fn or_else(&self, db: &'db dyn Db, because: Because<'db>) -> Diagnostic {
        Diagnostic::error(db, self.expr.span(db), "numeric type expected").label(
            db,
            Level::Error,
            self.expr.span(db),
            format!("I expected a numeric type but I found `{}`", self.ty),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self)
    }
}
