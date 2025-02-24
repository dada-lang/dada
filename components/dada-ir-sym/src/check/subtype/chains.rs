use std::{cell::Cell, task::Poll};

use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;
use futures::FutureExt;

use crate::check::{
    chains::Lien,
    combinator,
    env::Env,
    predicates::{Predicate, require_copy::require_term_is_copy},
    report::OrElse,
};

struct Alternatives {
    counter: Cell<usize>,
}

impl Alternatives {
    fn if_required(
        &self,
        not_required: impl Future<Output = Errors<bool>>,
        is_required: impl Future<Output = Errors<()>>,
    ) -> impl Future<Output = Errors<bool>> {
        let mut not_required = Box::pin(not_required);
        let mut is_required = Box::pin(is_required);
        std::future::poll_fn(move |cx| {
            if self.counter.get() == 0 {
                match is_required.poll_unpin(cx) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(true)),
                    Poll::Ready(Err(reported)) => Poll::Ready(Err(reported)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                not_required.poll_unpin(cx)
            }
        })
    }
}

// Rules (ignoring inference and layout rules)
//
// * `my <= C`
// * `our <= C1 if C1 is copy`
// * `(our C0) <= (our C1) if C0 <= C1`
// * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
// * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
// * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
// * `X C0 <= X C1 if C0 <= C1`
// * `X <= our if X is copy+owned`
// * `X <= my if X is move+owned`

#[boxed_async_fn]
async fn sub_chains<'a, 'db>(
    env: &'a Env<'db>,
    alternatives: &Alternatives,
    lower_chain: &[Lien<'db>],
    upper_chain: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    match (lower_chain.split_first(), upper_chain.split_first()) {
        (None, _) => {
            // `my <= C`
            Ok(true)
        }

        (Some((&Lien::Error(reported), _)), _) | (_, Some((&Lien::Error(reported), _))) => {
            Err(reported)
        }

        (Some((&Lien::Infer(v0), c0)), None) => {
            // XXX add bound to v0
            todo!("{v0:?} {c0:?}")
        }

        (Some((&Lien::Var(v0), [])), None) => {
            // `X <= my if X is move+owned`
            Ok(env.var_is_declared_to_be(v0, Predicate::Move)
                && env.var_is_declared_to_be(v0, Predicate::Owned))
        }

        // Nothing else is a subchain of `my`
        (Some((&Lien::Our, _)), None) => Ok(false),
        (Some((&Lien::Shared(_), _)), None) => Ok(false),
        (Some((&Lien::Leased(_), _)), None) => Ok(false),
        (Some((&Lien::Var(_), [_, ..])), None) => Ok(false),

        (Some((&lien0, c0)), Some((&lien1, c1))) => {
            sub_chains1(env, alternatives, lien0, c0, lien1, c1, or_else).await
        }
    }
}

#[boxed_async_fn]
async fn sub_chains1<'a, 'db>(
    env: &'a Env<'db>,
    alternatives: &Alternatives,
    lower_chain_head: Lien<'db>,
    lower_chain_tail: &[Lien<'db>],
    upper_chain_head: Lien<'db>,
    upper_chain_tail: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let db = env.db();
    match (
        lower_chain_head,
        lower_chain_tail,
        upper_chain_head,
        upper_chain_tail,
    ) {
        (Lien::Error(reported), _, _, _) | (_, _, Lien::Error(reported), _) => Err(reported),

        (Lien::Infer(v0), [], Lien::Infer(v1), []) => {
            // XXX relate v0 and v1
            todo!("{v0:?} {v1:?}")
        }

        (Lien::Infer(v0), c0, _, _) => {
            if c0.is_empty() {
                // XXX add bound to v0
                todo!("{v0:?} {c0:?}")
            } else {
                todo!("{v0:?} {c0:?}")
            }
        }

        (_, _, Lien::Infer(v1), c1) => {
            if c1.is_empty() {
                // XXX add bound to v0
                todo!("{v1:?} {c1:?}")
            } else {
                todo!("{v1:?} {c1:?}")
            }
        }

        (Lien::Our, [], head1, tail1) => {
            // `our <= C1 if C1 is copy`
            alternatives
                .if_required(
                    async {
                        // XXX
                        Ok(true)
                    },
                    async {
                        require_term_is_copy(
                            env,
                            Lien::head_tail_to_perm(db, head1, tail1).into(),
                            or_else,
                        )
                        .await
                    },
                )
                .await
        }
        (Lien::Our, c0, Lien::Our, c1) => {
            // `(our C0) <= (our C1) if C0 <= C1`
            sub_chains(env, alternatives, c0, c1, or_else).await
        }
        (Lien::Our, _, Lien::Leased(_), _) => Ok(false),
        (Lien::Our, _, Lien::Shared(_), _) => Ok(false),
        (Lien::Our, _, Lien::Var(_), _) => Ok(false),

        (Lien::Leased(_), _, Lien::Our, _) => Ok(false),
        (Lien::Leased(place0), c0, Lien::Leased(place1), c1) => {
            // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternatives, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Leased(_), _, Lien::Shared(_), _) => Ok(false),
        (Lien::Leased(_), _, Lien::Var(_), _) => Ok(false),

        (Lien::Shared(place0), c0, Lien::Our, [lien1, c1 @ ..]) => {
            // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
            sub_chains1(
                env,
                alternatives,
                Lien::Leased(place0),
                c0,
                *lien1,
                c1,
                or_else,
            )
            .await
        }
        (Lien::Shared(_), _, Lien::Our, []) => {
            // See above rule: if C1 is [] then `leased[place0] C0 <= []` will also be false.
            Ok(false)
        }
        (Lien::Shared(place0), c0, Lien::Shared(place1), c1) => {
            // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternatives, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Shared(_), _, Lien::Leased(_), _) => Ok(false),
        (Lien::Shared(_), _, Lien::Var(_), _) => Ok(false),

        (Lien::Var(v0), [], Lien::Our, []) => {
            // `X <= our`
            Ok(env.var_is_declared_to_be(v0, Predicate::Copy)
                && env.var_is_declared_to_be(v0, Predicate::Owned))
        }
        (Lien::Var(_), _, Lien::Our, _) => Ok(false),
        (Lien::Var(v0), c0, Lien::Var(v1), c1) => {
            // * `X C0 <= X C1 if C0 <= C1`
            if v0 == v1 {
                sub_chains(env, alternatives, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Var(_), _, Lien::Leased(_), _) => Ok(false),
        (Lien::Var(_), _, Lien::Shared(_), _) => Ok(false),
    }
}
