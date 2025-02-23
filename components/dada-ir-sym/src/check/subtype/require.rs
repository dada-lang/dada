//! Implement object-level subtyping.

use dada_ir_ast::{
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        chains::{Chain, Lien, RedTerm, RedTy, ToRedTerm, TyChain},
        combinator,
        env::Env,
        predicates::{
            is_provably_copy::term_is_provably_copy,
            is_provably_lent::term_is_provably_lent,
            is_provably_move::term_is_provably_move,
            is_provably_owned::term_is_provably_owned,
            isnt_provably_copy::term_isnt_provably_copy,
            require_copy::require_term_is_copy,
            require_isnt_provably_copy::require_term_isnt_provably_copy,
            require_lent::require_term_is_lent,
            require_move::{require_chain_is_move, require_term_is_move},
            require_owned::{require_chain_is_owned, require_term_is_owned},
            require_term_is_leased, require_term_is_not_leased, term_is_provably_leased,
        },
        report::{Because, OrElse},
    },
    ir::{
        indices::InferVarIndex,
        primitive::SymPrimitiveKind,
        types::{SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyKind, SymTyName},
        variables::SymVariable,
    },
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Expected {
    // The lower type is the expected one.
    Lower,

    // The upper type is the expected one.
    Upper,
}
impl Expected {
    fn expected_found<T>(self, lower: T, upper: T) -> (T, T) {
        match self {
            Expected::Lower => (lower, upper),
            Expected::Upper => (upper, lower),
        }
    }
}

pub async fn require_assignable_type<'db>(
    env: &Env<'db>,
    value_ty: SymTy<'db>,
    place_ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    match (value_ty.kind(db), place_ty.kind(db)) {
        (SymTyKind::Never, _) => Ok(()),
        _ => require_sub_terms(env, Expected::Upper, span, value_ty, place_ty).await,
    }
}

pub async fn require_sub_terms<'a, 'db>(
    env: &'a Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    combinator::require_all!(
        propagate_bounds(env, lower.into(), upper.into(), or_else),
        async {
            // Reduce and relate chains
            let red_term_lower = lower.to_red_term(db, env).await;
            let red_term_upper = upper.to_red_term(db, env).await;
            require_sub_redterms(env, red_term_lower, red_term_upper, or_else).await
        },
    )
    .await
}

/// Whenever we require that `lower <: upper`, we can also propagate certain bounds,
/// such as copy/lent and owned/move, from lower-to-upper and upper-to-lower.
/// This can unblock inference.
async fn propagate_bounds<'db>(
    env: &Env<'db>,
    lower: SymGenericTerm<'db>,
    upper: SymGenericTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    combinator::require_all!(
        // If subtype is copy, supertype must be
        async {
            if term_is_provably_copy(env, lower).await? {
                require_term_is_copy(env, upper, or_else).await?;
            }
            Ok(())
        },
        // If subtype is lent, supertype must be
        async {
            if term_is_provably_lent(env, lower).await? {
                require_term_is_lent(env, upper, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something move if you are move
        async {
            if term_is_provably_move(env, upper).await? {
                require_term_is_move(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something that isn't copy if you aren't copy
        async {
            if term_isnt_provably_copy(env, upper).await? {
                require_term_isnt_provably_copy(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something owned if you are owned
        async {
            if term_is_provably_owned(env, upper).await? {
                require_term_is_owned(env, lower, or_else).await?;
            }
            Ok(())
        },
        // Can only be a supertype of something leased if you are leased
        async {
            if term_is_provably_leased(env, lower).await? {
                require_term_is_leased(env, upper, or_else).await?;
            }
            Ok(())
        },
        // Can only be a subtype of something leased if you are leased
        async {
            if term_is_provably_leased(env, upper).await? {
                require_term_is_leased(env, lower, or_else).await?;
            }
            Ok(())
        },
    )
    .await
}

async fn require_sub_red_terms<'a, 'db>(
    env: &'a Env<'db>,
    lower: RedTerm<'db>,
    upper: RedTerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    match (&lower.ty(), &upper.ty()) {
        (RedTy::Error(reported), _) | (_, RedTy::Error(reported)) => Err(*reported),

        (RedTy::Infer(infer_lower), RedTy::Infer(infer_upper)) => todo!(),
        (RedTy::Infer(infer_lower), _) => todo!(),
        (_, RedTy::Infer(infer_lower)) => todo!(),

        (RedTy::Named(name_lower, lower_ty), RedTy::Named(name_upper, upper_ty)) => {}
        (RedTy::Named(..), _) | (_, RedTy::Named(..)) => todo!(),

        (RedTy::Never, RedTy::Never) => {
            require_sub_red_perms(env, lower.chains(), upper.chains(), or_else).await
        }
        (RedTy::Never, _) | (_, RedTy::Never) => todo!(),

        (RedTy::Var(var_lower), RedTy::Var(var_upper)) => {
            if var_lower == var_upper {
                require_sub_red_perms(env, expected, span, lower.chains(), upper.chains()).await
            } else {
                report_universal_mismatch(env, expected, span, var_lower, var_upper).await
            }
        }
        (RedTy::Var(_), _) | (_, RedTy::Var(_)) => todo!(),

        (RedTy::Perm, RedTy::Perm) => {
            require_sub_red_perms(env, expected, span, lower.chains(), upper.chains()).await
        }
    }
}

async fn require_sub_red_perms<'a, 'db>(
    env: &'a Env<'db>,
    lower_chains: &VecSet<Chain<'db>>,
    upper_chains: &VecSet<Chain<'db>>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    for lower_chain in lower_chains {
        for upper_chain in upper_chains {
            require_sub_red_perm(env, lower_chain, upper_chain, or_else).await?;
        }
    }
    Ok(())
}

async fn require_sub_chains<'a, 'db>(
    env: &'a Env<'db>,
    lower_chain: &[Lien<'db>],
    upper_chain: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // Rules (ignoring inference)
    //
    // * `my <= C`
    // * `our <= C1 if C1 is copy`
    // * `(our C0) <= (our C1) if C0 <= C1`
    // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
    // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
    // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
    // * `X C0 <= X C1 if C0 <= C1`

    let db = env.db();

    // If either the lower or upper bound is JUST an inference variable,
    // or both, this is an easy case-- we just want to add the opposite as a bound
    // of that variable.
    match (chain_is_infer(lower_chain), chain_is_infer(upper_chain)) {
        (Some(lower_var), Some(upper_var)) => todo!(),
        (Some(lower_var), None) => todo!(),
        (None, Some(upper_var)) => todo!(),
        (None, None) => (),
    }

    let Some((lower_head, lower_tail)) = lower_chain.split_first() else {
        // If the lower chain is empty, then it is "my", which implies upper-chain must be "my"
        return combinator::require_all!(
            require_chain_is_move(env, span, upper_chain),
            require_chain_is_owned(env, span, upper_chain),
        )
        .await;
    };

    match *lower_head {
        Lien::Our => {
            if lower_tail.is_empty() {
                // * `our <= C1 if C1 is copy`
                return require_term_is_not_leased(
                    env,
                    Lien::chain_to_perm(upper_chain, db).into(),
                )
                .await;
            } else {
                // * `(our C0) <= (our C1) if C0 <= C1`
            }
        }

        Lien::Shared(place) => {
            // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
            // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
            require_sub_of_shared(env, place, lower_tail, upper_chain, report_error).await
        }

        Lien::Leased(place) => {
            // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
        }

        Lien::Var(v) => {
            // * `X C0 <= X C1 if C0 <= C1`
        }

        Lien::Infer(v) => todo!(),

        Lien::Error(reported) => return Err(reported),
    }

    Ok(())
}

async fn require_sub_of_shared<'a, 'db>(
    env: &'a Env<'db>,
    lower_place: SymPlace<'db>,
    lower_tail: &[Lien<'db>],
    upper_chain: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
    // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
    let Some((upper_head, upper_tail)) = upper_chain.split_first() else {
        return Err(or_else.report(env.db(), Because::NotSubOfShared(lower_place)));
    };

    match *upper_head {
        Lien::Our => {
            // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
            require_sub_of_leased(env, lower_place, lower_tail, upper_tail, or_else).await
        }

        Lien::Shared(upper_place) => {
            // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
            if lower_place.is_covered_by(db, upper_place) {
                require_sub_chains(env, lower_tail, upper_tail, or_else).await
            } else {
                Err(or_else.report(env.db(), Because::NotSubOfShared(lower_place)))
            }
        }

        Lien::Leased(_) | Lien::Var(_) => {
            return Err(or_else.report(env.db(), Because::NotSubOfShared(lower_place)));
        }

        Lien::Infer(v) => todo!(),

        Lien::Error(reported) => return Err(reported),
    }
}

async fn require_sub_of_leased<'a, 'db>(
    env: &'a Env<'db>,
    lower_place: SymPlace<'db>,
    lower_tail: &[Lien<'db>],
    upper_chain: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    let Some((upper_head, upper_tail)) = upper_chain.split_first() else {
        return Err(or_else.report(env.db(), Because::NotSubOfLeased(lower_place)));
    };

    match *upper_head {
        Lien::Leased(upper_place) => {
            // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
            if lower_place.is_covered_by(db, upper_place) {
                require_sub_chains(env, lower_tail, upper_tail, or_else).await
            } else {
                Err(or_else.report(env.db(), Because::NotSubOfLeased(lower_place)))
            }
        }

        Lien::Our | Lien::Shared(_) | Lien::Var(_) => {
            return Err(or_else.report(env.db(), Because::NotSubOfLeased(lower_place)));
        }

        Lien::Infer(infer_var_index) => todo!(),

        Lien::Error(reported) => todo!(),
    }
}

fn chain_is_infer<'db>(chain: &[Lien<'db>]) -> Option<InferVarIndex> {
    if chain.len() != 1 {
        None
    } else {
        match chain[0] {
            Lien::Infer(v) => Some(v),
            _ => None,
        }
    }
}

fn report_error() -> Errors<()> {}
