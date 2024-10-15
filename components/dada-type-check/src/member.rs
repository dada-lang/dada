use std::pin::{self, pin};

use dada_ir_ast::{
    ast::{AstPerm, Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};
use dada_ir_sym::{
    class::{SymClass, SymClassMember, SymField},
    function::SymFunction,
    subst::Subst,
    ty::{SymGenericTerm, SymPerm, SymTy, SymTyKind, SymTyName},
};
use futures::{Stream, StreamExt};

use crate::{
    bound::Bound, checking_ir::PlaceExprKind, env::Env, executor::Check, exprs::ExprResult,
};

#[derive(Copy, Clone)]
pub(crate) struct MemberLookup<'member, 'chk, 'db> {
    check: &'member Check<'chk, 'db>,
    env: &'member Env<'db>,
}

impl<'member, 'chk, 'db> MemberLookup<'member, 'chk, 'db> {
    pub fn new(check: &'member Check<'chk, 'db>, env: &'member Env<'db>) -> Self {
        Self { check, env }
    }

    pub async fn lookup_member(
        self,
        owner: ExprResult<'chk, 'db>,
        id: SpannedIdentifier<'db>,
    ) -> ExprResult<'chk, 'db> {
        let db = self.check.db;
        let owner_ty = owner.ty(self.check, self.env);

        // Iterate over the bounds, looking for a valid method resolution.
        //
        // * If we find an upper bound:
        // * If we find a lower bound:
        //
        // Once we
        let mut bounds = self.env.bounds(self.check, owner_ty);
        while let Some(bound) = bounds.next().await {
            match bound {
                Bound::LowerBound(ty) => {
                    // The owner will be some supertype of `ty`.
                    if let Some(member) = self.search_type_for_member(ty, id.id) {
                        return self.confirm_member(owner, member, id, bounds);
                    } else {
                        // If there is no member, then since the owner must be a supertype of `ty`,
                        // this expression is invalid.
                        return self.no_such_member(id, owner.span, ty);
                    }
                }
                Bound::UpperBound(ty) => {
                    // The owner will be some subtype of `ty`.
                    if let Some(member) = self.search_type_for_member(ty, id.id) {
                        return self.confirm_member(owner, member, id, bounds);
                    } else {
                        // For an upper bound, it's ok not to find a match.
                        // We may a more precise bound later that does have a match.
                    }
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
        owner: ExprResult<'chk, 'db>,
        member: SearchResult<'db>,
        id: SpannedIdentifier<'db>,
        bounds: impl Stream<Item = Bound<SymTy<'db>>> + 'chk,
    ) -> ExprResult<'chk, 'db> {
        let db = self.check.db;

        // Iterate through any remaining bounds to make sure that this member is valid
        // for all of them and that no ambiguity arises.
        self.check.defer(self.env, async move |check, env| {
            let bounds = pin!(bounds);
            while let Some(bound) = bounds.next().await {}
        });

        // Construct the result
        match member {
            SearchResult::Field {
                owner: owner_class,
                field,
                field_ty,
            } => {
                let mut temporaries = vec![];
                let owner_place_expr =
                    owner.into_place_expr(self.check, self.env, &mut temporaries);
                let self_lv = owner_class.field_self_lv();
                let field_ty =
                    field_ty.subst_lv(self_lv, self_lv, owner_place_expr.to_sym_place(db));
                let place_expr = self.check.place_expr(
                    id.span,
                    field_ty,
                    PlaceExprKind::Field(owner_place_expr, field),
                );
                ExprResult::from_place_expr(self.check, self.env, place_expr, temporaries)
            }
            SearchResult::Method { owner: _, method } => {}
            SearchResult::Error(reported) => ExprResult::err(self.check, id.span, reported),
        }
    }

    fn no_such_member(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: SymTy<'db>,
    ) -> ExprResult<'chk, 'db> {
        let db = self.check.db;
        let SpannedIdentifier { span: id_span, id } = id;
        ExprResult::err(
            self.check,
            id_span,
            Diagnostic::error(db, id_span, format!("unrecognized field `{}`", id))
                .label(
                    db,
                    Level::Error,
                    id_span,
                    format!("I could not find a field declaration for `{id}`"),
                )
                .label(
                    db,
                    Level::Info,
                    owner_span,
                    format!(
                        "this has type `{ty}`, which doesn't appear to have a field `{id}`",
                        ty = self.env.describe_ty(self.check, owner_ty)
                    ),
                )
                .report(db),
        )
    }

    fn search_type_for_member(
        self,
        ty: SymTy<'db>,
        id: Identifier<'db>,
    ) -> Option<SearchResult<'db>> {
        let db = self.check.db;
        match ty.kind(db) {
            SymTyKind::Named(name, generics) => match name {
                // Primitive types don't have members.
                SymTyName::Primitive(_) => None,

                // Tuples have indexed members, not named ones.
                SymTyName::Tuple { arity } => None,

                // Classes have members.
                SymTyName::Class(owner) => self.search_class_for_member(owner, generics, id),
            },

            SymTyKind::Perm(perm, ty) => {
                Some(self.search_type_for_member(ty, id)?.with_perm(db, perm))
            }

            SymTyKind::Var(generic_index) => {
                // FIXME: where-clauses
                None
            }
            SymTyKind::Unknown => {
                // How to manage "any" types? Not sure what I even *want* here.
                // Parsing is ambiguous, for example.
                // Given `x: Any`, is `x.foo[...]` a method call or an indexed field access?
                // For now just force users to downcast.
                None
            }
            SymTyKind::Error(reported) => Some(SearchResult::Error(reported)),
        }
    }

    fn search_class_for_member(
        self,
        owner: SymClass<'db>,
        generics: &[SymGenericTerm<'db>],
        id: Identifier<'db>,
    ) -> Option<SearchResult<'db>> {
        let db = self.check.db;

        for &member in owner.members(db) {
            match member {
                SymClassMember::SymField(field) => {
                    if field.name(db) == id {
                        return Some(SearchResult::Field {
                            owner,
                            field,
                            field_ty: field.ty(db).substitute(db, &generics),
                        });
                    }
                }

                SymClassMember::SymFunction(method) => {
                    if method.name(db) == id {
                        return Some(SearchResult::Method { owner, method });
                    }
                }
            }
        }

        None
    }
}

#[derive(Copy, Clone)]
enum SearchResult<'db> {
    Field {
        owner: SymClass<'db>,
        field: SymField<'db>,
        field_ty: SymTy<'db>,
    },
    Method {
        owner: SymClass<'db>,
        method: SymFunction<'db>,
    },
    Error(Reported),
}

impl<'db> SearchResult<'db> {
    fn with_perm(self, db: &'db dyn crate::Db, perm: SymPerm<'db>) -> Self {
        match self {
            SearchResult::Field {
                owner,
                field,
                field_ty,
            } => SearchResult::Field {
                owner,
                field,
                field_ty: SymTy::new(db, SymTyKind::Perm(perm, field_ty)),
            },
            SearchResult::Method { owner, method } => SearchResult::Method { owner, method },
            SearchResult::Error(reported) => SearchResult::Error(reported),
        }
    }
}
