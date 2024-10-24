use dada_ir_ast::ast::{AstGenericTerm, AstPath, AstPathKind};

use super::{Expected, Parse, ParseFail, Parser};

impl<'db> Parse<'db> for AstPath<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Ok(id) = parser.eat_id() else {
            return Ok(None);
        };
        let mut path = AstPath::new(db, AstPathKind::Identifier(id));

        loop {
            if let Ok(_) = parser.eat_op(".") {
                let id = parser.eat_id()?;
                path = AstPath::new(db, AstPathKind::Member { path, id });
                continue;
            }

            if let Some(args) = AstGenericTerm::opt_parse_delimited(
                db,
                parser,
                crate::tokenizer::Delimiter::SquareBrackets,
                AstGenericTerm::eat_comma,
            )? {
                path = AstPath::new(db, AstPathKind::GenericArgs { path, args });
                continue;
            }

            return Ok(Some(path));
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("path")
    }
}

pub trait OrOptParse<'db, Variant1> {
    fn or_opt_parse<Enum, Variant2>(
        self,
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Enum>, ParseFail<'db>>
    where
        Variant1: Into<Enum>,
        Variant2: Parse<'db, Output: Into<Enum>>;
}

impl<'db, Variant1> OrOptParse<'db, Variant1> for Result<Option<Variant1>, ParseFail<'db>> {
    fn or_opt_parse<Enum, Variant2>(
        self,
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Enum>, ParseFail<'db>>
    where
        Variant1: Into<Enum>,
        Variant2: Parse<'db, Output: Into<Enum>>,
    {
        match self {
            Ok(Some(v1)) => Ok(Some(v1.into())),
            Ok(None) => match Variant2::opt_parse(db, parser) {
                Ok(Some(v2)) => Ok(Some(v2.into())),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }
}
