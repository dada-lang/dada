use crate::{parser::Parser, token_test::Identifier};

use dada_ir::{
    code::syntax::op::Op,
    code::syntax::{LocalVariableDecl, LocalVariableDeclData, LocalVariableDeclSpan},
    storage::Atomic,
};

use super::{CodeParser, ParseList};

impl CodeParser<'_, '_> {
    /// Parses a list of parameters delimited by `()`.
    pub(crate) fn parse_parameter_list(&mut self) -> Option<Vec<LocalVariableDecl>> {
        let (_, parameter_tokens) = self.delimited('(')?;
        let mut subparser = Parser::new(self.db, parameter_tokens);
        let mut subcodeparser = CodeParser {
            parser: &mut subparser,
            tables: self.tables,
            spans: self.spans,
        };
        Some(subcodeparser.parse_only_parameters())
    }

    fn parse_only_parameters(&mut self) -> Vec<LocalVariableDecl> {
        let p = self.parse_list(true, CodeParser::parse_parameter);
        self.emit_error_if_more_tokens("extra tokens after parameters");
        p
    }

    fn parse_parameter(&mut self) -> Option<LocalVariableDecl> {
        let opt_storage_mode = self.parse_atomic();
        if let Some((name_span, name)) = self.eat(Identifier) {
            let opt_ty = if let Some(colon_span) = self.eat_op(Op::Colon) {
                let opt_ty = self.parse_ty();

                if opt_ty.is_none() {
                    self.error_at_current_token(&"expected type after `:`".to_string())
                        .secondary_label(colon_span, "`:` is here".to_string())
                        .emit(self.db);
                }

                opt_ty
            } else {
                None
            };

            let (atomic_span, atomic) = match opt_storage_mode {
                Some(span) => (span, Atomic::Yes),
                None => (name_span, Atomic::No),
            };

            let decl = self.add(
                LocalVariableDeclData {
                    atomic,
                    name,
                    ty: opt_ty,
                },
                LocalVariableDeclSpan {
                    atomic_span,
                    name_span,
                },
            );

            Some(decl)
        } else {
            // No identifier == no parameter; if there's a storage mode,
            // that's an error.
            if let Some(span) = opt_storage_mode {
                self.error_at_current_token("expected parameter name after `atomic`")
                    .secondary_label(span, "`atomic` specified here")
                    .emit(self.db);
            }

            None
        }
    }
}
