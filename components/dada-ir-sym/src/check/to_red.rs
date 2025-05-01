//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Errors};
use dada_util::boxed_async_fn;

use crate::ir::{
    indices::FromInfer,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind},
};

use super::{
    Env,
    inference::Direction,
    live_places::LivePlaces,
    places::PlaceTy,
    predicates::Predicate,
    red::{RedChain, RedLink, RedPerm, RedTy},
    runtime::Runtime,
    stream::Consumer,
};

pub trait RedTyExt<'db>: Sized {
    fn display<'a>(&'a self, env: &'a Env<'db>) -> impl std::fmt::Display;
}

impl<'db> RedTyExt<'db> for RedTy<'db> {
    fn display<'a>(&'a self, env: &'a Env<'db>) -> impl std::fmt::Display {
        struct Wrapper<'a, 'db> {
            ty: &'a RedTy<'db>,
            #[expect(dead_code)] // FIXME?
            env: &'a Env<'db>,
        }

        impl std::fmt::Display for Wrapper<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.ty {
                    RedTy::Error(_reported) => write!(f, "<error>"),
                    RedTy::Named(sym_ty_name, sym_generic_terms) => {
                        write!(f, "{}[{:?}]", sym_ty_name, sym_generic_terms)
                    }
                    RedTy::Never => write!(f, "!"),

                    // FIXME: do better by querying the env state
                    RedTy::Infer(v) => write!(f, "?{}", v.as_usize()),

                    RedTy::Var(sym_variable) => write!(f, "{}", sym_variable),
                    RedTy::Perm => write!(f, "<perm>"),
                }
            }
        }

        Wrapper { ty: self, env }
    }
}

/// Convert something to a [`RedTy`] and an (optional) permission that is applied to that [`RedTy`][].
pub trait ToRedTy<'db> {
    fn to_red_ty(&self, env: &mut Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>);
}

impl<'db> ToRedTy<'db> for SymGenericTerm<'db> {
    fn to_red_ty(&self, env: &mut Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>) {
        match *self {
            SymGenericTerm::Type(ty) => ty.to_red_ty(env),
            SymGenericTerm::Perm(perm) => perm.to_red_ty(env),
            SymGenericTerm::Place(_) => panic!("cannot create a red term from a place"),
            SymGenericTerm::Error(reported) => (RedTy::err(env.db(), reported), None),
        }
    }
}

impl<'db> ToRedTy<'db> for SymTy<'db> {
    fn to_red_ty(&self, env: &mut Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>) {
        to_red_ty_with_runtime(*self, env.runtime())
    }
}

/// Convert `ty` to a red-ty given a runtime.
///
/// See [`ToRedTy`][].
pub fn to_red_ty_with_runtime<'db>(
    ty: SymTy<'db>,
    runtime: &Runtime<'db>,
) -> (RedTy<'db>, Option<SymPerm<'db>>) {
    let db = runtime.db;
    match *ty.kind(db) {
        SymTyKind::Perm(perm0, sym_ty) => match to_red_ty_with_runtime(sym_ty, runtime) {
            (red_ty, None) => (red_ty, Some(perm0)),
            (red_ty, Some(perm1)) => (red_ty, Some(SymPerm::apply(db, perm0, perm1))),
        },
        SymTyKind::Named(n, ref g) => (RedTy::Named(n, g.clone()), None),
        SymTyKind::Infer(infer) => {
            // every type inference variable has an associated permission inference variable,
            // so split that off
            let perm_infer = runtime.perm_infer(infer);
            (RedTy::Infer(infer), Some(SymPerm::infer(db, perm_infer)))
        }
        SymTyKind::Var(v) => (RedTy::Var(v), None),
        SymTyKind::Never => (RedTy::Never, None),
        SymTyKind::Error(reported) => (RedTy::err(db, reported), None),
    }
}

impl<'db> ToRedTy<'db> for SymPerm<'db> {
    fn to_red_ty(&self, env: &mut Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>) {
        let db = env.db();
        match *self.kind(db) {
            SymPermKind::Error(reported) => (RedTy::err(db, reported), None),
            _ => (RedTy::Perm, Some(*self)),
        }
    }
}

pub trait ToRedPerm<'db> {
    async fn to_red_perm(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'_, 'db, RedPerm<'db>, Errors<()>>,
    ) -> Errors<()>;
}

impl<'db, T: ToRedChainVec<'db>> ToRedPerm<'db> for T {
    async fn to_red_perm(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'_, 'db, RedPerm<'db>, Errors<()>>,
    ) -> Errors<()> {
        self.to_red_chain_vec(
            env,
            live_after,
            direction,
            Consumer::new(async |env, chainvec| {
                consumer
                    .consume(env, RedPerm::new(env.db(), chainvec))
                    .await
            }),
        )
        .await
    }
}

/// Create a `Vec<RedChain>`, each of which are a canonical list of permissions.
/// Canonical means that
///
/// * inference variables have been bounded,
/// * permissions have been flattened into distinct chains (`ref[p, q] => ref[p], ref[q]`),
/// * copy applications reduced (`mut[p] ref[q] => ref[q]`),
/// * and tails expanded (`mut[q] => mut[q] mut[p]`, given `let q: mut[p] String`).
///
/// The `consumer` callback may be invoked multiple times as a result of
/// inference variable bounding. Each callback is given a vector
/// corresponding to the collected results from
/// [`some_expanded_red_chain` in dada-model][dm].
///
/// [dm]: https://github.com/dada-lang/dada-model/blob/main/src/type_system/redperms.rs
pub trait ToRedChainVec<'db> {
    async fn to_red_chain_vec(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'_, 'db, Vec<RedChain<'db>>, Errors<()>>,
    ) -> Errors<()>;
}

impl<'db, T: ToRedLinkVecs<'db>> ToRedChainVec<'db> for T {
    async fn to_red_chain_vec(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'_, 'db, Vec<RedChain<'db>>, Errors<()>>,
    ) -> Errors<()> {
        self.to_red_linkvecs(
            env,
            live_after,
            direction,
            Consumer::new(async |env, linkvecs| {
                expand_tail(env, live_after, direction, linkvecs, vec![], &mut consumer).await
            }),
        )
        .await
    }
}

/// After we've done the initial expansion to red links, we may end up with a chain
/// that ends in something like `mut[p]` or `ref[p]`. In that case, we need to
/// find the permissions from `p` and append them to the chain. This is called
/// "expansion" in the dada-model code. This function expands the tail recursively
/// until there are no more permissions to add and then invokes `consumer.consume`.
#[boxed_async_fn]
async fn expand_tail<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    direction: Direction,
    mut unexpanded_linkvecs: Vec<Vec<RedLink<'db>>>,
    mut expanded_chains: Vec<RedChain<'db>>,
    consumer: &mut Consumer<'_, 'db, Vec<RedChain<'db>>, Errors<()>>,
) -> Errors<()> {
    let db = env.db();

    // This is super annoying. We have a vector of "pseudo-chains" (Vec<RedLink>)
    // like `[[mut[a]], [ref[b]], [my]]`. We want to take the tail of each
    // of those chains and expand it, so, e.g., if `a: mut[x] String`,
    // then we might expand to `[[mut[a] mut[x]], [ref[b]], [my]]`.
    // But if `x: mut[y, z] String`, then we have to expand again,
    // and this time with some flattening, yielding
    // `[[mut[a] mut[x] muy[y]], [mut[a] mut[x] mut[z]], [ref[b]], [my]]`.
    // And we haven't even started in on `ref[b]`.
    // To make it more often, we need to expect multiple callbacks because
    // of inference, so we have to write everything in a "callback, recursive"
    // style and can't readily return values.
    //
    // We do this by popping links from `unexpected_links`, examining them,
    // and then either expanding them or else pushing onto `expanded_links`.

    let Some(linkvec) = unexpanded_linkvecs.pop() else {
        // Nothing left to expand! Great.
        return consumer.consume(env, expanded_chains).await;
    };

    let place = match linkvec.last() {
        Some(RedLink::RefLive(place))
        | Some(RedLink::RefDead(place))
        | Some(RedLink::MutLive(place))
        | Some(RedLink::MutDead(place)) => place,

        // If the last link does not reference a place,
        // or there is no last link (i.e., we have `my`),
        // then we are done expanding. Push resulting chain and recurse.
        Some(RedLink::Our) | Some(RedLink::Var(_)) | None => {
            expanded_chains.push(RedChain::new(db, linkvec));
            return expand_tail(
                env,
                live_after,
                direction,
                unexpanded_linkvecs,
                expanded_chains,
                consumer,
            )
            .await;
        }
    };

    // Otherwise, convert expand that place to red-links. This will yield
    // a vector so we have to "flat map" the result onto the list of expanded
    // chains.
    place
        .to_red_linkvecs(
            env,
            live_after,
            direction,
            Consumer::new(async |env, linkvecs_place: Vec<Vec<_>>| {
                let mut unexpanded_linkvecs = unexpanded_linkvecs.clone();
                let mut expanded_chains = expanded_chains.clone();

                for linkvec_place in linkvecs_place {
                    if linkvec_place.is_empty() {
                        // If the link vec we get back is `my`, then push a fully expanded chain.
                        // This corresponds to e.g. `mut[a]` where `a: my String` -- the permission
                        // is complete.
                        expanded_chains.push(RedChain::new(db, linkvec.clone()));
                    } else {
                        // Otherwise, concatenate this new link vec with our original
                        // and push it back onto the "unexpanded" list. We will recursively
                        // examine it.
                        let output = concat_linkvecs(env, &linkvec, &linkvec_place);
                        unexpanded_linkvecs.push(output);
                    }
                }
                expand_tail(
                    env,
                    live_after,
                    direction,
                    unexpanded_linkvecs,
                    expanded_chains,
                    consumer,
                )
                .await
            }),
        )
        .await
}

/// Convert the permission into a vector of red-links.
///
/// This is part of the red permission process and does three forms of processing.
///
/// * Copy links drop their prefix, so e.g. `mut[p] ref[q]` becomes just `[ref[q]]`
/// * Flattening, so `ref[p,q]` becomes `[ref[p], ref[q]]`.
/// * If inference variables are involved, we block waiting for their bounds.
///
/// The `consumer` callback may be invoked multiple times as a result of inference bounds.
/// Each callback has a list of lists of links.
pub trait ToRedLinkVecs<'db> {
    async fn to_red_linkvecs(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'_, 'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()>;
}

impl<'db> ToRedLinkVecs<'db> for SymPerm<'db> {
    #[boxed_async_fn]
    async fn to_red_linkvecs(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'_, 'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()> {
        let db = env.db();
        match *self.kind(db) {
            SymPermKind::My => consumer.consume(env, vec![vec![]]).await,
            SymPermKind::Our => consumer.consume(env, vec![vec![RedLink::Our]]).await,
            SymPermKind::Shared(ref places) => {
                let links = places
                    .iter()
                    .map(|&place| {
                        if live_after.is_live(env, place) {
                            RedLink::RefLive(place)
                        } else {
                            RedLink::RefDead(place)
                        }
                    })
                    .map(|link| vec![link])
                    .collect::<Vec<_>>();
                consumer.consume(env, links).await
            }
            SymPermKind::Leased(ref places) => {
                let links = places
                    .iter()
                    .map(|&place| {
                        if live_after.is_live(env, place) {
                            RedLink::MutLive(place)
                        } else {
                            RedLink::MutDead(place)
                        }
                    })
                    .map(|link| vec![link])
                    .collect::<Vec<_>>();
                consumer.consume(env, links).await
            }
            SymPermKind::Apply(lhs, rhs) => {
                lhs.to_red_linkvecs(
                    env,
                    live_after,
                    direction,
                    Consumer::new(async |env, linkvecs_lhs: Vec<Vec<RedLink<'db>>>| {
                        rhs.to_red_linkvecs(
                            env,
                            live_after,
                            direction,
                            Consumer::new(async |env, linkvecs_rhs: Vec<Vec<_>>| {
                                let links = concat_linkvecvecs(env, &linkvecs_lhs, &linkvecs_rhs);
                                consumer.consume(env, links).await
                            }),
                        )
                        .await
                    }),
                )
                .await
            }
            SymPermKind::Infer(v) => {
                env.require_for_all_red_perm_bounds(v, direction, async |env, red_perm| {
                    for &chain in red_perm.chains(db) {
                        let links = chain.links(db).to_vec();
                        consumer.consume(env, vec![links]).await?;
                    }
                    Ok(())
                })
                .await
            }
            SymPermKind::Var(v) => {
                let linkvec = {
                    if env.var_is_declared_to_be(v, Predicate::Owned)
                        && env.var_is_declared_to_be(v, Predicate::Move)
                    {
                        vec![]
                    } else if env.var_is_declared_to_be(v, Predicate::Owned)
                        && env.var_is_declared_to_be(v, Predicate::Move)
                    {
                        vec![RedLink::Our]
                    } else {
                        vec![RedLink::Var(v)]
                    }
                };

                consumer.consume(env, vec![linkvec]).await
            }
            SymPermKind::Error(reported) => return Err(reported),
        }
    }
}

impl<'db> ToRedLinkVecs<'db> for SymPlace<'db> {
    #[boxed_async_fn]
    async fn to_red_linkvecs(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'_, 'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()> {
        let ty = self.place_ty(env).await;
        ty.to_red_linkvecs(env, live_after, direction, consumer)
            .await
    }
}

impl<'db> ToRedLinkVecs<'db> for SymTy<'db> {
    #[boxed_async_fn]
    async fn to_red_linkvecs(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'_, 'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()> {
        if let (_, Some(perm)) = self.to_red_ty(env) {
            perm.to_red_linkvecs(env, live_after, direction, consumer)
                .await
        } else {
            consumer.consume(env, vec![]).await
        }
    }
}

fn concat_linkvecvecs<'db>(
    env: &mut Env<'db>,
    lhs: &[Vec<RedLink<'db>>],
    rhs: &[Vec<RedLink<'db>>],
) -> Vec<Vec<RedLink<'db>>> {
    let mut output = Vec::with_capacity(lhs.len() * rhs.len());
    for l in lhs {
        for r in rhs {
            output.push(concat_linkvecs(env, l, r));
        }
    }
    output
}

fn concat_linkvecs<'db>(
    env: &mut Env<'db>,
    lhs: &[RedLink<'db>],
    mut rhs: &[RedLink<'db>],
) -> Vec<RedLink<'db>> {
    let mut lhs = lhs.to_vec();

    while let Some((rhs_head, rhs_tail)) = rhs.split_first() {
        if rhs_head.is_copy(env) {
            return rhs.to_vec();
        }
        lhs.push(*rhs_head);
        rhs = rhs_tail;
    }

    lhs
}
