use crate::{
    ir::binder::Binder,
    ir::classes::{SymAggregate, SymClassMember, SymField},
    ir::functions::SymFunction,
    ir::types::{SymGenericTerm, SymTy, SymTyKind, SymTyName},
};
use dada_ir_ast::{
    ast::{Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Err, Errors, Level, Reported},
    span::Span,
};
use dada_util::{debug, debug_heading};
use futures::StreamExt;

use crate::{
    check::env::Env,
    check::exprs::{ExprResult, ExprResultKind},
    ir::exprs::{SymPlaceExpr, SymPlaceExprKind},
    prelude::CheckedFieldTy,
};

use super::bound::TransitiveBounds;

#[derive(Copy, Clone)]
pub(crate) struct MemberLookup<'member, 'db> {
    env: &'member Env<'db>,
}

impl<'member, 'db> MemberLookup<'member, 'db> {
    pub fn new(env: &'member Env<'db>) -> Self {
        Self { env }
    }

    pub async fn lookup_member(
        self,
        owner: ExprResult<'db>,
        id: SpannedIdentifier<'db>,
    ) -> ExprResult<'db> {
        let owner_ty = owner.ty(self.env);

        // Iterate over the bounds, looking for a valid method resolution.
        //
        // * If we find an upper bound:
        // * If we find a lower bound:
        //
        // Once we
        let mut lower_bounds = self.env.transitive_ty_lower_bounds(owner_ty);

        while let Some(ty) = lower_bounds.next().await {
            // The owner will be some supertype of `ty`.
            match self.search_lower_bound_for_member(ty, id.id, &mut lower_bounds) {
                Ok(Some(member)) => {
                    return self.confirm_member(owner, ty, member, id, lower_bounds);
                }
                Ok(None) => {
                    // inference variable, just keep searching
                    continue;
                }
                Err(()) => {
                    // If there is no member, then since the owner must be a supertype of `ty`,
                    // this expression is invalid.
                    return self.no_such_member_result(id, owner.span, ty);
                }
            }
        }

        // Subtle: Not possible to get here. The reason is that the above for-loop will
        // never terminate until we can construct a complete expression for the body,
        // and we can't do that until we resolve all member references.

        unreachable!()
    }

    fn confirm_member(
        self,
        owner: ExprResult<'db>,
        owner_ty: SymTy<'db>,
        member: SearchResult<'db>,
        id: SpannedIdentifier<'db>,
        mut lower_bounds: TransitiveBounds<'db, SymTy<'db>>,
    ) -> ExprResult<'db> {
        let db = self.env.db();

        // Iterate through any remaining bounds to make sure that this member is valid
        // for all of them and that no ambiguity arises.
        if !matches!(member, SearchResult::Error(Reported(_))) {
            self.env.defer(id.span, {
                let owner = owner.clone();
                let member = member.clone();
                async move |env| {
                    let this = MemberLookup { env: &env };
                    while let Some(ty) = lower_bounds.next().await {
                        if let Err(Reported(_)) =
                            this.check_member(&owner, id, owner_ty, &member, ty, &mut lower_bounds)
                        {
                            return;
                        }
                    }
                }
            });
        }

        // Construct the result
        match member {
            SearchResult::Field {
                owner: _,
                field,
                field_ty,
            } => {
                let mut temporaries = vec![];
                let owner_place_expr = owner.into_place_expr(self.env, &mut temporaries);
                let field_ty = field_ty.substitute(db, &[owner_place_expr.into_sym_place(db)]);
                let place_expr = SymPlaceExpr::new(
                    db,
                    id.span,
                    field_ty,
                    SymPlaceExprKind::Field(owner_place_expr, field),
                );
                ExprResult::from_place_expr(self.env, place_expr, temporaries)
            }
            SearchResult::Method { owner: _, method } => {
                let mut temporaries = vec![];
                let owner = owner.into_expr(self.env, &mut temporaries);
                ExprResult {
                    temporaries,
                    span: owner.span(db).to(db, id.span),
                    kind: ExprResultKind::Method {
                        self_expr: owner,
                        function: method,
                        generics: None,
                        id_span: id.span,
                    },
                }
            }
            SearchResult::Error(reported) => ExprResult::err(db, reported),
        }
    }

    /// After we have chosen how to resolve a given member,
    /// we may still get more inference variable constraints,
    /// so we have to check that this would still be the right
    /// choice for that constraint
    /// or else there is ambiguity.
    fn check_member(
        self,
        owner: &ExprResult<'db>,
        id: SpannedIdentifier<'db>,
        prev_ty: SymTy<'db>,
        prev_member: &SearchResult<'db>,
        new_ty: SymTy<'db>,
        lower_bounds: &mut TransitiveBounds<'db, SymTy<'db>>,
    ) -> Errors<()> {
        match self.search_lower_bound_for_member(new_ty, id.id, lower_bounds) {
            Ok(Some(new_member)) => {
                if *prev_member == new_member {
                    Ok(())
                } else {
                    Err(self.ambiguous_member(
                        id,
                        owner.span,
                        prev_ty,
                        new_ty,
                        prev_member,
                        &new_member,
                    ))
                }
            }
            Ok(None) => {
                // inference variable, just keep searching
                Ok(())
            }
            Err(()) => {
                // FIXME: not the ideal error message
                Err(self.no_such_member(id, owner.span, new_ty))
            }
        }
    }

    fn ambiguous_member(
        self,
        id: SpannedIdentifier<'db>,
        #[expect(unused_variables)] owner_span: Span<'db>,
        #[expect(unused_variables)] prev_ty: SymTy<'db>,
        #[expect(unused_variables)] new_ty: SymTy<'db>,
        prev_member: &SearchResult<'db>,
        new_member: &SearchResult<'db>,
    ) -> Reported {
        let db = self.env.db();
        let SpannedIdentifier { span: id_span, id } = id;

        let mut diag = Diagnostic::error(db, id_span, format!("ambiguous member `{}`", id));

        diag = diag.label(
            db,
            Level::Error,
            id_span,
            format!("I found more than one way to resolve `{id}`"),
        );

        diag = match prev_member {
            SearchResult::Field {
                owner,
                field,
                field_ty: _,
            } => diag.label(
                db,
                Level::Info,
                field.name_span(db),
                format!(
                    "one option is the field `{f}` defined in `{owner}`",
                    f = field.name(db)
                ),
            ),
            SearchResult::Method { owner, method } => diag.label(
                db,
                Level::Info,
                method.name_span(db),
                format!(
                    "one option is the method `{f}` defined in `{owner}`",
                    f = method.name(db)
                ),
            ),
            SearchResult::Error(_) => unreachable!(),
        };

        diag = match new_member {
            SearchResult::Field {
                owner,
                field,
                field_ty: _,
            } => diag.label(
                db,
                Level::Info,
                field.name_span(db),
                format!(
                    "another option is the field `{f}` defined in `{owner}`",
                    f = field.name(db)
                ),
            ),
            SearchResult::Method { owner, method } => diag.label(
                db,
                Level::Info,
                method.name_span(db),
                format!(
                    "another option is the method `{m}` defined in `{owner}`",
                    m = method.name(db)
                ),
            ),
            SearchResult::Error(_) => unreachable!(),
        };

        diag.report(db)
    }

    fn no_such_member_result(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: SymTy<'db>,
    ) -> ExprResult<'db> {
        ExprResult::err(self.env.db(), self.no_such_member(id, owner_span, owner_ty))
    }

    fn no_such_member(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: SymTy<'db>,
    ) -> Reported {
        let db = self.env.db();
        let SpannedIdentifier { span: id_span, id } = id;
        Diagnostic::error(
            db,
            id_span,
            format!("unrecognized field or method `{}`", id),
        )
        .label(
            db,
            Level::Error,
            id_span,
            format!("I could not find a field or method named `{id}`"),
        )
        .label(
            db,
            Level::Info,
            owner_span,
            format!(
                "this has type `{ty}`, which doesn't appear to have a field or method `{id}`",
                ty = self.env.describe_ty(owner_ty)
            ),
        )
        .report(db)
    }

    fn search_lower_bound_for_member(
        self,
        ty: SymTy<'db>,
        id: Identifier<'db>,
        bounds: &mut TransitiveBounds<'db, SymTy<'db>>,
    ) -> Result<Option<SearchResult<'db>>, ()> {
        debug_heading!("search_lower_bound_for_member", id, ty);
        let db = self.env.db();
        match *ty.kind(db) {
            SymTyKind::Named(name, ref generics) => match name {
                // Primitive types don't have members.
                SymTyName::Primitive(_) => Err(()),

                // Tuples have indexed members, not named ones.
                SymTyName::Tuple { arity: _ } => Err(()),

                // Classes have members.
                SymTyName::Aggregate(owner) => {
                    Ok(Some(self.search_class_for_member(owner, generics, id)?))
                }

                // Future types have no members.
                SymTyName::Future => Err(()),
            },

            SymTyKind::Perm(_, sym_ty) => self.search_lower_bound_for_member(sym_ty, id, bounds),

            SymTyKind::Infer(var) => {
                // The transitive bounds search will not directly yield inference variables,
                // but we can encounter them if it yields e.g. a type like `leased ?X`.
                // In that case we just push them onto the transitive bounds iterator and explore
                // their bounds too.
                debug!("pushing inference var", var);
                bounds.push_inference_var(var);
                Ok(None)
            }

            SymTyKind::Var(_) => {
                // FIXME: where-clauses
                Err(())
            }

            SymTyKind::Never => Err(()),

            SymTyKind::Error(reported) => Ok(Some(SearchResult::Error(reported))),
        }
    }

    fn search_class_for_member(
        self,
        owner: SymAggregate<'db>,
        generics: &[SymGenericTerm<'db>],
        id: Identifier<'db>,
    ) -> Result<SearchResult<'db>, ()> {
        let db = self.env.db();
        debug_heading!("search_class_for_member", id, owner);

        for &member in owner.members(db) {
            match member {
                SymClassMember::SymField(field) => {
                    if field.name(db) == id {
                        debug!("found field", field);
                        return Ok(SearchResult::Field {
                            owner,
                            field,
                            field_ty: field.checked_field_ty(db).substitute(db, &generics),
                        });
                    } else {
                        debug!("found field with wrong name", field.name(db));
                    }
                }

                SymClassMember::SymFunction(method) => {
                    if method.name(db) == id {
                        debug!("found method", method);
                        return Ok(SearchResult::Method { owner, method });
                    } else {
                        debug!("found method with wrong name", method.name(db));
                    }
                }
            }
        }

        Err(())
    }
}

#[derive(Clone, PartialEq, Eq)]
enum SearchResult<'db> {
    Field {
        owner: SymAggregate<'db>,
        field: SymField<'db>,
        field_ty: Binder<'db, SymTy<'db>>,
    },
    Method {
        owner: SymAggregate<'db>,
        method: SymFunction<'db>,
    },
    Error(Reported),
}
