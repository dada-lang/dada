use std::pin::pin;

use dada_ir_ast::{
    ast::{Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Errors, Level, Reported},
    span::Span,
};
use dada_ir_sym::{
    binder::Binder,
    class::{SymClass, SymClassMember, SymField},
    function::SymFunction,
    ty::SymTyName,
};
use futures::{Stream, StreamExt};

use crate::{
    bound::Bound,
    check::Check,
    env::Env,
    exprs::{ExprResult, ExprResultKind},
    object_ir::{
        IntoObjectIr, ObjectGenericTerm, ObjectPlaceExpr, ObjectPlaceExprKind, ObjectTy,
        ObjectTyKind,
    },
};

#[derive(Copy, Clone)]
pub(crate) struct MemberLookup<'member, 'db> {
    check: &'member Check<'db>,
    env: &'member Env<'db>,
}

impl<'member, 'db> MemberLookup<'member, 'db> {
    pub fn new(check: &'member Check<'db>, env: &'member Env<'db>) -> Self {
        Self { check, env }
    }

    pub async fn lookup_member(
        self,
        owner: ExprResult<'db>,
        id: SpannedIdentifier<'db>,
    ) -> ExprResult<'db> {
        let db = self.check.db;
        let owner_ty = owner.ty(self.check, self.env);

        // Iterate over the bounds, looking for a valid method resolution.
        //
        // * If we find an upper bound:
        // * If we find a lower bound:
        //
        // Once we
        let mut lower_bounds = self
            .env
            .object_bounds(self.check, owner_ty)
            .filter_map(|b| {
                futures::future::ready(match b {
                    Bound::LowerBound(ty) => Some(ty),
                    Bound::UpperBound(_) => None,
                })
            });

        while let Some(ty) = lower_bounds.next().await {
            // The owner will be some supertype of `ty`.
            if let Some(member) = self.search_type_for_member(ty, id.id) {
                return self.confirm_member(owner, ty, member, id, lower_bounds);
            } else {
                // If there is no member, then since the owner must be a supertype of `ty`,
                // this expression is invalid.
                return self.no_such_member_result(id, owner.span, ty);
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
        owner_ty: ObjectTy<'db>,
        member: SearchResult<'db>,
        id: SpannedIdentifier<'db>,
        lower_bounds: impl Stream<Item = ObjectTy<'db>> + 'db,
    ) -> ExprResult<'db> {
        let db = self.check.db;

        // Iterate through any remaining bounds to make sure that this member is valid
        // for all of them and that no ambiguity arises.
        if !matches!(SearchResult::Error(Reported), member) {
            self.check.defer(self.env, {
                let owner = owner.clone();
                let member = member.clone();
                async move |check, env| {
                    let this = MemberLookup {
                        check: &check,
                        env: &env,
                    };
                    let mut lower_bounds = pin!(lower_bounds);
                    while let Some(ty) = lower_bounds.next().await {
                        if let Err(Reported) = this.check_member(&owner, id, owner_ty, &member, ty)
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
                owner: owner_class,
                field,
                field_ty,
            } => {
                let mut temporaries = vec![];
                let owner_place_expr =
                    owner.into_place_expr(self.check, self.env, &mut temporaries);
                let field_ty = field_ty.substitute(db, &[owner_place_expr.to_object_place()]);
                let place_expr = ObjectPlaceExpr::new(
                    db,
                    id.span,
                    field_ty,
                    ObjectPlaceExprKind::Field(owner_place_expr, field),
                );
                ExprResult::from_place_expr(self.check, self.env, place_expr, temporaries)
            }
            SearchResult::Method { owner: _, method } => {
                let mut temporaries = vec![];
                let owner = owner.into_expr(self.check, self.env, &mut temporaries);
                ExprResult {
                    temporaries,
                    span: owner.span(db).to(id.span),
                    kind: ExprResultKind::Method {
                        self_expr: owner,
                        function: method,
                        generics: None,
                        id_span: id.span,
                    },
                }
            }
            SearchResult::Error(reported) => ExprResult::err(self.check, id.span, reported),
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
        prev_ty: ObjectTy<'db>,
        prev_member: &SearchResult<'db>,
        new_ty: ObjectTy<'db>,
    ) -> Errors<()> {
        match self.search_type_for_member(new_ty, id.id) {
            Some(new_member) => {
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
            None => {
                // FIXME: not the ideal error message
                Err(self.no_such_member(id, owner.span, new_ty))
            }
        }
    }

    fn ambiguous_member(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        prev_ty: ObjectTy<'db>,
        new_ty: ObjectTy<'db>,
        prev_member: &SearchResult<'db>,
        new_member: &SearchResult<'db>,
    ) -> Reported {
        let db = self.check.db;
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
                field_ty,
            } => diag.label(
                db,
                Level::Info,
                field.name_span(db),
                format!("one option is the field `{f}`", f = field.name(db)),
            ),
            SearchResult::Method { owner, method } => diag.label(
                db,
                Level::Info,
                method.name_span(db),
                format!("one option is the method `{f}`", f = method.name(db)),
            ),
            SearchResult::Error(_) => unreachable!(),
        };

        diag = match new_member {
            SearchResult::Field {
                owner,
                field,
                field_ty,
            } => diag.label(
                db,
                Level::Info,
                field.name_span(db),
                format!("another option is the field `{f}`", f = field.name(db)),
            ),
            SearchResult::Method { owner, method } => diag.label(
                db,
                Level::Info,
                method.name_span(db),
                format!("another option is the method `{f}`", f = method.name(db)),
            ),
            SearchResult::Error(_) => unreachable!(),
        };

        diag.report(db)
    }

    fn no_such_member_result(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: ObjectTy<'db>,
    ) -> ExprResult<'db> {
        ExprResult::err(
            self.check,
            id.span,
            self.no_such_member(id, owner_span, owner_ty),
        )
    }

    fn no_such_member(
        self,
        id: SpannedIdentifier<'db>,
        owner_span: Span<'db>,
        owner_ty: ObjectTy<'db>,
    ) -> Reported {
        let db = self.check.db;
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
                ty = self.env.describe_ty(self.check, owner_ty)
            ),
        )
        .report(db)
    }

    fn search_type_for_member(
        self,
        ty: ObjectTy<'db>,
        id: Identifier<'db>,
    ) -> Option<SearchResult<'db>> {
        let db = self.check.db;
        match ty.kind(db) {
            ObjectTyKind::Named(name, generics) => match *name {
                // Primitive types don't have members.
                SymTyName::Primitive(_) => None,

                // Tuples have indexed members, not named ones.
                SymTyName::Tuple { arity } => None,

                // Classes have members.
                SymTyName::Class(owner) => self.search_class_for_member(owner, generics, id),
            },

            ObjectTyKind::Var(generic_index) => {
                // FIXME: where-clauses
                None
            }

            ObjectTyKind::Unknown => {
                // How to manage "any" types? Not sure what I even *want* here.
                // Parsing is ambiguous, for example.
                // Given `x: Any`, is `x.foo[...]` a method call or an indexed field access?
                // For now just force users to downcast.
                None
            }

            ObjectTyKind::Error(reported) => Some(SearchResult::Error(*reported)),
        }
    }

    fn search_class_for_member(
        self,
        owner: SymClass<'db>,
        generics: &[ObjectGenericTerm<'db>],
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
                            field_ty: field.ty(db).into_object_ir(db).substitute(db, &generics),
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

#[derive(Clone, PartialEq, Eq)]
enum SearchResult<'db> {
    Field {
        owner: SymClass<'db>,
        field: SymField<'db>,
        field_ty: Binder<ObjectTy<'db>>,
    },
    Method {
        owner: SymClass<'db>,
        method: SymFunction<'db>,
    },
    Error(Reported),
}
