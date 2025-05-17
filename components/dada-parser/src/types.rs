use salsa::Update;

use dada_ir_ast::{
    ast::{
        AstGenericDecl, AstGenericKind, AstGenericTerm, AstPath, AstPerm, AstPermKind, AstTy,
        AstTyKind, SpanVec,
    },
    span::{Span, Spanned},
};

use super::{
    Expected, Parse, ParseFail, Parser,
    tokenizer::{Delimiter, Keyword},
};

// Parsing types and permissions is annoying.
// Declare a cover grammar first.
#[derive(Debug, Update)]
enum TyOrPerm<'db> {
    /// could be anything from `a` to `a.b` to `a[x]` to `a.b[x]`
    Path(AstPath<'db>, Option<SpanVec<'db, AstGenericTerm<'db>>>),

    /// `type T` or `perm P`
    Generic(AstGenericDecl<'db>),

    /// Perm that starts with a keyword, like `my`
    PermKeyword(AstPerm<'db>),

    /// P1 P2
    Apply(AstPerm<'db>, AstTy<'db>),
}

impl<'db> Parse<'db> for TyOrPerm<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        if let Some(path) = AstPath::opt_parse(db, parser)? {
            let generic_args = AstGenericTerm::opt_parse_delimited(
                db,
                parser,
                Delimiter::SquareBrackets,
                AstGenericTerm::eat_comma,
            )?;

            return TyOrPerm::Path(path, generic_args).maybe_apply(db, parser);
        }

        if let Some(generic_decl) = AstGenericDecl::opt_parse(db, parser)? {
            return TyOrPerm::Generic(generic_decl).maybe_apply(db, parser);
        };

        if let Some(p) = KeywordPerm::opt_parse(db, parser)? {
            return TyOrPerm::PermKeyword(p).maybe_apply(db, parser);
        }

        Ok(None)
    }

    fn expected() -> Expected {
        Expected::Nonterminal("type or permission")
    }
}

impl<'db> Spanned<'db> for TyOrPerm<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            TyOrPerm::Path(path, args) => {
                let args_span = args.as_ref().map(|a| a.span);
                path.span(db).to(db, args_span)
            }
            TyOrPerm::Generic(decl) => decl.span(db),
            TyOrPerm::PermKeyword(p) => p.span(db),
            TyOrPerm::Apply(p, ty) => p.span(db).to(db, ty.span(db)),
        }
    }
}

impl<'db> TyOrPerm<'db> {
    /// If this could be a permission and it is followed by a type, parse it as an application.
    fn maybe_apply(
        self,
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        if self.can_be_perm(db) && parser.next_token_on_same_line() {
            if let Some(ty) = AstTy::opt_parse(db, parser)? {
                let perm = self.into_perm(db).unwrap();
                return Ok(Some(TyOrPerm::Apply(perm, ty)));
            }
        }

        Ok(Some(self))
    }

    /// True if this could syntactically be a permission.
    fn can_be_perm(&self, db: &'db dyn crate::Db) -> bool {
        match self {
            TyOrPerm::Path(path, None) => path.len(db) == 1,
            TyOrPerm::Path(_path, Some(_)) => false,
            TyOrPerm::Generic(decl) => matches!(decl.kind(db), AstGenericKind::Perm(_)),
            TyOrPerm::PermKeyword(_) => true,
            TyOrPerm::Apply(_, _) => false,
        }
    }

    fn into_perm(self, db: &'db dyn crate::Db) -> Option<AstPerm<'db>> {
        match self {
            TyOrPerm::Path(path, None) if path.len(db) == 1 => {
                let id = path.first_id(db);
                Some(AstPerm::new(db, id.span, AstPermKind::Variable(id)))
            }
            TyOrPerm::Path(..) => None,
            TyOrPerm::Generic(decl) => match decl.kind(db) {
                AstGenericKind::Perm(_) => Some(AstPerm::new(
                    db,
                    decl.span(db),
                    AstPermKind::GenericDecl(decl),
                )),
                _ => None,
            },
            TyOrPerm::PermKeyword(p) => Some(p),
            TyOrPerm::Apply(_, _) => None,
        }
    }

    /// True if this could syntactically be a permission.
    fn can_be_ty(&self, db: &'db dyn crate::Db) -> bool {
        match self {
            TyOrPerm::Path(..) => true,
            TyOrPerm::Generic(decl) => matches!(decl.kind(db), AstGenericKind::Type(_)),
            TyOrPerm::PermKeyword(_) => false,
            TyOrPerm::Apply(_, _) => true,
        }
    }

    fn into_ty(self, db: &'db dyn crate::Db) -> Option<AstTy<'db>> {
        let span = self.span(db);
        match self {
            TyOrPerm::Path(path, args) => Some(AstTy::new(db, span, AstTyKind::Named(path, args))),
            TyOrPerm::Generic(decl) => match decl.kind(db) {
                AstGenericKind::Type(_) => Some(AstTy::new(db, span, AstTyKind::GenericDecl(decl))),
                _ => None,
            },
            TyOrPerm::PermKeyword(_) => None,
            TyOrPerm::Apply(p, t) => Some(AstTy::new(db, span, AstTyKind::Perm(p, t))),
        }
    }
}

impl<'db> Parse<'db> for AstTy<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Some(ty_or_perm) = TyOrPerm::opt_parse(db, parser)? else {
            return Ok(None);
        };

        let span = ty_or_perm.span(db);

        if let Some(ty) = ty_or_perm.into_ty(db) {
            return Ok(Some(ty));
        }

        Err(ParseFail::Expected(span, Self::expected()))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("type")
    }
}

impl<'db> Parse<'db> for AstPerm<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Some(ty_or_perm) = TyOrPerm::opt_parse(db, parser)? else {
            return Ok(None);
        };

        let span = ty_or_perm.span(db);

        if let Some(perm) = ty_or_perm.into_perm(db) {
            return Ok(Some(perm));
        }

        Err(ParseFail::Expected(span, Self::expected()))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("permission")
    }
}

struct KeywordPerm;

impl<'db> Parse<'db> for KeywordPerm {
    type Output = AstPerm<'db>;

    fn opt_parse(
        db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<AstPerm<'db>>, ParseFail<'db>> {
        if let Ok(span) = tokens.eat_keyword(Keyword::Ref) {
            Ok(Some(parse_path_perm(
                db,
                span,
                tokens,
                AstPermKind::Referenced,
            )?))
        } else if let Ok(span) = tokens.eat_keyword(Keyword::Mut) {
            Ok(Some(parse_path_perm(
                db,
                span,
                tokens,
                AstPermKind::Mutable,
            )?))
        } else if let Ok(span) = tokens.eat_keyword(Keyword::Given) {
            Ok(Some(parse_path_perm(db, span, tokens, AstPermKind::Given)?))
        } else if let Ok(span) = tokens.eat_keyword(Keyword::My) {
            Ok(Some(AstPerm::new(db, span, AstPermKind::My)))
        } else if let Ok(span) = tokens.eat_keyword(Keyword::Our) {
            Ok(Some(AstPerm::new(db, span, AstPermKind::Our)))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected {
        AstPerm::expected()
    }
}

fn parse_path_perm<'db>(
    db: &'db dyn crate::Db,
    span: Span<'db>,
    parser: &mut Parser<'_, 'db>,
    op: impl Fn(Option<SpanVec<'db, AstPath<'db>>>) -> AstPermKind<'db>,
) -> Result<AstPerm<'db>, ParseFail<'db>> {
    let paths =
        AstPath::opt_parse_delimited(db, parser, Delimiter::SquareBrackets, AstPath::eat_comma)?;
    let kind = op(paths);
    Ok(AstPerm::new(db, span.to(db, parser.last_span()), kind))
}

impl<'db> Parse<'db> for AstGenericTerm<'db> {
    type Output = AstGenericTerm<'db>;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        let Some(ty_or_perm) = TyOrPerm::opt_parse(db, parser)? else {
            return Ok(None);
        };

        match ty_or_perm {
            // There is one case that could be either a type or a permission.
            TyOrPerm::Path(path, None) if path.len(db) == 1 => {
                Ok(Some(AstGenericTerm::Id(path.first_id(db))))
            }

            // For the rest, we can be guided by the syntax.
            TyOrPerm::Generic(_)
            | TyOrPerm::PermKeyword(_)
            | TyOrPerm::Path(..)
            | TyOrPerm::Apply(_, _) => {
                let can_be_perm = ty_or_perm.can_be_perm(db);
                let can_be_ty = ty_or_perm.can_be_ty(db);

                if can_be_perm {
                    assert!(!can_be_ty);
                    Ok(Some(ty_or_perm.into_perm(db).unwrap().into()))
                } else {
                    assert!(can_be_ty);
                    Ok(Some(ty_or_perm.into_ty(db).unwrap().into()))
                }
            }
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("type or permission")
    }
}
