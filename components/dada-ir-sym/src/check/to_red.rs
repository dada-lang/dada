//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use std::{boxed, pin::Pin};

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
    red::{RedPerm, RedPermLink, RedTy},
    runtime::Runtime,
};

trait ChainExt<'db>: Sized {
    /// Concatenate two lien chains; if `other` is copy, just returns `other`.
    async fn concat(&self, env: &mut Env<'db>, other: &Self) -> Errors<Self>;

    /// Check if the chain is copy. Will block if this chain contains an inference variable.
    async fn is_copy(&self, env: &mut Env<'db>) -> Errors<bool>;
}

impl<'db> ChainExt<'db> for RedPerm<'db> {
    /// See [`ChainExt::concat`][].
    async fn concat(&self, env: &mut Env<'db>, other: &Self) -> Errors<Self> {
        if other.is_copy(env).await? {
            Ok(other.clone())
        } else {
            let mut links = self.links.clone();
            links.extend(other.links.iter());
            Ok(Self::new(env.db(), links))
        }
    }

    /// See [`ChainExt::is_copy`][].
    async fn is_copy(&self, env: &mut Env<'db>) -> Errors<bool> {
        for lien in &self.links {
            if lien.is_copy(env).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

trait LienExt<'db>: Sized {
    /// Check if the lien is copy, blocking if inference info is needed.
    async fn is_copy(&self, env: &mut Env<'db>) -> Errors<bool>;
}

impl<'db> LienExt<'db> for RedPermLink<'db> {
    /// See [`LienExt::is_copy`][].
    async fn is_copy(&self, env: &mut Env<'db>) -> Errors<bool> {
        match *self {
            RedPermLink::Our | RedPermLink::Shared(_) => Ok(true),
            RedPermLink::Leased(_) => Ok(false),
            RedPermLink::Var(v) => Ok(test_var_is_provably(env, v, Predicate::Copy)),
            RedPermLink::Infer(v) => test_perm_infer_is_known_to_be(env, v, Predicate::Copy).await,
            RedPermLink::Error(reported) => Err(reported),
        }
    }
}

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

/// Create a "red perm", which is basically a flatted view of a perm.
/// Note that this method does only minimal simplification.
/// It does not check for dead places nor does it query the bounds
/// on inference variables. Callers are expected to manage that.
pub trait ToRedPerm<'db> {
    fn to_red_perm(&self, env: &mut Env<'db>) -> RedPerm<'db>;
}

impl<'db> ToRedPerm<'db> for SymPerm<'db> {
    fn to_red_perm(&self, env: &mut Env<'db>) -> RedPerm<'db> {
        let mut output = RedPerm::my(env.db());
        push_links_from_perm(env, *self, &mut output);
        output
    }
}

impl<'db> ToRedPerm<'db> for SymTy<'db> {
    fn to_red_perm(&self, env: &mut Env<'db>) -> RedPerm<'db> {
        let db = env.db();
        match *self.kind(db) {
            SymTyKind::Perm(lhs, rhs) => {
                let mut lhs = lhs.to_red_perm(env);
                let rhs = rhs.to_red_perm(env);
                for link in rhs.links {
                    push_red_link(env, link, &mut lhs);
                }
                lhs
            }
            SymTyKind::Infer(infer) => SymPerm::infer(db, env.perm_infer(infer)).to_red_perm(env),
            SymTyKind::Never | SymTyKind::Named(..) | SymTyKind::Var(_) => {
                SymPerm::my(db).to_red_perm(env)
            }
            SymTyKind::Error(reported) => RedPerm::err(db, reported),
        }
    }
}

fn push_links_from_perm<'db>(env: &mut Env<'db>, perm: SymPerm<'db>, output: &mut RedPerm<'db>) {
    let db = env.db();
    match perm.kind(db) {
        SymPermKind::My => {}
        SymPermKind::Our => {
            push_red_link(env, RedPermLink::Our, output);
        }
        SymPermKind::Shared(places) => {
            push_red_link(env, RedPermLink::Shared(VecSet::from_iter(places)), output);
        }
        SymPermKind::Leased(places) => {
            push_red_link(env, RedPermLink::Leased(VecSet::from_iter(places)), output);
        }
        SymPermKind::Apply(lhs, rhs) => {
            push_links_from_perm(env, *lhs, output);
            push_links_from_perm(env, *rhs, output);
        }
        SymPermKind::Infer(infer) => {
            push_red_link(env, RedPermLink::Infer(*infer), output);
        }
        SymPermKind::Var(var) => {
            push_red_link(env, RedPermLink::Var(*var), output);
        }
        SymPermKind::Error(reported) => {
            push_red_link(env, RedPermLink::Error(*reported), output);
        }
    }
}

fn push_red_link<'db>(env: &mut Env<'db>, link: RedPermLink<'db>, output: &mut RedPerm<'db>) -> ! {
    let db = env.db();
    let clear = match link {
        RedPermLink::Our | RedPermLink::Shared(_) => true,
        RedPermLink::Leased(vec_set) => false,
        RedPermLink::Var(var) => env.var_is_declared_to_be(*var, Predicate::Copy),
        RedPermLink::Infer(infer_var_index) => false,
        RedPermLink::Error(reported) => true,
    };
    if clear {
        output.links.clear();
    }
    output.links.push(link);
}

#[boxed_async_fn]
async fn simplify_red_perm<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    direction: Direction,
    red_perm: RedPerm<'db>,
    op: impl AsyncFnMut(&mut Env<'db>, RedPerm<'db>) -> Errors<()>,
) -> Errors<()> {
    let db = env.db();
    let mut output = RedPerm::my(db);

    // Step 1. Scan the links to find anything that is known to be copy.
    let mut start_index = 0;
    for (link, i) in output.links.iter().zip(0..) {
        if is_copy_link(env, link)? {
            start_index = i;
        }
    }

    //

    for link in red_perm.links {
        // Our plan:
        //
        // * If LHS or RHS is just an inference variable-- add appropriate bound
        // * Otherwise:
        //   - Flatten any inference variable bounds into the red perm
        //     - but what if we KNOW the inf variable is (say) copy but don't have a bound?
        //       -> then we can ignore earlier links
        //       - it seems to me that the "flattening" should occur when pushing something AFTER an inf variable
        //   - In the end check for shared/leased/given from dead places

        if let RedPermLink::Infer(infer) = link {
        } else if let Some(RedPermLink::Infer(infer)) = output.links.last() {
        } else {
            push_red_link(env, link, &mut output);
        }
    }

    Ok(())
}

fn is_copy_link<'db>(env: &Env<'db>, link: &RedPermLink<'db>) -> Errors<bool> {
    match link {
        RedPermLink::Our => Ok(true),
        RedPermLink::Shared(_) => Ok(true),
        RedPermLink::Leased(_) => Ok(false),
        RedPermLink::Var(var) => Ok(env.var_is_declared_to_be(var, Predicate::Copy)),
        RedPermLink::Infer(infer) => Ok(env.runtime().with_inference_var_data(*infer, |data| {
            data.is_known_to_provably_be(Predicate::Copy).is_some()
        })),
        RedPermLink::Error(reported) => Err(*reported),
    }
}
