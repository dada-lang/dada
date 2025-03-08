use std::sync::Arc;

use dada_ir_ast::{
    ast::SpannedBinaryOp,
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::{
        exprs::{SymExpr, SymPlaceExpr},
        primitive::SymPrimitive,
        red::RedTy,
        types::{SymPlace, SymTy, SymTyName},
        variables::SymVariable,
    },
};

use super::to_red::RedTyExt;

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
    fn report(&self, env: &Env<'db>, because: Because<'db>) -> Reported {
        self.or_else(env, because).report(env.db())
    }

    /// Create a diagnostic representing the error.
    ///
    /// The error would typically be expressed in high-level terms, like
    /// "cannot assign from `a` to `b`" or "incorrect type of function argument".
    ///
    /// The `because` argument signals the reason the low-level operation failed
    /// and will be used to provide additional details, like "`our` is not assignable to `my`".
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic;

    /// Convert a `&dyn OrElse<'db>` into an `ArcOrElse<'db>` so that it can be
    /// stored in an [`InferenceVarData`](`crate::check::inference::InferenceVarData`)
    /// or otherwise preserved beyond the current stack frame.
    /// See the trait comment for more details.
    fn to_arc(&self) -> ArcOrElse<'db>;
}

/// See [`OrElse::to_arc`][].
pub type ArcOrElse<'db> = Arc<dyn OrElse<'db> + 'db>;

impl<'db> OrElse<'db> for ArcOrElse<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        <dyn OrElse<'db>>::or_else(&**self, env, because)
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
            fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
                self.1.or_else(env, (self.0)(because))
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
    /// Miscellaneous
    JustSo,

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

    /// Primitive types are copy
    PrimitiveIsCopy(SymPrimitive<'db>),

    /// Leasing from a copy place yields a copy permission (which is not desired here)
    LeasedFromCopyIsCopy(Vec<SymPlace<'db>>),

    /// Universal mismatch
    UniversalMismatch(SymVariable<'db>, SymVariable<'db>),

    /// Name mismatch
    NameMismatch(SymTyName<'db>, SymTyName<'db>),

    /// Inference determined that the variable must be
    /// known to be `Predicate` "or else" the given error would occur.
    InferredIs(Predicate, ArcOrElse<'db>),

    /// Inference determined that the variable cannot be
    /// known to be `Predicate` "or else" the given error would occur.
    InferredIsnt(Predicate, ArcOrElse<'db>),

    /// Inference determined that the variable must have
    /// this lower bound "or else" the given error would occur.
    InferredLowerBound(RedTy<'db>, ArcOrElse<'db>),

    /// The inference variable declared here needs more constraints
    UnconstrainedInfer(Span<'db>),
}

impl<'db> Because<'db> {
    pub fn annotate_diagnostic(self, env: &Env<'db>, diagnostic: Diagnostic) -> Diagnostic {
        let db = env.db();
        let span = diagnostic.span.into_span(db);
        if let Some(child) = self.to_annotation(env, span) {
            diagnostic.child(child)
        } else {
            diagnostic
        }
    }

    fn to_annotation(&self, env: &Env<'db>, span: Span<'db>) -> Option<Diagnostic> {
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
            Because::VarDeclaredToBe(v, predicate) => Some(Diagnostic::info(
                db,
                span,
                format!("`{}` is declared to be `{}`", v, predicate),
            )),
            Because::NeverIsNotCopy => Some(Diagnostic::info(
                db,
                span,
                "the never type (`!`) is not considered `copy`",
            )),
            Because::NeverIsNotLent => Some(Diagnostic::info(
                db,
                span,
                "the never type (`!`) is not considered `lent`",
            )),
            Because::ClassIsNotCopy(name) => Some(Diagnostic::info(
                db,
                span,
                format!("class types (like `{name}`) are never considered `copy`"),
            )),
            Because::PrimitiveIsCopy(prim) => Some(Diagnostic::info(
                db,
                span,
                format!("primitive types (like `{prim}`) are always `copy`"),
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
            Because::InferredIs(predicate, or_else) => {
                let or_else_diagnostic = or_else.or_else(env, Because::JustSo);
                Some(Diagnostic::info(
                            db,
                            span,
                            format!(
                                "I inferred that `{predicate}` must be true because otherwise it would cause this error"
                            ),
                        )
                        .child(or_else_diagnostic))
            }
            Because::InferredIsnt(predicate, or_else) => {
                let or_else_diagnostic = or_else.or_else(env, Because::JustSo);
                Some(Diagnostic::info(
                        db,
                        span,
                        format!(
                            "I inferred that `{predicate}` must not be true because otherwise it would cause this error"
                        ),
                    )
                    .child(or_else_diagnostic))
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
                format!("this error might well be bogus, I just can't infer the type here"),
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
    pub span: Span<'db>,
    pub lower: SymTy<'db>,
    pub upper: SymTy<'db>,
}

impl<'db> OrElse<'db> for BadSubtypeError<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self { span, lower, upper } = *self;
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, span, format!("subtype expected")).label(
                db,
                Level::Error,
                span,
                format!("I expected `{lower} <: {upper}`, what gives?"),
            ),
        )
    }

    fn to_arc(&self) -> ArcOrElse<'db> {
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidInitializerType<'db> {
    pub variable: SymVariable<'db>,
    pub variable_span: Span<'db>,
    pub variable_ty: SymTy<'db>,
    pub initializer: SymExpr<'db>,
}

impl<'db> OrElse<'db> for InvalidInitializerType<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidAssignmentType<'db> {
    pub lhs: SymPlaceExpr<'db>,
    pub rhs: SymExpr<'db>,
}

impl<'db> OrElse<'db> for InvalidAssignmentType<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let lhs_ty = self.lhs.ty(db);
        let rhs_ty = self.rhs.ty(db);
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, self.rhs.span(db), format!("wrong type in assignment"))
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidReturnValue<'db> {
    pub value: SymExpr<'db>,
    pub return_ty: SymTy<'db>,
}

impl<'db> OrElse<'db> for InvalidReturnValue<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let value_ty = self.value.ty(db);
        because.annotate_diagnostic(
            env,
            Diagnostic::error(db, self.value.span(db), format!("invalid return value"))
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AwaitNonFuture<'db> {
    pub await_span: Span<'db>,
    pub future_expr: SymExpr<'db>,
}

impl<'db> OrElse<'db> for AwaitNonFuture<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let Self {
            await_span,
            future_expr,
        } = *self;
        let db = env.db();
        because.annotate_diagnostic(
            env,
            Diagnostic::error(
                db,
                await_span,
                format!("`await` can only be used on futures"),
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BooleanTypeRequired<'db> {
    pub expr: SymExpr<'db>,
}

impl<'db> OrElse<'db> for BooleanTypeRequired<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        because.annotate_diagnostic(
            env,
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
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OperatorRequiresNumericType<'db> {
    pub op: SpannedBinaryOp<'db>,
    pub expr: SymExpr<'db>,
}

impl<'db> OrElse<'db> for OperatorRequiresNumericType<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            op: SpannedBinaryOp { span: op_span, op },
            expr,
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
        Arc::new(*self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OperatorArgumentsMustHaveSameType<'db> {
    pub op: SpannedBinaryOp<'db>,
    pub lhs: SymExpr<'db>,
    pub rhs: SymExpr<'db>,
}

impl<'db> OrElse<'db> for OperatorArgumentsMustHaveSameType<'db> {
    fn or_else(&self, env: &Env<'db>, because: Because<'db>) -> Diagnostic {
        let db = env.db();
        let Self {
            op: SpannedBinaryOp { span: op_span, op },
            lhs,
            rhs,
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
        Arc::new(*self)
    }
}
