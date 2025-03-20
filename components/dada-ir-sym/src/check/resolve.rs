//! Code to resolve inference variables to concrete types and permissions.

use dada_ir_ast::diagnostic::{Diagnostic, Err, Level, Reported};
use dada_util::Set;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    subst::Subst,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind},
};

use super::{
    Env,
    inference::{Direction, InferVarKind},
    predicates::Predicate,
    red::{Chain, Lien, RedTy},
};

pub struct Resolver<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env mut Env<'db>,
    var_stack: Set<InferVarIndex>,
}

impl<'env, 'db> Resolver<'env, 'db> {
    pub fn new(env: &'env mut Env<'db>) -> Self {
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
    pub fn resolve<T>(&mut self, term: T) -> T::Output
    where
        T: Subst<'db, GenericTerm = SymGenericTerm<'db>>,
    {
        let mut bound_vars = self.env.bound_vars();
        term.resolve_infer_var(self.db, &mut bound_vars, |infer| {
            match self.resolve_infer_var(infer) {
                Ok(v) => Some(v),
                Result::Err(error) => Some(SymGenericTerm::err(self.db, self.report(infer, error))),
            }
        })
    }

    /// Resolve an inference variable to a generic term, given the variance of the location in which it appears
    fn resolve_infer_var(
        &mut self,
        infer: InferVarIndex,
    ) -> Result<SymGenericTerm<'db>, ResolverError<'db>> {
        if self.var_stack.insert(infer) {
            let mut compute_result = || -> Result<SymGenericTerm<'db>, ResolverError<'db>> {
                match self.env.infer_var_kind(infer) {
                    InferVarKind::Type => {
                        if let Some(ty) = self.bounding_ty(infer, Direction::FromBelow)? {
                            Ok(ty.into())
                        } else if let Some(ty) = self.bounding_ty(infer, Direction::FromAbove)? {
                            Ok(ty.into())
                        } else {
                            Err(ResolverError::NoBounds)
                        }
                    }

                    InferVarKind::Perm => {
                        Ok(self.bounding_perm(infer, Direction::FromBelow)?.into())
                    }
                }
            };

            let result = compute_result();
            self.var_stack.remove(&infer);
            result
        } else {
            Err(ResolverError::Cycle)
        }
    }

    fn report(&self, infer: InferVarIndex, err: ResolverError<'db>) -> Reported {
        let span = self.env.infer_var_span(infer);
        match err {
            ResolverError::NoBounds => {
                Diagnostic::error(self.db, span, "no bounds found for inference variable")
                    .report(self.db)
            }
            ResolverError::Cycle => {
                Diagnostic::error(self.db, span, "cyclic bounds found for inference variable")
                    .report(self.db)
            }
            ResolverError::Irreconciliable { left, right } => {
                self.report_irreconciliable_error(infer, left, right)
            }
        }
    }

    fn report_irreconciliable_error<T: Err<'db>>(
        &self,
        infer: InferVarIndex,
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
    ) -> T {
        // FIXME: This error stinks. We need better spans threaded through inference to do better, though.
        // This would be an interesting place to deply AI.

        let (infer_var_kind, infer_var_span) = self
            .env
            .runtime()
            .with_inference_var_data(infer, |data| (data.kind(), data.span()));

        let message = match infer_var_kind {
            InferVarKind::Type => "cannot infer a type due to conflicting constraints",
            InferVarKind::Perm => "cannot infer a permission due to conflicting constraints",
        };
        T::err(
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
        )
    }

    /// Return the bounding type on the type inference variable `v` from the given `direction`.
    fn bounding_ty(
        &mut self,
        infer: InferVarIndex,
        direction: Direction,
    ) -> Result<Option<SymTy<'db>>, ResolverError<'db>> {
        let db = self.env.db();

        let bound = self.env.red_ty_bound(infer, direction).peek();

        let Some((red_ty, _)) = bound else {
            return Ok(None);
        };

        let sym_ty = match red_ty {
            RedTy::Error(reported) => SymTy::err(db, reported),
            RedTy::Named(name, args) => {
                let args = self.resolve(args);
                SymTy::new(db, SymTyKind::Named(name, args))
            }
            RedTy::Never => SymTy::new(db, SymTyKind::Never),
            RedTy::Infer(_) => panic!("infer bound cannot be another infer"),
            RedTy::Var(var) => SymTy::new(db, SymTyKind::Var(var)),
            RedTy::Perm => panic!("infer bound cannot be a perm"),
        };

        let perm_infer = self.env.perm_infer(infer);
        let sym_perm = self.bounding_perm(perm_infer, direction)?;
        Ok(Some(SymTy::new(db, SymTyKind::Perm(sym_perm, sym_ty))))
    }

    /// Return the bounding perm on the permission inference variable `v` from the given `direction`.
    fn bounding_perm(
        &mut self,
        infer: InferVarIndex,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
        let runtime = self.env.runtime().clone();
        runtime.with_inference_var_data(infer, |data| {
            let chains = match direction {
                Direction::FromBelow => data.lower_chains(),
                Direction::FromAbove => data.upper_chains(),
            };

            self.merge_lien_chains(chains.iter().map(|pair| &pair.0), direction)
        })
    }

    /// Merge a list of lien chains, computing their LUB or GLB depending on `direction`.
    fn merge_lien_chains<'a>(
        &mut self,
        mut lien_chains: impl Iterator<Item = &'a Chain<'db>>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>>
    where
        'db: 'a,
    {
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

    /// Merge `left` and `right` producing the lub or glb according to `direction`
    ///
    /// `left` and `right` cannot contain inference variables.
    fn merge_resolved_perms(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
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
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
        assert!(self.is_my_perm(my_perm));
        if self.is_my_perm(other_perm) {
            Ok(my_perm)
        } else if self.meets_shared_bound(other_perm) {
            // my <: our <: shared
            match direction {
                Direction::FromBelow => {
                    // We need a subtype, so take the "my" permission.
                    Ok(my_perm)
                }

                Direction::FromAbove => {
                    // We need a supertype, so take the our/shared permission.
                    Ok(other_perm)
                }
            }
        } else {
            Err(ResolverError::irreconciliable(my_perm, other_perm))
        }
    }

    /// Merge a "our" (fully owned) permission chain with `lien_chain` (which cannot be `my`).
    fn merge_our_perm_and_perm(
        &self,
        our_perm: SymPerm<'db>,
        other_perm: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
        assert!(self.is_our_perm(our_perm));
        assert!(!self.is_my_perm(other_perm));

        if self.is_our_perm(other_perm) {
            Ok(our_perm)
        } else if self.meets_shared_bound(other_perm) {
            // our <: shared
            match direction {
                Direction::FromBelow => {
                    // We need a subtype, so take the "our" permission.
                    Ok(our_perm)
                }

                Direction::FromAbove => {
                    // We need a supertype, so take the our/shared permission.
                    Ok(other_perm)
                }
            }
        } else {
            Err(ResolverError::irreconciliable(our_perm, other_perm))
        }
    }

    /// Merge two permissions, neither of which is `my` or `our` (those are handled specially).
    fn merge_other_perms(
        &self,
        left: SymPerm<'db>,
        right: SymPerm<'db>,
        direction: Direction,
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
        match direction {
            Direction::FromBelow => self.merge_other_perms_glb(left, right),
            Direction::FromAbove => self.merge_other_perms_lub(left, right),
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
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
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
                    return Err(ResolverError::irreconciliable(left, right));
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
                        return Err(ResolverError::irreconciliable(left, right));
                    }
                }
            }
        }

        assert!(!merged_leaves.is_empty());

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
    ) -> Result<Vec<SymPlace<'db>>, ResolverError<'db>> {
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
    ) -> Result<SymPerm<'db>, ResolverError<'db>> {
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
                    return Err(ResolverError::irreconciliable(left, right));
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
                        return Err(ResolverError::irreconciliable(left, right));
                    }
                }
            }
        }

        assert!(!merged_leaves.is_empty());

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
    fn lien_chain_to_perm(&mut self, lien_chain: &Chain<'db>) -> SymPerm<'db> {
        self.liens_to_perm(lien_chain)
    }

    /// Convert a list of `Lien`s into a `SymPerm`.
    fn liens_to_perm(&mut self, liens: &[Lien<'db>]) -> SymPerm<'db> {
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
            Lien::Infer(infer) => self.resolve(SymPerm::infer(self.db, infer)),
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
            SymPermKind::Var(var) => self.env.var_is_declared_to_be(var, Predicate::Copy),
            SymPermKind::Error(_) => false,
        }
    }
}

enum ResolverError<'db> {
    NoBounds,

    Cycle,

    Irreconciliable {
        left: SymGenericTerm<'db>,
        right: SymGenericTerm<'db>,
    },
}

impl<'db> ResolverError<'db> {
    pub fn irreconciliable(
        left: impl Into<SymGenericTerm<'db>>,
        right: impl Into<SymGenericTerm<'db>>,
    ) -> Self {
        ResolverError::Irreconciliable {
            left: left.into(),
            right: right.into(),
        }
    }
}
