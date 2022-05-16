use crate::{
    filename::Filename,
    span::{LineColumn, Offset},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LineTable {
    /// Stores the index of the `\n` for each line.
    /// So `0..line_endings[0]` represents the range of characters for the first line
    /// and so forth.
    line_endings: Vec<Offset>,
    end_offset: Offset,
}

impl LineTable {
    fn new(source_text: &str) -> Self {
        let mut p: usize = 0;
        let mut table = LineTable {
            line_endings: vec![],
            end_offset: Offset::from(source_text.len()),
        };
        for line in source_text.lines() {
            p += line.len();
            table.line_endings.push(Offset::from(p));
            p += 1;
        }
        table
    }

    /// Given a (1-based) line number, find the start of the line.
    ///
    /// If `line` is out of range, panics.
    fn line_start(&self, line0: usize) -> Offset {
        if line0 == 0 {
            Offset::from(0_u32)
        } else {
            let previous_line0 = line0 - 1;
            self.line_endings[previous_line0] + 1_u32
        }
    }

    fn num_lines(&self) -> usize {
        self.line_endings.len() + 1
    }

    fn offset(&self, position: LineColumn) -> Offset {
        if position.line0_usize() >= self.num_lines() {
            return self.end_offset;
        }
        let line_start = self.line_start(position.line0_usize());
        (line_start + position.column0()).min(self.end_offset)
    }

    fn line_column(&self, position: Offset) -> LineColumn {
        match self.line_endings.binary_search(&position) {
            Ok(line0) | Err(line0) => {
                let line_start = self.line_start(line0);
                LineColumn::new0(line0, position - line_start)
            }
        }
    }
}

/// Converts a character index `position` into a line and column tuple.
pub fn line_column(db: &dyn crate::Db, filename: Filename, position: Offset) -> LineColumn {
    let table = line_table(db, filename);
    table.line_column(position)
}

/// Given a (1-based) line/column tuple, returns a character index.
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
            if i as u32 == position.into() {
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
        for p in 0..source_text.chars().count() {
            let offset = p.into();
            let expected = offset_to_line_column_naive(source_text, offset);
            let actual = line_table.line_column(offset);
            assert_eq!(expected, actual, "at {:?}", offset);
        }
    }

    #[test]
    fn crlf_line_endings() {
        check_line_column("foo\r\nbar\r\nbaz")
    }

    #[test]
    fn lf_line_endings() {
        check_line_column("foo\nbar\nbaz")
    }
}
