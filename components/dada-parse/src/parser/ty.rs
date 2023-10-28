use dada_ir::{
    code::syntax::{op::Op, Perm, PermData, PermPaths, PermPathsData, Ty, TyData},
    kw::Keyword,
};

use super::{CodeParser, Parser, SpanFallover};

impl CodeParser<'_, '_> {
    /// Parse `: Ty`.
    pub(crate) fn parse_colon_ty(&mut self) -> Option<Ty> {
        let Some(colon_span) = self.eat_op(Op::Colon) else {
            return None;
        };
        let opt_ty = self.parse_ty();
        if opt_ty.is_none() {
            self.error_at_current_token("expected type after `:`")
                .secondary_label(colon_span, "`:` is here")
                .emit(self.db);
        }
        opt_ty
    }

    /// Parse a dada type like `my String`.
    pub(crate) fn parse_ty(&mut self) -> Option<Ty> {
        let perm = self.parse_perm();
        let path = self.parse_path();

        match (perm, path) {
            (None, None) => None,
            (Some(perm), None) => {
                self.error(
                    self.spans[perm],
                    "expected a path to follow this permission",
                )
                .emit(self.db);
                None
            }
            (perm, Some(path)) => Some(self.add(
                TyData { perm, path },
                self.span_consumed_since_parsing(perm.or_parsing(path)),
            )),
        }
    }

    pub(crate) fn parse_perm(&mut self) -> Option<Perm> {
        if let Some((span, _)) = self.eat(Keyword::My) {
            self.disallow_perm_paths(Keyword::My);
            Some(self.add(PermData::My, span))
        } else if let Some((span, _)) = self.eat(Keyword::Our) {
            self.disallow_perm_paths(Keyword::Our);
            Some(self.add(PermData::Our, span))
        } else if let Some((shared_span, _)) = self.eat(Keyword::Shared) {
            let paths = self.parse_perm_paths();
            Some(self.add(
                PermData::Shared(paths),
                self.span_consumed_since(shared_span),
            ))
        } else if let Some((leased_span, _)) = self.eat(Keyword::Leased) {
            let paths = self.parse_perm_paths();
            Some(self.add(
                PermData::Leased(paths),
                self.span_consumed_since(leased_span),
            ))
        } else if let Some((given_span, _)) = self.eat(Keyword::Given) {
            let paths = self.parse_perm_paths();
            Some(self.add(PermData::Given(paths), self.span_consumed_since(given_span)))
        } else {
            None
        }
    }

    pub(crate) fn disallow_perm_paths(&mut self, keyword: Keyword) {
        let Some((span, _)) = self.delimited('{') else {
            return;
        };
        self.error(
            span,
            format!("no paths are needed after the {keyword} permission"),
        )
        .emit(self.db);
    }

    pub(crate) fn parse_perm_paths(&mut self) -> Option<PermPaths> {
        let Some((span, token_tree)) = self.delimited('{') else {
            return None;
        };
        let mut parser = Parser::new(self.db, token_tree);
        let mut subparser = parser.code_parser(self.tables, self.spans);
        let paths = subparser.parse_only_paths();
        Some(self.add(PermPathsData { paths }, span))
    }
}
