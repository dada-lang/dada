//! Code to resolve inference variables to concrete types and permissions.

use dada_ir_ast::diagnostic::{Diagnostic, Err, Level};
use dada_util::Set;
use futures::StreamExt;

use crate::{
    check::chains::ToChain,
    ir::{
        indices::InferVarIndex,
        subst::Subst,
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy},
    },
};

use super::{
    Env,
    bound::Direction,
    chains::{Lien, LienChain, TyChain, TyChainKind},
};

pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}

pub struct Resolver<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env Env<'db>,
    var_stack: Set<InferVarIndex>,
}

impl<'env, 'db> Resolver<'env, 'db> {
    pub fn new(env: &'env Env<'db>) -> Self {
        Self {
            db: env.db(),
            env,
            var_stack: Default::default(),
        }
    }

    /// Return a version of `term` in which all inference variables are (deeply) removed.
    pub fn resolve_term<T>(&mut self, term: T, variance: Variance) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>>,
    {
        term.resolve_infer_var(self.db, |v| Some(self.resolve_infer_var(v, variance)))
    }

    /// Resolve an inference variable to a generic term, given the variance of the location in which it appears
    async fn resolve_infer_var(
        &mut self,
        v: InferVarIndex,
        variance: Variance,
    ) -> SymGenericTerm<'db> {
        if self.var_stack.insert(v) {
            let result = match self.env.infer_var_kind(v) {
                SymGenericKind::Type => self.resolve_ty_var(v, variance).await.into(),
                SymGenericKind::Perm => todo!(),
                SymGenericKind::Place => todo!(),
            };
            self.var_stack.remove(&v);
            result
        } else {
            let span = self
                .env
                .runtime()
                .with_inference_var_data(v, |data| data.span());
            SymGenericTerm::err(
                self.db,
                Diagnostic::error(self.db, span, "recursive inference variable").report(self.db),
            )
        }
    }

    /// Resolve a type inference variable `v` to a type, given the variance of the location in which it appears.
    async fn resolve_ty_var(&mut self, v: InferVarIndex, variance: Variance) -> SymTy<'db> {
        match variance {
            // In a covariant setting, we can pick any supertype of the "true type" represented by this variable.
            // So look at its supertype bounds.
            Variance::Covariant => self.bounding_ty(v, Direction::UpperBoundedBy).await,

            // As above, but for subtypes.
            Variance::Contravariant => self.bounding_ty(v, Direction::LowerBoundedBy).await,

            Variance::Invariant => {
                // FIXME
                self.bounding_ty(v, Direction::UpperBoundedBy).await
            }
        }
    }

    /// Return the bounding type on the type inference variable `v` from the given `direction`.
    async fn bounding_ty(&self, v: InferVarIndex, direction: Direction) -> SymTy<'db> {
        // First find the bounding type chains. These may contain inference variables but only in generic arguments.
        let mut perm_chains = vec![];
        let mut type_chain_kinds = vec![];
        let mut bounds = self.env.transitive_ty_var_bounds(v, direction);
        while let Some(ty) = bounds.next().await {
            let ty_chains = self.bounding_ty_chains(ty, direction).await;
            for TyChain { lien, kind } in ty_chains {
                perm_chains.push(lien);
                type_chain_kinds.push(kind);
            }
        }

        let merged_perm = self.merge_lien_chains(v, perm_chains, direction);

        chains
    }

    /// Convert `ty` into list of bounding type chains from the given `direction`.
    async fn bounding_ty_chains(&self, ty: SymTy<'db>, direction: Direction) -> Vec<TyChain<'db>> {
        ToChain::new(self.env).ty_chains(ty, direction).await
    }

    fn merge_lien_chains(
        &self,
        v: InferVarIndex,
        lien_chains: Vec<LienChain<'db>>,
        direction: Direction,
    ) -> SymPerm<'db> {
        let mut lien_chains = lien_chains.into_iter();

        let Some(first) = lien_chains.next() else {
            return SymPerm::my(self.db);
        };

        let mut merged = self.lien_chain_to_perm(first);
        for lien_chain in lien_chains {
            merged = match self.merge_perm_and_lien_chain(merged, lien_chain, direction) {
                Ok(perm) => perm,
                Err(IrreconciliablePerms { left, right }) => {
                    // FIXME: This error stinks. We need better spans threaded through inference to do better, though.
                    let infer_var_span = self
                        .env
                        .runtime()
                        .with_inference_var_data(v, |data| data.span());
                    return SymPerm::err(
                        self.db,
                        Diagnostic::error(self.db, infer_var_span, "irreconciliable permissions")
                            .label(
                                self.db,
                                Level::Error,
                                infer_var_span,
                                format!("permission 1 is {left:?}"),
                            )
                            .label(
                                self.db,
                                Level::Error,
                                infer_var_span,
                                format!("permission 2 is {right:?}"),
                            )
                            .report(self.db),
                    );
                }
            };
        }

        merged
    }

    /// Merge `unmerged_chain` into `merged_perm` (which must be a merged perm produced by us),
    /// returning either a new merged perm or else an error.
    fn merge_perm_and_lien_chain(
        &self,
        merged_perm: SymPerm<'db>,
        unmerged_chain: LienChain<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        let unmerged_perm = self.lien_chain_to_perm(unmerged_chain);
        if self.is_my_perm(merged_perm) {
            self.merge_my_perm_and_perm(merged_perm, unmerged_perm, direction)
        } else if self.is_our_perm(unmerged_perm) {
            self.merge_my_perm_and_perm(unmerged_perm, merged_perm, direction)
        } else if self.is_our_perm(merged_perm) {
            self.merge_our_perm_and_perm(merged_perm, unmerged_perm, direction)
        } else if self.is_our_perm(unmerged_perm) {
            self.merge_our_perm_and_perm(unmerged_perm, merged_perm, direction)
        } else {
            self.merge_other_perms(merged_perm, unmerged_perm, direction)
        }
    }

    /// Merge a "my" (fully owned) permission chain with `other`.
    fn merge_my_perm_and_perm(
        &self,
        my_perm: SymPerm<'db>,
        other_perm: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        assert!(self.is_my_perm(my_perm));
        if self.is_my_perm(other_perm) {
            Ok(my_perm)
        } else if self.is_our_perm(other_perm) || self.is_shared_perm(other_perm) {
            // my <: our <: shared
            match direction {
                Direction::LowerBoundedBy => {
                    // We need a subtype, so take the "my" permission.
                    Ok(my_perm)
                }

                Direction::UpperBoundedBy => {
                    // We need a supertype, so take the our/shared permission.
                    Ok(other_perm)
                }
            }
        } else if self.is_leased_perm(other_perm) {
            Err(IrreconciliablePerms {
                left: my_perm,
                right: other_perm,
            })
        } else {
            unreachable!()
        }
    }

    /// Merge a "our" (fully owned) permission chain with `lien_chain` (which cannot be `my`).
    fn merge_our_perm_and_perm(
        &self,
        our_perm: SymPerm<'db>,
        other_perm: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        assert!(self.is_our_perm(our_perm));
        assert!(!self.is_my_perm(other_perm));

        if self.is_our_perm(other_perm) {
            Ok(our_perm)
        } else if self.is_shared_perm(other_perm) {
            // our <: shared
            match direction {
                Direction::LowerBoundedBy => {
                    // We need a subtype, so take the "our" permission.
                    Ok(our_perm)
                }

                Direction::UpperBoundedBy => {
                    // We need a supertype, so take the our/shared permission.
                    Ok(other_perm)
                }
            }
        } else if self.is_leased_perm(other_perm) {
            Err(IrreconciliablePerms {
                left: our_perm,
                right: other_perm,
            })
        } else {
            unreachable!()
        }
    }

    /// Merge two permissions, neither of which is `my` or `our` (those are handled specially).
    fn merge_other_perms(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        match direction {
            Direction::LowerBoundedBy => self.merge_other_perms_glb(left, right),
            Direction::UpperBoundedBy => self.merge_other_perms_lub(left, right),
        }
    }

    /// Compute mutual subtype of two shared permissions (greatest lower bound).
    ///
    /// Since the result must be a subtype, we want the intersection of the two permissions--
    /// something that is true for both left *and* right.
    ///
    /// Become it comes from a lien, the right permission never has more than one place,
    /// but the left may.
    ///
    /// Examples:
    ///
    /// * (`shared[a]`, `shared[b]`) = error
    /// * (`shared[a]`, `leased[b]`) = error
    /// * (`shared[a]`, `X`) = error
    /// * (`shared[a]`, `shared[a.b]`) = `shared[a.b]`
    /// * (`shared[a, b]`, `shared[a]`) = `shared[a]`
    /// * (`shared[a, b]`, `shared[a.b]`) = `shared[a.b]`
    /// * (`leased[a, b]`, `leased[a.b]`) = `leased[a.b]`
    /// * (`shared[a]`, `shared[a]`) = `shared[a]`
    /// * (`shared[a] shared[b]`, `shared[a]`) = `shared[a] shared[b]`
    /// * (`shared[a] shared[b]`, `shared[c]`) = error
    /// * (`shared[a] X`, `shared[a] Y`) = error
    /// * (`shared[a] X`, `shared[a.b] X`) = `shared[a.b] X`
    /// * (`leased[a] X`, `leased[a] Y`) = error
    /// * (`leased[a] X`, `leased[a.b] X`) = `leased[a.b] X`
    /// * (`X`, `X`) = `X`
    /// * (`X`, `Y`) = error
    fn merge_other_perms_glb(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        let mut left_leaves = left.leaves(self.db).peekable();
        let mut right_leaves = right.leaves(self.db).peekable();

        let mut merged_leaves = vec![];

        while left_leaves.peek().is_some() && right_leaves.peek().is_some() {
            let left_leaf = left_leaves.next().unwrap();
            let right_leaf = right_leaves.next().unwrap();

            match (left_leaf.kind(self.db), right_leaf.kind(self.db)) {
                // Handled by previous cases.
                (SymPermKind::My, _) | (_, SymPermKind::My) => unreachable!(),
                (SymPermKind::Our, _) | (_, SymPermKind::Our) => unreachable!(),

                // Not leaves.
                (SymPermKind::Apply(..), _) | (_, SymPermKind::Apply(..)) => unreachable!(),

                // Never part of the merging process.
                (SymPermKind::Infer(_), _) | (_, SymPermKind::Infer(_)) => unreachable!(),

                // Propagate errors.
                (SymPermKind::Error(reported), _) | (_, SymPermKind::Error(reported)) => {
                    return Ok(SymPerm::err(self.db, *reported));
                }

                // Incompatible
                (SymPermKind::Shared(_), SymPermKind::Leased(_))
                | (SymPermKind::Leased(_), SymPermKind::Shared(_))
                | (SymPermKind::Shared(_), SymPermKind::Var(_))
                | (SymPermKind::Var(_), SymPermKind::Shared(_))
                | (SymPermKind::Leased(_), SymPermKind::Var(_))
                | (SymPermKind::Var(_), SymPermKind::Leased(_)) => {
                    return Err(IrreconciliablePerms { left, right });
                }

                (SymPermKind::Shared(left_places), SymPermKind::Shared(right_places)) => {
                    merged_leaves.push(SymPerm::shared(
                        self.db,
                        self.intersect_places(left, left_places, right, right_places)?,
                    ))
                }

                (SymPermKind::Leased(left_places), SymPermKind::Leased(right_places)) => {
                    merged_leaves.push(SymPerm::leased(
                        self.db,
                        self.intersect_places(left, left_places, right, right_places)?,
                    ));
                }

                (SymPermKind::Var(left_variable), SymPermKind::Var(right_variable)) => {
                    if left_variable == right_variable {
                        merged_leaves.push(left_leaf);
                    } else {
                        return Err(IrreconciliablePerms { left, right });
                    }
                }
            }
        }

        assert!(merged_leaves.len() >= 1);

        Ok(merged_leaves
            .into_iter()
            .reduce(|a, b| SymPerm::apply(self.db, a, b))
            .unwrap())
    }

    /// Compute the intersection of `left_places` and `right_places`.
    /// `right_places` must have length 1.
    ///
    /// Examples:
    ///
    /// * `([a], [b]) = error`
    /// * `([a], [a]) = [a]`
    /// * `([a], [a.b]) = [a.b]`
    /// * `([a, c], [a.b]) = [a.b]`
    /// * `([a, c], [c.d]) = [c.d]`
    fn intersect_places(
        &self,
        left: SymPerm<'db>,
        left_places: &[SymPlace<'db>],
        right: SymPerm<'db>,
        right_places: &[SymPlace<'db>],
    ) -> Result<Vec<SymPlace<'db>>, IrreconciliablePerms<'db>> {
        assert_eq!(right_places.len(), 1);
        let right_place = right_places[0];
        if left_places.iter().any(|p| p.covers(self.db, right_place)) {
            Ok(vec![right_place])
        } else {
            Err(IrreconciliablePerms { left, right })
        }
    }

    /// Compute mutual supertype of two permissions (least upper bound).
    ///
    /// Since the result must be a supertype, we want something that genearlizes left and right,
    /// but we can lose specificity.
    ///
    /// Become it comes from a lien, the right permission never has more than one place,
    /// but the left may.
    ///
    /// Examples:
    ///
    /// * (`shared[a]`, `shared[b]`) = `shared[a, b]`
    /// * (`shared[a]`, `X`) = error
    /// * (`shared[a]`, `leased[b]`) = error
    /// * (`shared[a]`, `shared[a.b]`) = `shared[a]`
    /// * (`shared[a, b]`, `shared[a]`) = `shared[a, b]`
    /// * (`shared[a, b]`, `shared[a.b]`) = `shared[a, b]`
    /// * (`leased[a, b]`, `leased[a.b]`) = `leased[a, b]`
    /// * (`shared[a]`, `shared[a]`) = `shared[a]`
    /// * (`shared[a] shared[b]`, `shared[a]`) = `shared[a]`
    /// * (`shared[a] shared[b]`, `shared[c]`) = `shared[a, c]`
    /// * (`shared[a] shared[b]`, `shared[c] shared[b]`) = `shared[a, c] shared[b]`
    /// * (`shared[a] X`, `shared[a] Y`) = error (*)
    /// * (`shared[a] X`, `shared[a.b] X`) = `shared[a] X`
    /// * (`leased[a] X`, `leased[a] Y`) = error (*)
    /// * (`leased[a] X`, `leased[a.b] X`) = `leased[a] X`
    /// * (`X`, `X`) = `X`
    /// * (`X`, `Y`) = error (*)
    ///
    /// (*) Conceivably could be computed with better where-clauses.
    fn merge_other_perms_lub(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
    ) -> Result<SymPerm<'db>, IrreconciliablePerms<'db>> {
        let mut left_leaves = left.leaves(self.db).peekable();
        let mut right_leaves = right.leaves(self.db).peekable();

        let mut merged_leaves = vec![];

        while left_leaves.peek().is_some() && right_leaves.peek().is_some() {
            let left_leaf = left_leaves.next().unwrap();
            let right_leaf = right_leaves.next().unwrap();

            match (left_leaf.kind(self.db), right_leaf.kind(self.db)) {
                // Either not part of merging process or ruled out by earlier phases.
                (SymPermKind::My, _) | (_, SymPermKind::My) => unreachable!(),
                (SymPermKind::Our, _) | (_, SymPermKind::Our) => unreachable!(),
                (SymPermKind::Apply(..), _) | (_, SymPermKind::Apply(..)) => unreachable!(),
                (SymPermKind::Infer(_), _) | (_, SymPermKind::Infer(_)) => unreachable!(),

                // Propagate errors.
                (SymPermKind::Error(reported), _) | (_, SymPermKind::Error(reported)) => {
                    return Ok(SymPerm::err(self.db, *reported));
                }

                // Incompatible
                (SymPermKind::Shared(_), SymPermKind::Leased(_))
                | (SymPermKind::Leased(_), SymPermKind::Shared(_))
                | (SymPermKind::Shared(_), SymPermKind::Var(_))
                | (SymPermKind::Var(_), SymPermKind::Shared(_))
                | (SymPermKind::Leased(_), SymPermKind::Var(_))
                | (SymPermKind::Var(_), SymPermKind::Leased(_)) => {
                    return Err(IrreconciliablePerms { left, right });
                }

                // Leaves.
                (SymPermKind::Shared(left_places), SymPermKind::Shared(right_places)) => {
                    merged_leaves.push(SymPerm::shared(
                        self.db,
                        self.union_places(left_places, right_places),
                    ));
                }

                (SymPermKind::Leased(left_places), SymPermKind::Leased(right_places)) => {
                    merged_leaves.push(SymPerm::leased(
                        self.db,
                        self.union_places(left_places, right_places),
                    ));
                }

                (SymPermKind::Var(left_variable), SymPermKind::Var(right_variable)) => {
                    if left_variable == right_variable {
                        merged_leaves.push(left_leaf);
                    } else {
                        return Err(IrreconciliablePerms { left, right });
                    }
                }
            }
        }

        assert!(merged_leaves.len() >= 1);

        Ok(merged_leaves
            .into_iter()
            .reduce(|a, b| SymPerm::apply(self.db, a, b))
            .unwrap())
    }

    /// Compute the union of `left_places` and `right_places`.
    /// `right_places` must have length 1.
    ///
    /// Examples:
    ///
    /// * `([a], [b]) = [a, b]`
    /// * `([a], [a]) = [a]`
    /// * `([a], [a.b]) = [a]`
    /// * `([a, c], [a.b]) = [a, c]`
    /// * `([a, c], [c.d]) = [a, c]`
    /// * `([a.b], [a]) = [a]`
    fn union_places(
        &self,
        left_places: &[SymPlace<'db>],
        right_places: &[SymPlace<'db>],
    ) -> Vec<SymPlace<'db>> {
        assert_eq!(right_places.len(), 1);
        let right_place = right_places[0];

        if left_places.iter().any(|p| p.covers(self.db, right_place)) {
            left_places.to_vec()
        } else {
            left_places
                .iter()
                .copied()
                .filter(|p| !right_place.covers(self.db, *p))
                .chain(std::iter::once(right_place))
                .collect()
        }
    }

    /// Convert a `LienChain` into a `SymPerm`.
    fn lien_chain_to_perm(&self, lien_chain: LienChain<'db>) -> SymPerm<'db> {
        self.liens_to_perm(&lien_chain.links)
    }

    /// Convert a list of `Lien`s into a `SymPerm`.
    fn liens_to_perm(&self, liens: &[Lien<'db>]) -> SymPerm<'db> {
        let Some((first, rest)) = liens.split_first() else {
            return SymPerm::my(self.db);
        };

        let first_perm = match *first {
            Lien::Our => {
                assert!(rest.is_empty());
                return SymPerm::our(self.db);
            }

            Lien::Shared(sym_place) => SymPerm::shared(self.db, vec![sym_place]),
            Lien::Leased(sym_place) => SymPerm::leased(self.db, vec![sym_place]),
            Lien::Var(sym_variable) => SymPerm::var(self.db, sym_variable),
            Lien::Error(reported) => return SymPerm::err(self.db, reported),
        };

        if rest.is_empty() {
            first_perm
        } else {
            let rest_perms = self.liens_to_perm(rest);
            SymPerm::apply(self.db, first_perm, rest_perms)
        }
    }

    /// Return true if `perm` is [`SymPermKind::My`][].
    /// We never produce a `my` permission underneath an `apply` node.
    fn is_my_perm(&self, perm: SymPerm<'db>) -> bool {
        match perm.kind(self.db) {
            SymPermKind::My => true,
            SymPermKind::Apply(_, _) => false,
            SymPermKind::Our => false,
            SymPermKind::Shared(_) => false,
            SymPermKind::Leased(_) => false,
            SymPermKind::Infer(_) => false,
            SymPermKind::Var(_) => false,
            SymPermKind::Error(_) => false,
        }
    }

    /// Return true if `perm` is [`SymPermKind::Our`][].
    /// We never produce a `our` permission underneath an `apply` node.
    fn is_our_perm(&self, perm: SymPerm<'db>) -> bool {
        match perm.kind(self.db) {
            SymPermKind::Our => true,
            SymPermKind::Apply(_, _) => false,
            SymPermKind::My => false,
            SymPermKind::Shared(_) => false,
            SymPermKind::Leased(_) => false,
            SymPermKind::Infer(_) => false,
            SymPermKind::Var(_) => false,
            SymPermKind::Error(_) => false,
        }
    }

    /// Return true if `perm` is [`SymPermKind::Shared`][].
    /// This can appear on the left side of an `apply` node.
    fn is_shared_perm(&self, perm: SymPerm<'db>) -> bool {
        match *perm.kind(self.db) {
            SymPermKind::Shared(_) => true,
            SymPermKind::Apply(left, _) => self.is_shared_perm(left),
            SymPermKind::My => false,
            SymPermKind::Our => false,
            SymPermKind::Leased(_) => false,
            SymPermKind::Infer(_) => false,
            SymPermKind::Var(_) => false,
            SymPermKind::Error(_) => false,
        }
    }

    /// Return true if `perm` is [`SymPermKind::Leased`][].
    /// This can appear on the left side of an `apply` node.
    fn is_leased_perm(&self, perm: SymPerm<'db>) -> bool {
        match *perm.kind(self.db) {
            SymPermKind::Leased(_) => true,
            SymPermKind::Apply(left, _) => self.is_leased_perm(left),
            SymPermKind::My => false,
            SymPermKind::Our => false,
            SymPermKind::Shared(_) => false,
            SymPermKind::Infer(_) => false,
            SymPermKind::Var(_) => false,
            SymPermKind::Error(_) => false,
        }
    }
}

struct IrreconciliablePerms<'db> {
    left: SymPerm<'db>,
    right: SymPerm<'db>,
}
