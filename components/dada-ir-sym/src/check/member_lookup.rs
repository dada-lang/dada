use crate::ir::{
    binder::Binder,
    classes::{SymAggregate, SymClassMember, SymField},
    functions::SymFunction,
    types::{SymGenericTerm, SymPerm, SymTy, SymTyName},
};
use dada_ir_ast::{
    ast::{Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Err, Errors, Level, Reported},
    span::Span,
};
use dada_util::{debug, debug_heading};

use crate::{
    check::env::Env,
    check::exprs::{ExprResult, ExprResultKind},
    ir::exprs::{SymPlaceExpr, SymPlaceExprKind},
    prelude::CheckedFieldTy,
};

use super::{red::RedTy, to_red::ToRedTy};

pub(crate) struct MemberLookup<'member, 'db> {
    env: &'member mut Env<'db>,
}

impl<'member, 'db> MemberLookup<'member, 'db> {
    pub fn new(env: &'member mut Env<'db>) -> Self {
        Self { env }
    }

    pub async fn lookup_member(
        &mut self,
        owner: ExprResult<'db>,
        id: SpannedIdentifier<'db>,
    ) -> ExprResult<'db> {
        let owner_ty = owner.ty(self.env);

        // Block until we find a lower bound on the owner's type.
        let (lower_bound, owner_perm) = non_infer_lower_bound(self.env, owner_ty).await;

        // The owner will be some supertype of `ty`.
        match self.search_lower_bound_for_member(lower_bound, id.id) {
            Ok(Some(member)) => self.confirm_member(owner, owner_perm, member, id),
            Ok(None) => {
                // If there is no member, then since the owner must be a supertype of `ty`,
                // this expression is invalid.
                self.no_such_member_result(id, owner.span, owner_ty)
            }
            Err(reported) => ExprResult::err(self.env.db(), reported),
        }
    }

    fn confirm_member(
        &mut self,
        owner: ExprResult<'db>,
        owner_perm: Option<SymPerm<'db>>,
        member: SearchResult<'db>,
        id: SpannedIdentifier<'db>,
    ) -> ExprResult<'db> {
        let db = self.env.db();

        // Construct the result
        match member {
            SearchResult::Field {
                owner: _,
                field,
                field_ty,
            } => {
                let mut temporaries = vec![];

                // The type of the field will be the declared type `F` with...
                // * `self` replaced with the place `owner`
                // * the permission from the owner applied
                let owner_place_expr = owner.into_place_expr(self.env, &mut temporaries);
                let field_ty = field_ty.substitute(db, &[owner_place_expr.into_sym_place(db)]);
                let field_ty_with_perm = match owner_perm {
                    None => field_ty,
                    Some(p) => SymTy::perm(db, p, field_ty),
                };

                // construct the place expression
                let place_expr = SymPlaceExpr::new(
                    db,
                    id.span,
                    field_ty_with_perm,
                    SymPlaceExprKind::Field(owner_place_expr, field),
                );
                ExprResult::from_place_expr(db, place_expr, temporaries)
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
        }
    }

    fn no_such_member_result(
        &mut self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: SymTy<'db>,
    ) -> ExprResult<'db> {
        ExprResult::err(self.env.db(), self.no_such_member(id, owner_span, owner_ty))
    }

    fn no_such_member(
        &mut self,
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
        &mut self,
        lower_bound: RedTy<'db>,
        id: Identifier<'db>,
    ) -> Errors<Option<SearchResult<'db>>> {
        debug_heading!("search_lower_bound_for_member", lower_bound, id);
        match lower_bound {
            RedTy::Named(name, ref generics) => match name {
                // Primitive types don't have members.
                SymTyName::Primitive(_) => Ok(None),

                // Tuples have indexed members, not named ones.
                SymTyName::Tuple { arity: _ } => Ok(None),

                // Classes have members.
                SymTyName::Aggregate(owner) => self.search_aggr_for_member(owner, generics, id),

                // Future types have no members.
                SymTyName::Future => Ok(None),
            },
            RedTy::Error(reported) => Err(reported),
            RedTy::Never => Ok(None),
            RedTy::Infer(_) => panic!("did not expect inference variable"),
            RedTy::Var(_) => Ok(None),
            RedTy::Perm => panic!("did not expect permission red-ty"),
        }
    }

    fn search_aggr_for_member(
        &mut self,
        owner: SymAggregate<'db>,
        generics: &[SymGenericTerm<'db>],
        id: Identifier<'db>,
    ) -> Errors<Option<SearchResult<'db>>> {
        let db = self.env.db();
        debug_heading!("search_class_for_member", id, owner);

        for &member in owner.members(db) {
            match member {
                SymClassMember::SymField(field) => {
                    if field.name(db) == id {
                        debug!("found field", field);
                        return Ok(Some(SearchResult::Field {
                            owner,
                            field,
                            field_ty: field.checked_field_ty(db).substitute(db, &generics),
                        }));
                    } else {
                        debug!("found field with wrong name", field.name(db));
                    }
                }

                SymClassMember::SymFunction(method) => {
                    if method.name(db) == id {
                        debug!("found method", method);
                        return Ok(Some(SearchResult::Method { owner, method }));
                    } else {
                        debug!("found method with wrong name", method.name(db));
                    }
                }
            }
        }

        Ok(None)
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
}

/// Convert `ty` to a [`RedTy`][]; if the result is an inference variable,
/// then wait until that variable has a lower-bound.
///
/// # Returns
///
/// A [`RedTy`][] that is a lower bound for `ty` and which is not an inference variable.
async fn non_infer_lower_bound<'db>(
    env: &mut Env<'db>,
    ty: SymTy<'db>,
) -> (RedTy<'db>, Option<SymPerm<'db>>) {
    let (red_ty, perm) = ty.to_red_ty(env);
    if let RedTy::Infer(infer_var_index) = red_ty {
        match env
            .runtime()
            .loop_on_inference_var(infer_var_index, |data| data.lower_red_ty())
            .await
        {
            Some((bound_red_ty, _)) => (bound_red_ty, perm),
            None => {
                // Subtle: Not possible to get here. The reason is that the above for-loop will
                // never terminate until we can construct a complete expression for the body,
                // and we can't do that until we resolve all member references.
                unreachable!()
            }
        }
    } else {
        (red_ty, perm)
    }
}
