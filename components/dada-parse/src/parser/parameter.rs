use crate::{parser::Parser, token_test::Identifier};

use dada_ir::{
    kw::Keyword,
    op::Op,
    parameter::{Parameter, ParameterSpans},
    span::Span,
    storage_mode::StorageMode,
};

use super::ParseList;

impl<'db> Parser<'db> {
    pub(crate) fn parse_only_parameters(&mut self) -> Vec<Parameter> {
        let p = self.parse_list(true, Parser::parse_parameter);
        self.emit_error_if_more_tokens("extra tokens after parameters");
        p
    }

    fn parse_parameter(&mut self) -> Option<Parameter> {
        let opt_storage_mode = self.parse_storage_mode();
        if let Some((name_span, name)) = self.eat(Identifier) {
            let opt_ty = if let Some(colon_span) = self.eat_op(Op::Colon) {
                let opt_ty = self.parse_ty();

                if opt_ty.is_none() {
                    self.error_at_current_token(&format!("expected type after `:`"))
                        .label(colon_span, format!("`:` is here"))
                        .emit(self.db);
                }

                opt_ty
            } else {
                None
            };

            let (mode_span, mode) = match opt_storage_mode {
                Some(pair) => pair,
                None => (name_span, StorageMode::Shared),
            };

            let spans = ParameterSpans {
                name: name_span,
                mode: mode_span,
            };

            Some(Parameter::new(self.db, name, mode, opt_ty, spans))
        } else {
            // No identifier == no parameter; if there's a storage mode,
            // that's an error.
            if let Some((span, mode)) = opt_storage_mode {
                self.error_at_current_token(format!(
                    "expected parameter name after storage mode `{mode}`"
                ))
                .label(span, "storage mode specified here")
                .emit(self.db);
            }

            None
        }
    }

    fn parse_storage_mode(&mut self) -> Option<(Span, StorageMode)> {
        if let Some((span, _)) = self.eat(Keyword::Shared) {
            Some((span, StorageMode::Shared))
        } else if let Some((span, _)) = self.eat(Keyword::Var) {
            Some((span, StorageMode::Var))
        } else if let Some((span, _)) = self.eat(Keyword::Atomic) {
            Some((span, StorageMode::Atomic))
        } else {
            None
        }
    }
}
