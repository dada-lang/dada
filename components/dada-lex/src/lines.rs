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

/// Converts a character index `position` into a (1-based) line and column tuple.
pub fn line_column(db: &dyn crate::Db, filename: Word, position: Offset) -> LineColumn {
    let table = line_table(db, filename);
    match table.line_endings.binary_search(&position) {
        Ok(line) => LineColumn {
            line: line as u32 + 1,
            column: 1,
        },
        Err(0) => LineColumn {
            line: 1,
            column: (position + 1_u32).into(),
        },
        Err(line) => {
            let end_previous_line = table.line_endings[line - 1];
            LineColumn {
                line: line as u32 + 1,
                column: position - end_previous_line + 1,
            }
        }
    }
}

#[salsa::memoized(in crate::Jar ref)]
pub fn line_table(db: &dyn crate::Db, filename: Word) -> LineTable {
    let source_text = dada_manifest::source_text(db, filename);
    let mut p: usize = 0;
    let mut table = LineTable {
        line_endings: vec![],
    };
    for line in source_text.lines() {
        p += line.len();
        table.line_endings.push(Offset::from(p));
    }
    table
}
