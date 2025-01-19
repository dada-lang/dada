//! Code to resolve inference variables to concrete types and permissions.

use dada_ir_ast::diagnostic::{Diagnostic, Err, Level};
use dada_util::Set;
use futures::StreamExt;

use crate::{
    check::chains::ToChain,
    ir::{
        indices::InferVarIndex,
        subst::Subst,
        types::{
            SymGenericKind, SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind,
            Variance,
        },
    },
};

use super::{
    Env,
    bound::Direction,
    chains::{Lien, LienChain, TyChain, TyChainKind},
    subtype_check::is_subterm,
};

pub struct Resolver<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env Env<'db>,
    var_stack: Set<InferVarIndex>,
}

impl<'env, 'db> Resolver<'env, 'db> {
    pub fn new(env: &'env Env<'db>) -> Self {
        assert!(
            env.runtime().check_complete(),
            "resolution is only possible once type constraints are known"
        );

        Self {
            db: env.db(),
            env,
            var_stack: Default::default(),
        }
    }

    /// Return a version of `term` in which all inference variables are (deeply) removed.
    pub fn resolve_term<T>(&mut self, term: T) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>>,
    {
        term.resolve_infer_var(self.db, |v| Some(self.resolve_infer_var(v)))
    }

    /// Resolve an inference variable to a generic term, given the variance of the location in which it appears
    fn resolve_infer_var(&mut self, v: InferVarIndex) -> SymGenericTerm<'db> {
        if self.var_stack.insert(v) {
            let result = match self.env.infer_var_kind(v) {
                SymGenericKind::Type => self.resolve_ty_var(v).into(),
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
    fn resolve_ty_var(&mut self, v: InferVarIndex) -> SymGenericTerm<'db> {
        let (lower_bound, upper_bound): (Option<SymGenericTerm<'db>>, Option<SymGenericTerm<'db>>) =
            match self.env.infer_var_kind(v) {
                SymGenericKind::Type => (
                    self.bounding_ty(v, Direction::LowerBoundedBy)
                        .map(|ty| ty.into()),
                    self.bounding_ty(v, Direction::UpperBoundedBy)
                        .map(|ty| ty.into()),
                ),
                SymGenericKind::Perm => (
                    self.bounding_perm(v, Direction::LowerBoundedBy)
                        .map(|perm| perm.into()),
                    self.bounding_perm(v, Direction::UpperBoundedBy)
                        .map(|perm| perm.into()),
                ),
                SymGenericKind::Place => unimplemented!("place variable inference"),
            };

        match (lower_bound, upper_bound) {
            (Some(bound), None) | (None, Some(bound)) => bound,
            (Some(lower_bound), Some(upper_bound)) => {
                // Here is the challenge. We know that each of the upper bounds, individually, was a supertype
                // of each of the lower bounds. But that does not make the LUB a supertype of the GLB.
                if lower_bound == upper_bound || is_subterm(self.env, lower_bound, upper_bound) {
                    upper_bound
                } else {
                    // For now, we just error.
                    self.report_irreconciliable_error(v, lower_bound.into(), upper_bound.into())
                }
            }
            (None, None) => {
                // No bounds on this type variable.
                // What should we pick?
                // Or should we error?
                todo!()
            }
        }
    }

    /// Return the bounding type on the type inference variable `v` from the given `direction`.
    fn bounding_ty(&mut self, v: InferVarIndex, direction: Direction) -> Option<SymTy<'db>> {
        self.env.runtime().assert_check_complete(async {
            let mut ty_chains = vec![];
            let mut bounds = self.env.transitive_ty_var_bounds(v, direction);
            while let Some(ty) = bounds.next().await {
                ToChain::new(self.env)
                    .push_ty_chains(ty, direction, &mut ty_chains)
                    .await;
            }

            if ty_chains.is_empty() {
                return None;
            }

            match self.merge_ty_chains(ty_chains, direction) {
                Ok(ty) => Some(ty),
                Err(Irreconciliable { left, right }) => {
                    Some(self.report_irreconciliable_error(v, left, right))
                }
            }
        })
    }

    /// Return the bounding perm on the permission inference variable `v` from the given `direction`.
    fn bounding_perm(&mut self, v: InferVarIndex, direction: Direction) -> Option<SymPerm<'db>> {
        self.env.runtime().assert_check_complete(async {
            let mut lien_chains = vec![];
            let mut bounds = self.env.transitive_perm_var_bounds(v, direction);
            while let Some(perm) = bounds.next().await {
                ToChain::new(self.env)
                    .push_lien_chains(perm, direction, &mut lien_chains)
                    .await;
            }

            if lien_chains.is_empty() {
                return None;
            }

            match self.merge_lien_chains(lien_chains, direction) {
                Ok(perm) => Some(perm),
                Err(Irreconciliable { left, right }) => {
                    Some(self.report_irreconciliable_error(v, left, right))
                }
            }
        })
    }

    /// Merge a list of type chains, computing their LUB or GLB depending on `direction`.
    fn merge_ty_chains(
        &mut self,
        ty_chains: Vec<TyChain<'db>>,
        direction: Direction,
    ) -> Result<SymTy<'db>, Irreconciliable<'db>> {
        // First find the bounding type chains. These may contain inference variables but only in generic arguments.
        let mut perm_chains = vec![];
        let mut type_chain_kinds = vec![];
        for TyChain { lien, kind } in ty_chains {
            perm_chains.push(lien);
            type_chain_kinds.push(kind);
        }

        let merged_perm = self.merge_lien_chains(perm_chains, direction)?;
        let merged_ty = self.merge_ty_chain_kinds(type_chain_kinds, direction)?;
        Ok(merged_perm.apply_to_ty(self.db, merged_ty))
    }

    /// Merge a list of lien chains, computing their LUB or GLB depending on `direction`.
    fn merge_lien_chains(
        &self,
        lien_chains: Vec<LienChain<'db>>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        let mut lien_chains = lien_chains.into_iter();

        let Some(first) = lien_chains.next() else {
            return Ok(SymPerm::my(self.db));
        };

        let mut merged_perm = self.lien_chain_to_perm(first);
        for unmerged_chain in lien_chains {
            let unmerged_perm = self.lien_chain_to_perm(unmerged_chain);
            merged_perm = self.merge_resolved_perms(merged_perm, unmerged_perm, direction)?;
        }

        Ok(merged_perm)
    }

    fn merge_ty_chain_kinds(
        &mut self,
        type_chain_kinds: Vec<TyChainKind<'db>>,
        direction: Direction,
    ) -> Result<SymTy<'db>, Irreconciliable<'db>> {
        let mut type_chain_kinds = type_chain_kinds.into_iter();

        let Some(first) = type_chain_kinds.next() else {
            return Ok(SymTy::never(self.db));
        };

        let mut merged_ty = self.ty_chain_kind_to_ty(first);
        for unmerged_kind in type_chain_kinds {
            let unmerged_ty = self.ty_chain_kind_to_ty(unmerged_kind);
            merged_ty = self.merge_ty_chain_kind_tys(merged_ty, unmerged_ty, direction)?;
        }

        Ok(merged_ty)
    }

    fn ty_chain_kind_to_ty(&self, kind: TyChainKind<'db>) -> SymTy<'db> {
        match kind {
            TyChainKind::Error(reported) => SymTy::err(self.db, reported),
            TyChainKind::Named(n, args) => SymTy::named(self.db, n, args),
            TyChainKind::Never => SymTy::never(self.db),
            TyChainKind::Var(v) => SymTy::var(self.db, v),
        }
    }

    /// Merge `left` and `right` producing the lub or glb according to `direction`
    ///
    /// `left` and `right` cannot contain inference variables.
    fn merge_resolved_perms(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        if self.is_my_perm(left) {
            self.merge_my_perm_and_perm(left, right, direction)
        } else if self.is_my_perm(right) {
            self.merge_my_perm_and_perm(right, left, direction)
        } else if self.is_our_perm(left) {
            self.merge_our_perm_and_perm(left, right, direction)
        } else if self.is_our_perm(right) {
            self.merge_our_perm_and_perm(right, left, direction)
        } else {
            self.merge_other_perms(left, right, direction)
        }
    }

    /// Merge a "my" (fully owned) permission chain with `other`.
    fn merge_my_perm_and_perm(
        &self,
        my_perm: SymPerm<'db>,
        other_perm: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        assert!(self.is_my_perm(my_perm));
        if self.is_my_perm(other_perm) {
            Ok(my_perm)
        } else if self.meets_shared_bound(other_perm) {
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
        } else {
            Err(Irreconciliable::new(my_perm, other_perm))
        }
    }

    /// Merge a "our" (fully owned) permission chain with `lien_chain` (which cannot be `my`).
    fn merge_our_perm_and_perm(
        &self,
        our_perm: SymPerm<'db>,
        other_perm: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        assert!(self.is_our_perm(our_perm));
        assert!(!self.is_my_perm(other_perm));

        if self.is_our_perm(other_perm) {
            Ok(our_perm)
        } else if self.meets_shared_bound(other_perm) {
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
        } else {
            Err(Irreconciliable::new(our_perm, other_perm))
        }
    }

    /// Merge two permissions, neither of which is `my` or `our` (those are handled specially).
    fn merge_other_perms(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        match direction {
            Direction::LowerBoundedBy => self.merge_other_perms_glb(left, right),
            Direction::UpperBoundedBy => self.merge_other_perms_lub(left, right),
        }
    }

    /// Compute mutual subtype of two permissions (greatest lower bound).
    /// Neither permission can be `my` or `our`.
    ///
    /// Since the result must be a subtype, we want the intersection of the two permissions--
    /// something that is true for both left *and* right.
    ///
    /// Examples (same results hold in reverse):
    ///
    /// * (`shared[a]`, `shared[b]`) = error
    /// * (`shared[a]`, `leased[b]`) = error
    /// * (`shared[a]`, `X`) = error
    /// * (`shared[a]`, `shared[a.b]`) = `shared[a.b]`
    /// * (`shared[a, b]`, `shared[a]`) = `shared[a]`
    /// * (`shared[a, b]`, `shared[a.b]`) = `shared[a.b]`
    /// * (`leased[a, b]`, `leased[a.b]`) = `leased[a.b]`
    /// * (`shared[a, b]`, `shared[a, c]`) = `shared[a]`
    /// * (`shared[a, b]`, `shared[a.b, c]`) = `shared[a.b]`
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
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        let mut left_leaves = left.leaves(self.db).peekable();
        let mut right_leaves = right.leaves(self.db).peekable();

        let mut merged_leaves = vec![];

        while left_leaves.peek().is_some() && right_leaves.peek().is_some() {
            let left_leaf = left_leaves.next().unwrap();
            let right_leaf = right_leaves.next().unwrap();

            match (left_leaf.kind(self.db), right_leaf.kind(self.db)) {
                // Handled by previous cases.
                (SymPermKind::My, _)
                | (_, SymPermKind::My)
                | (SymPermKind::Our, _)
                | (_, SymPermKind::Our)
                | (SymPermKind::Apply(..), _)
                | (_, SymPermKind::Apply(..))
                | (SymPermKind::Infer(_), _)
                | (_, SymPermKind::Infer(_)) => {
                    unreachable!("unexpected permission kinds {left_leaf:?} and {right_leaf:?}")
                }

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
                    return Err(Irreconciliable::new(left, right));
                }

                (SymPermKind::Shared(left_places), SymPermKind::Shared(right_places)) => {
                    merged_leaves.push(SymPerm::shared(
                        self.db,
                        self.intersect_places(left_places, right_places)?,
                    ))
                }

                (SymPermKind::Leased(left_places), SymPermKind::Leased(right_places)) => {
                    merged_leaves.push(SymPerm::leased(
                        self.db,
                        self.intersect_places(left_places, right_places)?,
                    ));
                }

                (SymPermKind::Var(left_variable), SymPermKind::Var(right_variable)) => {
                    if left_variable == right_variable {
                        merged_leaves.push(left_leaf);
                    } else {
                        return Err(Irreconciliable::new(left, right));
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
    ///
    /// Examples:
    ///
    /// * `([a], [b]) = error`
    /// * `([a], [a]) = [a]`
    /// * `([a], [a.b]) = [a.b]`
    /// * `([a.b.c], [a.b]) = [a.b.c]`
    /// * `([a, c], [a.b]) = [a.b]`
    /// * `([a, c], [c.d]) = [c.d]`
    /// * `([a.b.c], [a, a.b, a.b.c.d]) = [a.b.c]` (\*)
    /// * `([a], [a.b, a, a.b.c.d]) = [a]` (\*)
    ///
    /// (\*) We generally expect minimized sets of places, but this
    /// function can tolerate non-minimized inputs. It always produces
    /// minimized output (as shown).
    fn intersect_places(
        &self,
        left_places: &[SymPlace<'db>],
        right_places: &[SymPlace<'db>],
    ) -> Result<Vec<SymPlace<'db>>, Irreconciliable<'db>> {
        let mut intersected_places = vec![];

        for &left_place in left_places {
            for &right_place in right_places {
                if left_place.covers(self.db, right_place) {
                    intersected_places.push(right_place);
                } else if right_place.covers(self.db, left_place) {
                    intersected_places.push(left_place);
                }
            }
        }

        Ok(self.minimize_places(intersected_places))
    }

    /// Removes duplicates from `places` or elements that are covered by another element.
    /// For example, `[a, a, a.b, a.b.c]` becomes `[a]`.
    fn minimize_places(&self, places: Vec<SymPlace<'db>>) -> Vec<SymPlace<'db>> {
        let mut minimized_places: Vec<SymPlace<'db>> = vec![];

        for place in places {
            if minimized_places.iter().any(|&mp| mp.covers(self.db, place)) {
                continue;
            }

            minimized_places.retain(|&mp| !place.covers(self.db, mp));

            minimized_places.push(place);
        }

        minimized_places
    }

    /// Compute mutual supertype of two permissions (least upper bound).
    ///
    /// Since the result must be a supertype, we want something that genearlizes left and right,
    /// but we can lose specificity.
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
    /// * (`X`, `Y`) = error (\*)
    ///
    /// (\*) Conceivably could be computed with better where-clauses.
    fn merge_other_perms_lub(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
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
                    return Err(Irreconciliable::new(left, right));
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
                        return Err(Irreconciliable::new(left, right));
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
    /// * (`[a.b.c]`, `[a, a.b, a.b.c.d]`) = `[a]` (\*\*)
    /// * (`[a]`, `[a.b, a, a.b.c.d]`) = `[a]` (\*\*)
    ///
    /// (\*) We generally expect minimized sets of places, but this
    /// function can tolerate non-minimized inputs. It always produces
    /// minimized output (as shown).
    fn union_places(
        &self,
        left_places: &[SymPlace<'db>],
        right_places: &[SymPlace<'db>],
    ) -> Vec<SymPlace<'db>> {
        let mut unioned_places = vec![];

        for &left_place in left_places {
            for &right_place in right_places {
                if left_place.covers(self.db, right_place) {
                    unioned_places.push(left_place);
                } else if right_place.covers(self.db, left_place) {
                    unioned_places.push(right_place);
                }
            }
        }

        self.minimize_places(unioned_places)
    }

    /// Convert a `LienChain` into a `SymPerm`.
    fn lien_chain_to_perm(&self, lien_chain: LienChain<'db>) -> SymPerm<'db> {
        self.liens_to_perm(lien_chain.links())
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

    /// Return true if `perm` meets the `shared` bound.
    /// This can be [`SymPermKind::My`][], [`SymPermKind::Shared`][], [`SymPermKind::Our`][],
    /// or [`SymPermKind::Var`][] with a shared variable.
    /// This can appear on the left side of an `apply` node.
    fn meets_shared_bound(&self, perm: SymPerm<'db>) -> bool {
        match *perm.kind(self.db) {
            SymPermKind::Shared(_) => true,
            SymPermKind::Apply(left, _) => self.meets_shared_bound(left),
            SymPermKind::My => true,
            SymPermKind::Our => true,
            SymPermKind::Leased(_) => false,
            SymPermKind::Infer(_) => false,
            SymPermKind::Var(var) => self.env.is_copy_var(var),
            SymPermKind::Error(_) => false,
        }
    }

    /// Merge two types that resulted from type kinds.
    /// These types cannot have permissions nor inference variables.
    fn merge_ty_chain_kind_tys(
        &mut self,
        left: SymTy<'db>,
        right: SymTy<'db>,
        direction: Direction,
    ) -> Result<SymTy<'db>, Irreconciliable<'db>> {
        match (left.kind(self.db), right.kind(self.db)) {
            (&SymTyKind::Error(reported), _) | (_, &SymTyKind::Error(reported)) => {
                Ok(SymTy::err(self.db, reported))
            }

            // Never is only a subtype of itself.
            (SymTyKind::Never, SymTyKind::Never) => Ok(left),
            (SymTyKind::Never, _) | (_, SymTyKind::Never) => {
                return Err(Irreconciliable::new(left, right));
            }

            // Generic variables can only be related to themselves.
            (SymTyKind::Var(v1), SymTyKind::Var(v2)) => {
                if v1 == v2 {
                    Ok(left)
                } else {
                    return Err(Irreconciliable::new(left, right));
                }
            }
            (SymTyKind::Named(..), SymTyKind::Var(..))
            | (SymTyKind::Var(..), SymTyKind::Named(..)) => {
                return Err(Irreconciliable::new(left, right));
            }

            // Named types can only be related to themselves.
            (SymTyKind::Named(n1, args1), SymTyKind::Named(n2, args2)) => {
                if n1 == n2 {
                    // FIXME: variance
                    let variances = self.env.variances(*n1);
                    assert_eq!(args1.len(), variances.len());
                    assert_eq!(args1.len(), args2.len());
                    let resolved_args1 = self.resolve_term(args1);
                    let resolved_args2 = self.resolve_term(args2);
                    let generics = resolved_args1
                        .into_iter()
                        .zip(resolved_args2)
                        .zip(variances)
                        .map(|((a1, a2), v)| self.merge_generic_arguments(a1, a2, direction, v))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(SymTy::named(self.db, *n1, generics))
                } else {
                    Err(Irreconciliable::new(left, right))
                }
            }

            // Unreachable cases.
            (SymTyKind::Perm(..), _)
            | (_, SymTyKind::Perm(..))
            | (SymTyKind::Infer(_), _)
            | (_, SymTyKind::Infer(_)) => {
                unreachable!(
                    "unexpected type kinds in merge_ty_chain_kind_tys: ({left:?}, {right:?})"
                )
            }
        }
    }

    /// Merge two generic arguments, taking the variance of the generic argument into account.
    ///
    /// # Parameters
    ///
    /// * `left`, `right` -- the two generic arguments
    /// * `original_direction` -- whether we are computing the LUB or GLB of the the original type whose generic argument this is.
    /// * `variance` -- the variance of this argument in the original type
    fn merge_generic_arguments(
        &mut self,
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
        original_direction: Direction,
        variance: Variance,
    ) -> Result<SymGenericTerm<'db>, Irreconciliable<'db>> {
        // This is a bit subtle. The `original_direction` indicates whether we are trying to
        // find the LUB or GLB of two types -- say, `Foo[X]` and `Foo[Y]`.
        // We are trying to decide how to merge `X` and `Y` into `Z`.
        match variance {
            // For a covariant parameter, assuming direction is
            // `UpperBoundedBy` (LUB), we want `Z = LUB(X, Y)`.
            // So e.g. the common supertype of `Foo[our String]` and `Foo[my String]`
            // is `Foo[our String]`.
            //
            // If the direction is `LowerBoundedBy` (GLB), then e.g.
            // we have something like `Fn(our String)` and `Fn(my String)` and we want
            // the common *subtype*, `Fn(my String)`.
            Variance::Covariant => self.merge_generic_terms(left, right, original_direction),

            // As above, but reversed.
            Variance::Contravariant => {
                self.merge_generic_terms(left, right, original_direction.reverse())
            }

            // Invariance is tricky. If `Foo[X]` and `Foo[Y]` are invariant,
            // they can only be merged into a common sub- or super-type
            // if `X == Y`. So `Cell[our String]` and `Cell[my String]`
            // have no common super or subtypes!
            //
            // However `left` and `right` may have inference
            // variables etc so we have to resolve them before comparing for
            // equality.
            Variance::Invariant => {
                let left = self.resolve_term(left);
                let right = self.resolve_term(right);
                if left == right {
                    Ok(left)
                } else {
                    Err(Irreconciliable::new(left, right))
                }
            }
        }
    }

    /// Merge two generic terms, computing their LUB or GLB depending on `direction`.
    fn merge_generic_terms(
        &mut self,
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
        direction: Direction,
    ) -> Result<SymGenericTerm<'db>, Irreconciliable<'db>> {
        match (left, right) {
            (SymGenericTerm::Error(reported), _) | (_, SymGenericTerm::Error(reported)) => {
                Ok(SymGenericTerm::Error(reported))
            }

            (SymGenericTerm::Type(ty), SymGenericTerm::Type(ty2)) => {
                Ok(self.merge_tys(ty, ty2, direction)?.into())
            }
            (SymGenericTerm::Type(_), _) | (_, SymGenericTerm::Type(_)) => {
                unreachable!("kind mismatch")
            }

            (SymGenericTerm::Perm(perm), SymGenericTerm::Perm(perm2)) => {
                Ok(self.merge_unresolved_perms(perm, perm2, direction)?.into())
            }
            (SymGenericTerm::Perm(_), _) | (_, SymGenericTerm::Perm(_)) => {
                unreachable!("kind mismatch")
            }

            (SymGenericTerm::Place(_), SymGenericTerm::Place(_)) => {
                unreachable!("place generic argument")
            }
        }
    }

    /// Merge two types, computing their LUB or GLB depending on `direction`.
    fn merge_tys(
        &mut self,
        left: SymTy<'db>,
        right: SymTy<'db>,
        direction: Direction,
    ) -> Result<SymTy<'db>, Irreconciliable<'db>> {
        self.env.runtime().assert_check_complete(async {
            // FIXME: Should we be propagating context for the type chains?
            // Seems like yes, but that also suggests we need to rework this merging process a bit.
            let mut ty_chains = vec![];
            ToChain::new(self.env)
                .push_ty_chains(left, direction, &mut ty_chains)
                .await;
            ToChain::new(self.env)
                .push_ty_chains(right, direction, &mut ty_chains)
                .await;
            self.merge_ty_chains(ty_chains, direction)
        })
    }

    /// Merge two permissions, computing their LUB or GLB depending on `direction`.
    fn merge_unresolved_perms(
        &mut self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, Irreconciliable<'db>> {
        self.env.runtime().assert_check_complete(async {
            // FIXME: Should we be propagating context for the type chains?
            // Seems like yes, but that also suggests we need to rework this merging process a bit.
            let mut lien_chains = vec![];
            ToChain::new(self.env)
                .push_lien_chains(left, direction, &mut lien_chains)
                .await;
            ToChain::new(self.env)
                .push_lien_chains(right, direction, &mut lien_chains)
                .await;
            self.merge_lien_chains(lien_chains, direction)
        })
    }

    fn report_irreconciliable_error<T: Err<'db>>(
        &self,
        v: InferVarIndex,
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
    ) -> T {
        // FIXME: This error stinks. We need better spans threaded through inference to do better, though.
        // This would be an interesting place to deply AI.

        let (infer_var_kind, infer_var_span) = self
            .env
            .runtime()
            .with_inference_var_data(v, |data| (data.kind(), data.span()));

        let message = match infer_var_kind {
            SymGenericKind::Type => "cannot infer a type due to conflicting constraints",
            SymGenericKind::Perm => "cannot infer a permission due to conflicting constraints",
            SymGenericKind::Place => "cannot infer a place due to conflicting constraints",
        };
        return T::err(
            self.db,
            Diagnostic::error(self.db, infer_var_span, message)
                .label(
                    self.db,
                    Level::Error,
                    infer_var_span,
                    format!("constraint 1 is {left:?}"),
                )
                .label(
                    self.db,
                    Level::Error,
                    infer_var_span,
                    format!("constraint 2 is {right:?}"),
                )
                .report(self.db),
        );
    }
}

struct Irreconciliable<'db> {
    left: SymGenericTerm<'db>,
    right: SymGenericTerm<'db>,
}

impl<'db> Irreconciliable<'db> {
    pub fn new(
        left: impl Into<SymGenericTerm<'db>>,
        right: impl Into<SymGenericTerm<'db>>,
    ) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}
