use crate::token_test::Identifier;

use dada_ir::code::{
    syntax::op::Op,
    syntax::{Name, NameData, Path, PathData},
};

use super::CodeParser;

impl CodeParser<'_, '_> {
    /// Parse a `foo` name.
    pub(super) fn parse_name(&mut self) -> Option<Name> {
        let (word_span, word) = self.eat(Identifier)?;
        Some(self.add(NameData { word }, word_span))
    }

    /// Parse a `foo` or `foo.bar` (or `foo.bar.baz`...) path.
    pub(super) fn parse_path(&mut self) -> Option<Path> {
        let name = self.parse_name()?;
        let mut path = self.add(PathData::Name(name), self.spans[name]);

        while self.eat_op(Op::Dot).is_some() {
            if let Some(name) = self.parse_name() {
                path = self.add(
                    PathData::Dot(path, name),
                    self.span_consumed_since_parsing(name),
                );
            } else {
                self.error_at_current_token("expected a name after a `.` to make a path")
                    .emit(self.db);
                break;
            }
        }

        Some(path)
    }
}
