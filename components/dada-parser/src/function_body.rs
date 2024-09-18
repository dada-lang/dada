use dada_ir_ast::{
    ast::{AstBlock, AstExpr, AstLetStatement, AstStatement, AstTy, FunctionBody},
    span::Offset,
};

use crate::{
    miscellaneous::OrOptParse,
    tokenizer::{tokenize, Keyword},
    Parse, Parser,
};

#[salsa::tracked]
impl<'db> crate::prelude::FunctionBodyBlock<'db> for FunctionBody<'db> {
    #[salsa::tracked]
    fn block(self, db: &'db dyn crate::Db) -> AstBlock<'db> {
        let contents = self.contents(db);
        let tokens = tokenize(db, self.into(), Offset::ZERO, contents);
        let statements = Parser::new(db, self.into(), &tokens)
            .parse_many_and_report_diagnostics::<AstStatement>(db);
        AstBlock::new(db, statements)
    }
}

impl<'db> Parse<'db> for AstBlock<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Some(statements) = AstStatement::opt_parse_delimited(
            db,
            parser,
            crate::tokenizer::Delimiter::CurlyBraces,
            AstStatement::eat_many,
        )?
        else {
            return Ok(None);
        };

        Ok(Some(AstBlock::new(db, statements)))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("block")
    }
}

impl<'db> Parse<'db> for AstStatement<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        AstLetStatement::opt_parse(db, parser).or_opt_parse::<Self, AstExpr>(db, parser)
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("statement")
    }
}

impl<'db> Parse<'db> for AstLetStatement<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Ok(_) = parser.eat_keyword(Keyword::Let) else {
            return Ok(None);
        };
        let name = parser.eat_id()?;
        let ty = AstTy::opt_parse_guarded(":", db, parser)?;
        let initializer = AstExpr::opt_parse_guarded("=", db, parser)?;
        Ok(Some(AstLetStatement::new(db, name, ty, initializer)))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("let statement")
    }
}
