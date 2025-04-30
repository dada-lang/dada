//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use std::pin::Pin;

use dada_ir_ast::diagnostic::{Err, Errors};
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::ir::{
    indices::FromInfer,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind},
};

use super::{
    Env,
    inference::Direction,
    live_places::LivePlaces,
    places::PlaceTy,
    predicates::{
        Predicate, is_provably_copy::place_is_provably_copy, test_perm_infer_is_known_to_be,
        test_var_is_provably,
    },
    red::{RedChain, RedLink, RedPerm, RedTy},
    report,
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
        consumer: Consumer<'db, RedPerm<'db>, Errors<()>>,
    ) -> Errors<()> {
    }
}

/// Create a `RedChain`, which is a canonical list of permissions.
/// Canonical means that
///
/// * inference variables have been bounded,
/// * permissions have been flattened (`ref[p, q] => ref[p], ref[q]`),
/// * copy applications reduced (`mut[p] ref[q] => ref[q]`),
/// * and tails expanded (`mut[q] => mut[q] mut[p]`, given `let q: mut[p] String`).
///
/// The `consumer` callback may be invoked multiple times as a result of
/// flattening and inference variable bounding. Each callback corresponds
/// to a result of [`some_expanded_red_chain` in dada-model][dm].
///
/// [dm]: https://github.com/dada-lang/dada-model/blob/main/src/type_system/redperms.rs
pub trait ToRedChains<'db> {
    async fn to_red_chains(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'db, Vec<RedChain<'db>>, Errors<()>>,
    ) -> Errors<()>;
}

impl<'db> ToRedChains<'db> for SymPerm<'db> {
    async fn to_red_chains(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'db, Vec<RedChain<'db>>, Errors<()>>,
    ) -> Errors<()> {
        self.to_red_links(
            env,
            live_after,
            direction,
            Consumer::new(async |env, links: Vec<RedLink<'db>>| {
                expand_tail(env, live_after, direction, links, &mut consumer).await
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
    links: Vec<Vec<RedLink<'db>>>,
    consumer: &mut Consumer<'db, Vec<RedChain<'db>>, Errors<()>>,
) -> Errors<()> {
    let db = env.db();

    // XXX
    let Some(last) = links.last() else {
        return consumer.consume(env, RedChain::my(db)).await;
    };

    let place = match *last {
        RedLink::RefLive(place)
        | RedLink::RefDead(place)
        | RedLink::MutLive(place)
        | RedLink::MutDead(place) => place,
        RedLink::Our => return consumer.consume(env, RedChain::new(db, links)).await,
        RedLink::Var(_) => return consumer.consume(env, RedChain::new(db, links)).await,
        RedLink::Error(reported) => return Err(reported),
    };

    place
        .to_red_links(
            env,
            live_after,
            direction,
            Consumer::new(async |env, links_place: Vec<_>| {
                if links_place.is_empty() {
                    consumer.consume(env, RedChain::new(db, &links)).await
                } else {
                    let output = concat_links(env, &links, links_place)?;
                    expand_tail(env, live_after, direction, output, consumer).await
                }
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
pub trait ToRedLinks<'db> {
    async fn to_red_links(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()>;
}

impl<'db> ToRedLinks<'db> for SymPerm<'db> {
    #[boxed_async_fn]
    async fn to_red_links(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
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
                lhs.to_red_links(
                    env,
                    live_after,
                    direction,
                    Consumer::new(async |env, links_lhs: Vec<Vec<RedLink<'db>>>| {
                        rhs.to_red_links(
                            env,
                            live_after,
                            direction,
                            Consumer::new(async |env, links_rhs: Vec<Vec<_>>| {
                                let links = concat_links2(env, &links_lhs, &links_rhs)?;
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
            SymPermKind::Var(v) => consumer.consume(env, vec![vec![RedLink::Var(v)]]).await,
            SymPermKind::Error(reported) => return Err(reported),
        }
    }
}

impl<'db> ToRedLinks<'db> for SymPlace<'db> {
    #[boxed_async_fn]
    async fn to_red_links(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        consumer: Consumer<'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()> {
        let ty = self.place_ty(env).await;
        ty.to_red_links(env, live_after, direction, consumer).await
    }
}

impl<'db> ToRedLinks<'db> for SymTy<'db> {
    #[boxed_async_fn]
    async fn to_red_links(
        &self,
        env: &mut Env<'db>,
        live_after: LivePlaces,
        direction: Direction,
        mut consumer: Consumer<'db, Vec<Vec<RedLink<'db>>>, Errors<()>>,
    ) -> Errors<()> {
        if let (_, Some(perm)) = self.to_red_ty(env) {
            perm.to_red_links(env, live_after, direction, consumer)
                .await
        } else {
            consumer.consume(env, vec![]).await
        }
    }
}

fn concat_links2<'db>(
    env: &mut Env<'db>,
    lhs: &[Vec<RedLink<'db>>],
    mut rhs: &[Vec<RedLink<'db>>],
) -> Errors<Vec<Vec<RedLink<'db>>>> {
    let mut output = vec![];
    for l in lhs {
        for r in rhs {
            output.push(concat_links(env, l, r)?);
        }
    }
    Ok(output)
}

fn concat_links<'db>(
    env: &mut Env<'db>,
    lhs: &[RedLink<'db>],
    mut rhs: &[RedLink<'db>],
) -> Errors<Vec<RedLink<'db>>> {
    let mut lhs = lhs.to_vec();

    while let Some((rhs_head, rhs_tail)) = rhs.split_first() {
        if rhs_head.is_copy(env)? {
            return Ok(rhs.to_vec());
        }
        lhs.push(*rhs_head);
        rhs = rhs_tail;
    }

    Ok(lhs)
}
