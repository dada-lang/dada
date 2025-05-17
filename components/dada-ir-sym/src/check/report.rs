use std::{panic::Location, sync::Arc};

use dada_ir_ast::{
    ast::SpannedBinaryOp,
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};
use serde::Serialize;

use crate::{
    check::{debug::export, env::Env, predicates::Predicate},
    ir::{
        exprs::{SymExpr, SymPlaceExpr},
        generics::SymWhereClause,
        types::{SymPlace, SymTy, SymTyName},
        variables::SymVariable,
    },
};

use super::{
    inference::Direction,
    red::{RedPerm, RedTy},
    to_red::RedTyExt,
};

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
    #[track_caller]
    fn report(&self, env: &mut Env<'db>, because: Because<'db>) -> Reported {
        let diagnostic = self.or_else(env, because);
        env.report(diagnostic)
    }

    /// Create a diagnostic representing the error.
    ///
    /// The error would typically be expressed in high-level terms, like
    /// "cannot assign from `a` to `b`" or "incorrect type of function argument".
    ///
    /// The `because` argument signals the reason the low-level operation failed
    /// and will be used to provide additional details, like "`our` is not assignable to `my`".
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic;

    /// Convert a `&dyn OrElse<'db>` into an `ArcOrElse<'db>` so that it can be
    /// stored in an [`InferenceVarData`](`crate::check::inference::InferenceVarData`)
    /// or otherwise preserved beyond the current stack frame.
    /// See the trait comment for more details.
    fn to_arc(&self) -> ArcOrElse<'db>;

    /// Returns the location in the *compiler source* where this `or_else` was created.
    /// Useful for debugging.
    fn compiler_location(&self) -> &'static Location<'static>;
}

/// See [`OrElse::to_arc`][].
#[derive(Clone)]
pub struct ArcOrElse<'db> {
    data: Arc<dyn OrElse<'db> + 'db>,
}

impl<'db, T> From<Arc<T>> for ArcOrElse<'db>
where
    T: OrElse<'db> + 'db,
{
    fn from(data: Arc<T>) -> Self {
        ArcOrElse { data }
    }
}

impl<'db> OrElse<'db> for ArcOrElse<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        self.data.or_else(env, because)
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        ArcOrElse::clone(self)
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.data.compiler_location()
    }
}

impl<'db> std::ops::Deref for ArcOrElse<'db> {
    type Target = dyn OrElse<'db> + 'db;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl Serialize for ArcOrElse<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErasedOrElse {
            compiler_location: export::CompilerLocation<'static>,
        }

        ErasedOrElse {
            compiler_location: self.compiler_location().into(),
        }
        .serialize(serializer)
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
    #[track_caller]
    fn map_because(
        self,
        f: impl 'db + Clone + Fn(Because<'db>) -> Because<'db>,
    ) -> impl OrElse<'db> {
        struct MapBecause<F, G>(F, G, &'static Location<'static>);

        impl<'db, F, G> OrElse<'db> for MapBecause<F, G>
        where
            F: 'db + Clone + Fn(Because<'db>) -> Because<'db>,
            G: std::ops::Deref<Target: OrElse<'db>>,
        {
            fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
                self.1.or_else(env, (self.0)(because))
            }

            fn to_arc(&self) -> ArcOrElse<'db> {
                Arc::new(MapBecause(self.0.clone(), self.1.to_arc(), self.2)).into()
            }

            fn compiler_location(&self) -> &'static Location<'static> {
                self.2
            }
        }

        MapBecause(f, self, Location::caller())
    }
}

/// Reason that a low-level typing operation failed.
pub enum Because<'db> {
    /// Miscellaneous
    JustSo,

    /// Universal variable was not declared to be `predicate` (and it must be)
    VarNotDeclaredToBe(SymVariable<'db>, Predicate),

    /// The never type is not copy
    NeverIsNotCopy,

    /// A where clause would be needed on the given variable
    NoWhereClause(SymVariable<'db>, Predicate),

    /// Struct types are never lent, even if they have lent things in them,
    /// as they can still have non-lent things.
    StructsAreNotLent(SymTyName<'db>),

    /// Leasing from a copy place yields a copy permission (which is not desired here)
    LeasedFromCopyIsCopy(Vec<SymPlace<'db>>),

    /// Universal mismatch
    UniversalMismatch(SymVariable<'db>, SymVariable<'db>),

    /// Name mismatch
    NameMismatch(SymTyName<'db>, SymTyName<'db>),

    /// Indicates that there was a previous constraint from elsewhere in the
    /// program that caused a conflict with the current value
    InferredPermBound(Direction, RedPerm<'db>, ArcOrElse<'db>),

    /// Inference determined that the variable must have
    /// this lower bound "or else" the given error would occur.
    InferredLowerBound(RedTy<'db>, ArcOrElse<'db>),

    /// The inference variable declared here needs more constraints
    UnconstrainedInfer(Span<'db>),
}

impl<'db> Because<'db> {
    pub fn annotate_diagnostic(self, env: &mut Env<'db>, diagnostic: Diagnostic) -> Diagnostic {
        let db = env.db();
        let span = diagnostic.span.into_span(db);
        if let Some(child) = self.to_annotation(env, span) {
            diagnostic.child(child)
        } else {
            diagnostic
        }
    }

    fn to_annotation(&self, env: &mut Env<'db>, span: Span<'db>) -> Option<Diagnostic> {
        let db = env.db();
        match self {
            Because::JustSo => None,
            Because::VarNotDeclaredToBe(v, predicate) => Some(Diagnostic::info(
                db,
                span,
                format!(
                    "to conclude that `{}` is `{}`, I would need you to add a declaration",
                    v, predicate
                ),
            )),
            Because::NeverIsNotCopy => Some(Diagnostic::info(
                db,
                span,
                "the never type (`!`) is not considered `copy`",
            )),
            Because::LeasedFromCopyIsCopy(places) => {
                if places.len() == 1 {
                    Some(Diagnostic::info(
                        db,
                        span,
                        format!(
                            "`{place}` is `copy`, so leasing from `{place}` yields a `copy` permission",
                            place = places[0]
                        ),
                    ))
                } else {
                    Some(Diagnostic::info(
                        db,
                        span,
                        format!(
                            "{places} are all `copy`, so leasing from them yields a `copy` permission",
                            places = anded_list(places),
                        ),
                    ))
                }
            }
            Because::UniversalMismatch(v1, v2) => Some(Diagnostic::info(
                db,
                span,
                format!("I cannot know whether `{v1}` and `{v2}` are the same"),
            )),
            Because::NameMismatch(n1, n2) => Some(Diagnostic::info(
                db,
                span,
                format!("`{n1}` and `{n2}` are distinct types"),
            )),
            Because::InferredPermBound(direction, red_perm, or_else) => {
                let or_else_diagnostic = or_else.or_else(env, Because::JustSo);
                Some(
                            Diagnostic::info(
                                db,
                                span,
                                format!(
                                    "I inferred that the perm must be {assignable_from_or_to} `{bound}` or else this error will occur",
                                    assignable_from_or_to = match direction {
                                        Direction::FromBelow => "assignable from",
                                        Direction::FromAbove => "assignable to",
                                    },
                                    bound = format!("{red_perm:?}"), // FIXME
                                ),
                            )
                            .child(or_else_diagnostic),
                        )
            }
            Because::InferredLowerBound(red_ty, or_else) => {
                let or_else_diagnostic = or_else.or_else(env, Because::JustSo);
                Some(Diagnostic::info(
                                db,
                                span,
                                format!(
                                    "I inferred that the type `{red_ty}` is required because otherwise it would cause this error",
                                    red_ty = red_ty.display(env),
                                ),
                            )
                            .child(or_else_diagnostic))
            }
            Because::UnconstrainedInfer(span) => Some(Diagnostic::info(
                db,
                *span,
                "this error might well be bogus, I just can't infer the type here".to_string(),
            )),
            Because::NoWhereClause(var, predicate) => Some(Diagnostic::info(
                db,
                span,
                format!("the variable `{var}` needs a where-clause to be considered `{predicate}`"),
            )),
            Because::StructsAreNotLent(s) => Some(Diagnostic::info(
                db,
                span,
                format!("the struct type `{s}` is never considered `lent`"),
            )),
        }
    }
}

fn anded_list<T>(v: &[T]) -> String
where
    T: std::fmt::Display,
{
    use std::fmt::Write;

    let mut s = String::new();
    let Some((last, prefix)) = v.split_last() else {
        return s;
    };

    for p in prefix {
        write!(s, "{p}, ").unwrap();
    }

    write!(s, "and {last}").unwrap();
    s
}

/// Give a really bad subtype error.
///
/// Every usage of this is a bug.
#[derive(Copy, Clone, Debug)]
pub struct BadSubtypeError<'db> {
    span: Span<'db>,
    lower: SymTy<'db>,
    upper: SymTy<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> BadSubtypeError<'db> {
    #[track_caller]
    pub fn new(span: Span<'db>, lower: SymTy<'db>, upper: SymTy<'db>) -> Self {
        Self {
            span,
            lower,
            upper,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for BadSubtypeError<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            span,
            lower,
            upper,
            compiler_location: _,
        } = *self;
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, span, "subtype expected".to_string()).label(
                db,
                Level::Error,
                span,
                format!("expected `{upper}`, found `{lower}`"),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

/// Give a really bad subtype error.
///
/// Every usage of this is a bug.
#[derive(Copy, Clone, Debug)]
pub struct WhereClauseError<'db> {
    span: Span<'db>,
    where_clause: SymWhereClause<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> WhereClauseError<'db> {
    #[track_caller]
    pub fn new(span: Span<'db>, where_clause: SymWhereClause<'db>) -> Self {
        Self {
            span,
            where_clause,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for WhereClauseError<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            span,
            where_clause,
            compiler_location: _,
        } = *self;
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                span,
                "where clause on function not satisfied".to_string(),
            )
            .label(
                db,
                Level::Error,
                span,
                format!("expected `{where_clause:?}`"),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidInitializerType<'db> {
    variable: SymVariable<'db>,
    variable_span: Span<'db>,
    variable_ty: SymTy<'db>,
    initializer: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> InvalidInitializerType<'db> {
    #[track_caller]
    pub fn new(
        variable: SymVariable<'db>,
        variable_span: Span<'db>,
        variable_ty: SymTy<'db>,
        initializer: SymExpr<'db>,
    ) -> Self {
        Self {
            variable,
            variable_span,
            variable_ty,
            initializer,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for InvalidInitializerType<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let initializer_ty = self.initializer.ty(db);
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                self.initializer.span(db),
                format!(
                    "variable `{v}` initialized with value of wrong type",
                    v = self.variable
                ),
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
                    "`{v}` has type `{variable_ty}`",
                    v = self.variable,
                    variable_ty = self.variable_ty
                ),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidAssignmentType<'db> {
    lhs: SymPlaceExpr<'db>,
    rhs: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> InvalidAssignmentType<'db> {
    #[track_caller]
    pub fn new(lhs: SymPlaceExpr<'db>, rhs: SymExpr<'db>) -> Self {
        Self {
            lhs,
            rhs,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for InvalidAssignmentType<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let lhs_ty = self.lhs.ty(db);
        let rhs_ty = self.rhs.ty(db);
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                self.rhs.span(db),
                "wrong type in assignment".to_string(),
            )
            .label(
                db,
                Level::Error,
                self.rhs.span(db),
                format!("this expression has type `{rhs_ty}`"),
            )
            .label(
                db,
                Level::Info,
                self.lhs.span(db),
                format!("I expected something assignable to this, which has type `{lhs_ty}`",),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidReturnValue<'db> {
    value: SymExpr<'db>,
    return_ty: SymTy<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> InvalidReturnValue<'db> {
    #[track_caller]
    pub fn new(value: SymExpr<'db>, return_ty: SymTy<'db>) -> Self {
        Self {
            value,
            return_ty,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for InvalidReturnValue<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let value_ty = self.value.ty(db);
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, self.value.span(db), "invalid return value".to_string())
                .label(
                    db,
                    Level::Error,
                    self.value.span(db),
                    format!(
                        "I expected a value of the return type, but this has type `{value_ty}`"
                    ),
                )
                .label(
                    db,
                    Level::Info,
                    // FIXME: with a bit of work, we could thread the span where return type is declared
                    self.value.span(db),
                    format!(
                        "the return type is declared to be `{return_ty}`",
                        return_ty = self.return_ty,
                    ),
                ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AwaitNonFuture<'db> {
    await_span: Span<'db>,
    future_expr: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> AwaitNonFuture<'db> {
    #[track_caller]
    pub fn new(await_span: Span<'db>, future_expr: SymExpr<'db>) -> Self {
        Self {
            await_span,
            future_expr,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for AwaitNonFuture<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let Self {
            await_span,
            future_expr,
            compiler_location: _,
        } = *self;
        let db = env.db();
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                await_span,
                "`await` can only be used on futures".to_string(),
            )
            .label(
                db,
                Level::Error,
                await_span,
                "I expect `await` to be applied to a future",
            )
            .label(
                db,
                Level::Info,
                future_expr.span(db),
                format!(
                    "this expression has type `{future_ty}`",
                    future_ty = future_expr.ty(db),
                ),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BooleanTypeRequired<'db> {
    expr: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> BooleanTypeRequired<'db> {
    #[track_caller]
    pub fn new(expr: SymExpr<'db>) -> Self {
        Self {
            expr,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for BooleanTypeRequired<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                self.expr.span(db),
                "boolean expression required".to_string(),
            )
            .label(
                db,
                Level::Error,
                self.expr.span(db),
                format!(
                    "I expected this expression to have a boolean type, but it has the type `{}`",
                    self.expr.ty(db)
                ),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NumericTypeExpected<'db> {
    expr: SymExpr<'db>,
    ty: SymTy<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> NumericTypeExpected<'db> {
    #[track_caller]
    pub fn new(expr: SymExpr<'db>, ty: SymTy<'db>) -> Self {
        Self {
            expr,
            ty,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for NumericTypeExpected<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, self.expr.span(db), "numeric type expected").label(
                db,
                Level::Error,
                self.expr.span(db),
                format!("I expected a numeric type but I found `{}`", self.ty),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OperatorRequiresNumericType<'db> {
    op: SpannedBinaryOp<'db>,
    expr: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> OperatorRequiresNumericType<'db> {
    #[track_caller]
    pub fn new(op: SpannedBinaryOp<'db>, expr: SymExpr<'db>) -> Self {
        Self {
            op,
            expr,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for OperatorRequiresNumericType<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            op: SpannedBinaryOp { span: op_span, op },
            expr,
            compiler_location: _,
        } = *self;

        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, expr.span(db), "numeric type expected")
                .label(
                    db,
                    Level::Error,
                    expr.span(db),
                    format!(
                        "I expected this to have a numeric type but it had the type `{}`",
                        expr.ty(db)
                    ),
                )
                .label(
                    db,
                    Level::Info,
                    op_span,
                    format!("the operator `{op}` requires numeric arguments"),
                ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OperatorArgumentsMustHaveSameType<'db> {
    op: SpannedBinaryOp<'db>,
    lhs: SymExpr<'db>,
    rhs: SymExpr<'db>,
    compiler_location: &'static Location<'static>,
}

impl<'db> OperatorArgumentsMustHaveSameType<'db> {
    #[track_caller]
    pub fn new(op: SpannedBinaryOp<'db>, lhs: SymExpr<'db>, rhs: SymExpr<'db>) -> Self {
        Self {
            op,
            lhs,
            rhs,
            compiler_location: Location::caller(),
        }
    }
}

impl<'db> OrElse<'db> for OperatorArgumentsMustHaveSameType<'db> {
    fn or_else(&self, env: &mut Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            op: SpannedBinaryOp { span: op_span, op },
            lhs,
            rhs,
            compiler_location: _,
        } = *self;

        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, op_span, "same types expected")
                .label(
                    db,
                    Level::Error,
                    op_span,
                    format!("I expected both arguments to `{op}` to have the same type",),
                )
                .label(
                    db,
                    Level::Info,
                    lhs.span(db),
                    format!("has type `{}`", lhs.ty(db)),
                )
                .label(
                    db,
                    Level::Info,
                    rhs.span(db),
                    format!("has type `{}`", rhs.ty(db)),
                ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self).into()
    }

    fn compiler_location(&self) -> &'static Location<'static> {
        self.compiler_location
    }
}
