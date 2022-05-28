use crate::{
    filename::Filename,
    span::{LineColumn, Offset, Span},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LineTable {
    /// Always has at least one element for the first line
    lines: Vec<LineInfo>,
    end_offset: Offset,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct LineInfo {
    /// Offset of line start
    start: Offset,
    /// Spans of chars with utf8 length > 1
    wide_chars: Vec<Span>,
}

impl LineTable {
    fn new(source_text: &str) -> Self {
        let mut table = LineTable {
            lines: vec![LineInfo {
                start: Offset::from(0u32),
                wide_chars: Vec::new(),
            }],
            end_offset: Offset::from(source_text.len()),
        };
        for (i, c) in source_text.char_indices() {
            if c == '\n' {
                table.lines.push(LineInfo {
                    start: Offset::from(i + 1),
                    wide_chars: Vec::new(),
                })
            } else if c.len_utf8() > 1 {
                table.lines.last_mut().unwrap().wide_chars.push(Span {
                    start: Offset::from(i),
                    end: Offset::from(i + c.len_utf8()),
                });
            }
        }
        table
    }

    fn num_lines(&self) -> usize {
        self.lines.len()
    }

    fn offset(&self, position: LineColumn) -> Offset {
        if position.line0_usize() >= self.num_lines() {
            return self.end_offset;
        }
        let line = &self.lines[position.line0_usize()];
        let mut offset = u32::from(line.start + position.column0());
        for wc in line.wide_chars.iter() {
            if u32::from(wc.start) < offset {
                offset += wc.len() - 1;
            }
        }
        Offset::from(offset).min(self.end_offset)
    }

    fn line_column(&self, position: Offset) -> LineColumn {
        match self.lines.binary_search_by_key(&position, |l| l.start) {
            Ok(line0) => LineColumn::new0(line0, 0u32),
            Err(next_line0) => {
                let line0 = next_line0 - 1;
                let line = &self.lines[line0];
                // not quite column yet, because there may be wide characters between line start and position
                // at this point it's the byte offset from line start
                // we need to adjust for it
                let mut column0 = position - line.start;
                for wc in line.wide_chars.iter() {
                    if wc.start >= position {
                        break;
                    }
                    // e.g.: 🙂 will have len 4, but we count it as 1 character, so we substract 3
                    column0 -= wc.len() - 1;
                }
                LineColumn::new0(line0, column0)
            }
        }
    }
}

/// Converts a character index `position` into a line and column tuple.
pub fn line_column(db: &dyn crate::Db, filename: Filename, position: Offset) -> LineColumn {
    let table = line_table(db, filename);
    table.line_column(position)
}

/// Given a line/column tuple, returns a character index.
pub fn offset(db: &dyn crate::Db, filename: Filename, position: LineColumn) -> Offset {
    let table = line_table(db, filename);
    table.offset(position)
}

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
fn line_table(db: &dyn crate::Db, filename: Filename) -> LineTable {
    let source_text = crate::manifest::source_text(db, filename);
    LineTable::new(source_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset_to_line_column_naive(source_text: &str, position: Offset) -> LineColumn {
        let mut line: u32 = 0;
        let mut col: u32 = 0;
        for (i, c) in source_text.char_indices() {
            if Offset::from(i) == position {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        LineColumn::new0(line, col)
    }

    fn check_line_column(source_text: &str) {
        let line_table = LineTable::new(source_text);
        for (i, _) in source_text.char_indices() {
            let offset = Offset::from(i);
            let expected = offset_to_line_column_naive(source_text, offset);
            let actual = line_table.line_column(offset);
            assert_eq!(expected, actual, "at {:?}", offset);
            let round_trip = line_table.offset(actual);
            assert_eq!(offset, round_trip);
        }
    }

    #[test]
    fn crlf_line_endings() {
        check_line_column("foo\r\nb🙂ar\r\nbaz")
    }

    #[test]
    fn lf_line_endings() {
        check_line_column("foo\nb🙂ar\nbaz")
    }
}
