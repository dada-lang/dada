use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        predicates::{
            Predicate,
            test::{combinator::exists, is::test_term_is},
        },
    },
    ir::{
        classes::SymAggregateStyle,
        indices::InferVarIndex,
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind, SymTyName},
        variables::SymVariable,
    },
};

pub(crate) async fn require_term_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    match term {
        SymGenericTerm::Type(sym_ty) => require_ty_is(env, span, sym_ty, predicate).await,
        SymGenericTerm::Perm(sym_perm) => require_perm_is(env, span, sym_perm, predicate).await,
        SymGenericTerm::Place(place) => panic!("unexpected place term: {place:?}"),
        SymGenericTerm::Error(reported) => Err(reported),
    }
}

/// Requires that `(lhs rhs)` satisfies the given predicate.
/// The semantics of `(lhs rhs)` is: `rhs` if `rhs is copy` or `lhs union rhs` otherwise.
async fn require_apply_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    match predicate {
        Predicate::Move => {
            do_both(
                require_term_is(env, span, lhs, Predicate::Move),
                require_term_is(env, span, rhs, Predicate::Move),
            )
            .await
        }
        Predicate::Owned => {
            if test_term_is(env, rhs, Predicate::Copy).await? {
                // If RHS is copy, then (LHS RHS) is equivalent to (RHS)
                // and therefore `lhs` doesn't matter.
                require_term_is(env, span, rhs, Predicate::Owned).await?;
            } else {
                // Otherwise, both must be `owned`.
                do_both(
                    require_term_is(env, span, lhs, Predicate::Owned),
                    require_term_is(env, span, rhs, Predicate::Owned),
                )
                .await?;
            }
            Ok(())
        }
        Predicate::Copy => {
            if !test_term_is(env, rhs, Predicate::Copy).await? {
                require_term_is(env, span, lhs, Predicate::Copy).await?;
            }
            Ok(())
        }
        Predicate::Lent => {
            if test_term_is(env, rhs, Predicate::Copy).await? {
                // If RHS is copy, then (LHS RHS) is equivalent to (RHS)
                // and therefore the RHS must be `lent` but `lhs` doesn't matter.
                require_term_is(env, span, rhs, Predicate::Lent).await?;
            } else {
                // Otherwise, at least one must be `lent`.
                require_either_is(env, span, lhs, rhs, Predicate::Lent).await?;
            }
            Ok(())
        }
    }
}

/// Requires that either `lhs` or `rhs` satisfies the given predicate.
async fn require_either_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    lhs: SymGenericTerm<'db>,
    rhs: SymGenericTerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    // Simultaneously test for whether LHS/RHS is `predicate`.
    // If either is, we are done.
    // If either is *not*, the other must be.
    do_both(
        async {
            if !test_term_is(env, rhs, predicate).await? {
                require_term_is(env, span, lhs, predicate).await?;
            }
            Ok(())
        },
        async {
            if !test_term_is(env, lhs, predicate).await? {
                require_term_is(env, span, rhs, predicate).await?;
            }
            Ok(())
        },
    )
    .await
}

#[boxed_async_fn]
async fn require_ty_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymTy<'db>,
    predicate: Predicate,
) -> Errors<()> {
    let db = env.db();
    match *term.kind(db) {
        // Error cases first
        SymTyKind::Error(reported) => Err(reported),

        // Apply
        SymTyKind::Perm(sym_perm, sym_ty) => {
            require_apply_is(env, span, sym_perm.into(), sym_ty.into(), predicate).await
        }

        // Never
        SymTyKind::Never => match predicate {
            Predicate::Move | Predicate::Owned => Ok(()),
            Predicate::Copy | Predicate::Lent => {
                Err(report_never_must_be_but_isnt(env, span, predicate))
            }
        },

        // Variable and inference
        SymTyKind::Infer(infer) => require_infer_is(env, span, infer, predicate),
        SymTyKind::Var(var) => require_var_is(env, span, var, predicate),

        // Named types
        SymTyKind::Named(sym_ty_name, ref generics) => match sym_ty_name {
            SymTyName::Primitive(_sym_primitive) => match predicate {
                Predicate::Copy | Predicate::Owned => Ok(()),
                Predicate::Move | Predicate::Lent => {
                    Err(report_term_must_be_but_isnt(env, span, term, predicate))
                }
            },
            SymTyName::Aggregate(sym_aggregate) => match sym_aggregate.style(db) {
                SymAggregateStyle::Class => {
                    require_class_is(env, span, term, generics, predicate).await
                }
                SymAggregateStyle::Struct => match predicate {
                    Predicate::Move => {
                        require_exists(
                            generics,
                            async |&generic| test_term_is(env, generic, predicate).await,
                            || report_term_must_be_but_isnt(env, span, term, predicate),
                        )
                        .await
                    }
                    Predicate::Copy | Predicate::Owned => {
                        require_for_all(generics, async |&generic| {
                            require_term_is(env, span, generic, predicate).await
                        })
                        .await
                    }
                    Predicate::Lent => {
                        Err(report_term_must_be_but_isnt(env, span, term, predicate))
                    }
                },
            },
            SymTyName::Future => require_class_is(env, span, term, generics, predicate).await,
            SymTyName::Tuple { arity } => {
                assert_eq!(arity, generics.len());
                match predicate {
                    Predicate::Move => {
                        require_exists(
                            generics,
                            async |&generic| test_term_is(env, generic, predicate).await,
                            || report_term_must_be_but_isnt(env, span, term, predicate),
                        )
                        .await
                    }
                    Predicate::Copy | Predicate::Owned => {
                        require_for_all(generics, async |&generic| {
                            require_term_is(env, span, generic, predicate).await
                        })
                        .await
                    }
                    Predicate::Lent => {
                        Err(report_term_must_be_but_isnt(env, span, term, predicate))
                    }
                }
            }
        },
    }
}

async fn require_class_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: SymTy<'db>,
    generics: &[SymGenericTerm<'db>],
    predicate: Predicate,
) -> Errors<()> {
    match predicate {
        // Classes are always move
        Predicate::Move => Ok(()),

        // A class is owned if its generics are owned
        Predicate::Owned => {
            require_for_all(generics, async |&generic| {
                require_term_is(env, span, generic, Predicate::Owned).await
            })
            .await
        }

        // Classes are never intrinsically copy or lent.
        Predicate::Copy | Predicate::Lent => {
            Err(report_term_must_be_but_isnt(env, span, term, predicate))
        }
    }
}

#[boxed_async_fn]
async fn require_perm_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    perm: SymPerm<'db>,
    predicate: Predicate,
) -> Errors<()> {
    let db = env.db();
    match (predicate, perm.kind(db)) {
        // Error cases first
        (_, &SymPermKind::Error(reported)) => Err(reported),

        // My = Move & Owned
        (Predicate::Move | Predicate::Owned, &SymPermKind::My) => Ok(()),
        (Predicate::Copy | Predicate::Lent, &SymPermKind::My) => {
            Err(report_term_must_be_but_isnt(env, span, perm, predicate))
        }

        // Our = Copy & Owned
        (Predicate::Copy | Predicate::Owned, &SymPermKind::Our) => Ok(()),
        (Predicate::Move | Predicate::Lent, &SymPermKind::Our) => {
            Err(report_term_must_be_but_isnt(env, span, perm, predicate))
        }

        // Shared = Copy & Lent
        (Predicate::Copy | Predicate::Lent, &SymPermKind::Shared(_)) => Ok(()),
        (Predicate::Move | Predicate::Owned, &SymPermKind::Shared(_)) => {
            Err(report_term_must_be_but_isnt(env, span, perm, predicate))
        }

        // Leased = Move & Lent
        (Predicate::Move | Predicate::Lent, &SymPermKind::Leased(_)) => Ok(()),
        (Predicate::Copy | Predicate::Owned, &SymPermKind::Leased(_)) => {
            Err(report_term_must_be_but_isnt(env, span, perm, predicate))
        }

        // Apply
        (predicate, &SymPermKind::Apply(lhs, rhs)) => {
            require_apply_is(env, span, lhs.into(), rhs.into(), predicate).await
        }

        // Variable and inference
        (predicate, &SymPermKind::Var(var)) => require_var_is(env, span, var, predicate),
        (predicate, &SymPermKind::Infer(infer)) => require_infer_is(env, span, infer, predicate),
    }
}

fn require_var_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Errors<()> {
    if env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(report_var_must_be_but_is_not_declared_to_be(
            env, span, var, predicate,
        ))
    }
}

/// Requires the inference variable to meet the given predicate (possibly reporting an error
/// if that is contradictory).
pub fn require_infer_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<()> {
    let inverted_predicate = predicate.invert();
    let (is_already, is_inverted_already) = env.runtime().with_inference_var_data(infer, |data| {
        (data.is(predicate), data.is(inverted_predicate))
    });

    // Check if we are already required to be the predicate.
    if is_already.is_some() {
        return Ok(());
    }

    // Check if were already required to be the inverted predicate
    // and report an error if so.
    if let Some(inverted_span) = is_inverted_already {
        return Err(report_infer_is_contradictory(
            env,
            infer,
            predicate,
            span,
            inverted_predicate,
            inverted_span,
        ));
    }

    // Record the requirement in the runtime, awakening any tasks that may be impacted.
    env.runtime()
        .require_inference_var_is(infer, predicate, span);

    Ok(())
}

fn report_infer_is_contradictory<'db>(
    env: &Env<'db>,
    var_index: InferVarIndex,
    predicate: Predicate,
    predicate_span: Span<'db>,
    inverted_predicate: Predicate,
    inverted_span: Span<'db>,
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
        format!("I could not infer a `{var_kind}` here because it would have to be both `{predicate}` and `{inverted_predicate}`"),
    )
    .label(
        db,
        Level::Error,
        inverted_span,
        format!("required to be `{inverted_predicate}` here"),
    )
    .label(
        db,
        Level::Error,
        predicate_span,
        format!("required to be `{predicate}` here"),
    )
    .report(db)
}

fn report_never_must_be_but_isnt<'db>(
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

fn report_term_must_be_but_isnt<'db>(
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

fn report_var_must_be_but_is_not_declared_to_be<'db>(
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

async fn require_for_all<'db, T>(
    items: impl IntoIterator<Item = T>,
    f: impl AsyncFn(T) -> Errors<()>,
) -> Errors<()> {
    let _v: Vec<()> = futures::future::try_join_all(items.into_iter().map(|elem| f(elem))).await?;
    Ok(())
}

async fn require_exists<'db, T>(
    items: impl IntoIterator<Item = T>,
    test_fn: impl AsyncFn(T) -> Errors<bool>,
    or_else: impl FnOnce() -> Reported,
) -> Errors<()> {
    if exists(items, test_fn).await? {
        Ok(())
    } else {
        Err(or_else())
    }
}

async fn do_both<'db>(
    first: impl Future<Output = Errors<()>>,
    second: impl Future<Output = Errors<()>>,
) -> Errors<()> {
    let ((), ()) = futures::future::try_join(first, second).await?;
    Ok(())
}
