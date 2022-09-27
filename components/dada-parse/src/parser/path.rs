use crate::token_test::Identifier;

use dada_ir::code::{
    syntax::op::Op,
    syntax::{Name, NameData, Path, PathData},
};

use super::CodeParser;
use super::ParseList;

impl CodeParser<'_, '_> {
    /// Parse a `foo` name.
    pub(super) fn parse_name(&mut self) -> Option<Name> {
        let (word_span, word) = self.eat(Identifier)?;
        Some(self.add(NameData { word }, word_span))
    }

    /// Parse a list of paths, reporting an error if anthing else remains after.
    pub(super) fn parse_only_paths(&mut self) -> Vec<Path> {
        let p = self.parse_list(true, CodeParser::parse_path);
        self.emit_error_if_more_tokens("extra tokens after paths");
        p
    }

    /// Parse a `foo` or `foo.bar` (or `foo.bar.baz`...) path.
    pub(super) fn parse_path(&mut self) -> Option<Path> {
        let start_name = self.parse_name()?;
        let mut dot_names = vec![];

        while self.eat_op(Op::Dot).is_some() {
            if let Some(name) = self.parse_name() {
                dot_names.push(name);
            } else {
                self.error_at_current_token("expected a name after a `.` to make a path")
                    .emit(self.db);
                break;
            }
        }

        Some(self.add(
            PathData {
                start_name,
                dot_names,
            },
            self.span_consumed_since_parsing(start_name),
        ))
    }
}
