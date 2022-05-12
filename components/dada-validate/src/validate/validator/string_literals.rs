use super::*;

impl Validator<'_> {
    ///
    pub(super) fn concatenate(
        &mut self,
        concatenate_expr: syntax::Expr,
        exprs: &[syntax::Expr],
    ) -> validated::Expr {
        // See https://dada-lang.org/docs/reference/string-literals for full details.

        let validated_exprs = if !self.should_strip_margin(exprs) {
            exprs
                .iter()
                .map(|expr| self.reserve_validated_expr(*expr))
                .collect()
        } else {
            self.strip_margin_from_exprs(exprs)
        };

        self.add(
            validated::ExprData::Concatenate(validated_exprs),
            concatenate_expr,
        )
    }

    pub(super) fn support_escape(&self, expr: syntax::Expr, s: &str) -> String {
        let mut buffer = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(c) = chars.peek() {
                    match c {
                        'n' | 'r' | 't' | '"' | '\\' | '{' | '}' => {
                            buffer.push(escape(*c));
                            chars.next();
                            continue;
                        }
                        _ => {
                            // FIXME: it'd be better to compute the exact offset within the string
                            // literal, but that's a *touch* tricky to do since `s` may be some
                            // stripped subset. We'd either have to track original span info for `s`
                            // or else recompute it.
                            dada_ir::error!(self.span(expr), "unrecognized escape `\\{c}`",)
                                .emit(self.db);
                        }
                    }
                }
            }
            buffer.push(ch);
        }
        return buffer;

        #[track_caller]
        fn escape(ch: char) -> char {
            match ch {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '"' => '\"',
                '{' => '{',
                '}' => '}',
                _ => panic!("not a escape: {:?}", ch),
            }
        }
    }

    /// If the initial string begins with a literal (not escaped) newline, we
    /// need to strip margin. Otherwise, we do nothing but modify escapes.
    fn should_strip_margin(&self, exprs: &[syntax::Expr]) -> bool {
        if exprs.is_empty() {
            return false;
        }

        if let syntax::ExprData::StringLiteral(word) = exprs[0].data(self.syntax_tables()) {
            let word_str = word.as_str(self.db);
            word_str.starts_with('\n')
        } else {
            false
        }
    }

    fn strip_margin_from_exprs(&mut self, exprs: &[syntax::Expr]) -> Vec<validated::Expr> {
        let margin = self.compute_margin(exprs);

        let mut validated_exprs = Vec::with_capacity(exprs.len());
        for (expr, index) in exprs.iter().zip(0..) {
            if let syntax::ExprData::StringLiteral(word) = expr.data(self.syntax_tables()) {
                let word_str = word.as_str(self.db);
                let without_margin = self.strip_margin_from_str(margin, word_str);
                let mut without_margin = &without_margin[..];

                // We always strip leading/trailing newline, because people tend to write
                // things like
                //
                // ```
                // print("
                //     Foo
                // ");
                // ```
                //
                // and we want that to print "Foo". Note that we haven't applied
                // escapes yet, so `print("\n")` is unaffected.
                if index == 0 {
                    if let Some(s) = without_margin.strip_prefix('\n') {
                        without_margin = s;
                    }
                }

                if index == exprs.len() - 1 {
                    if let Some(s) = without_margin.strip_suffix('\n') {
                        without_margin = s;
                    }
                }

                // Finally, expand escapes.
                let escaped = self.support_escape(*expr, without_margin);
                let word = Word::from(self.db, escaped);
                validated_exprs.push(self.add(validated::ExprData::StringLiteral(word), *expr));
            } else {
                validated_exprs.push(self.reserve_validated_expr(*expr));
            }
        }
        validated_exprs
    }

    /// Returns the number of characters that should be stripped from the start of
    /// every (non-empty) line. Dada format string literals automatically strip
    /// the "margin", which is common whitespace that appears at the start of
    /// every line (ignoring empty lines).
    fn compute_margin(&self, exprs: &[syntax::Expr]) -> usize {
        let mut dummy_string = String::new();
        for expr in exprs {
            if let syntax::ExprData::StringLiteral(s) = expr.data(self.syntax_tables()) {
                dummy_string.push_str(s.as_str(self.db));
            } else {
                dummy_string.push_str("{...}");
            }
        }

        // Find the first line that is not entirely whitespace, and compute its
        // whitespace prefix from that. We will then intersect this with all other
        // lines.
        let prefix: String = dummy_string
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(1)
            .flat_map(|line| line.chars())
            .take_while(|c| c.is_whitespace())
            .collect();

        dummy_string
            .lines()
            .map(|s| {
                let c = count_bytes_in_common(prefix.as_bytes(), s.as_bytes());
                if c == s.len() {
                    // Careful: if some line consists entirely of whitespace that is part of the
                    // prefix, then we still keep the entire prefix.
                    prefix.len()
                } else {
                    c
                }
            })
            .min()
            .unwrap_or(0)
    }

    fn strip_margin_from_str(&self, margin: usize, word: &str) -> String {
        itertools::Itertools::intersperse(
            word.split('\n').zip(0..).map(|(line, index)| {
                if index == 0 {
                    // We only strip whitespace after a `\n` has been found.
                    // When you have a string like `"  foo\n  bar"`
                    line
                } else if line.len() < margin {
                    assert!(
                        line.trim().is_empty(),
                        "margin of {margin} is not entirely whitespace in {line:?}"
                    );
                    ""
                } else {
                    assert!(
                        line[..margin].trim().is_empty(),
                        "margin of {margin} is not entirely whitespace in {line:?}"
                    );
                    &line[margin..]
                }
            }),
            "\n",
        )
        .collect()
    }
}
