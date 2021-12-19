use dada_ir::{
    span::{LineColumn, Offset},
    word::Word,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LineTable {
    /// Stores the index of the `\n` for each line.
    /// So `0..line_endings[0]` represents the range of characters for the first line
    /// and so forth.
    line_endings: Vec<Offset>,
}

impl LineTable {
    fn line_start(&self, line: usize) -> Offset {
        if line == 0 {
            Offset::from(0_u32)
        } else {
            self.line_endings[line - 1] + 1_u32
        }
    }
}

/// Converts a character index `position` into a (1-based) line and column tuple.
pub fn line_column(db: &dyn crate::Db, filename: Word, position: Offset) -> LineColumn {
    let table = line_table(db, filename);
    match table.line_endings.binary_search(&position) {
        Ok(line) | Err(line) => {
            let line_start = table.line_start(line);
            LineColumn {
                line: line as u32 + 1,
                column: position - line_start + 1,
            }
        }
    }
}

#[salsa::memoized(in crate::Jar ref)]
fn line_table(db: &dyn crate::Db, filename: Word) -> LineTable {
    let source_text = dada_manifest::source_text(db, filename);
    let mut p: usize = 0;
    let mut table = LineTable {
        line_endings: vec![],
    };
    for line in source_text.lines() {
        p += line.len();
        table.line_endings.push(Offset::from(p));
        p += 1;
    }
    table
}
