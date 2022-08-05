use dada_ir::{
    input_file::InputFile,
    span::{FileSpan, Offset},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DadaRange {
    pub start: DadaLineColumn,
    pub end: DadaLineColumn,
}

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DadaLineColumn {
    pub line0: u32,
    pub column0: u32,
}

#[wasm_bindgen]
impl DadaLineColumn {
    pub(crate) fn from(db: &dada_db::Db, input_file: InputFile, position: Offset) -> Self {
        let lc = dada_ir::lines::line_column(db, input_file, position);
        Self {
            line0: lc.line0(),
            column0: lc.column0(),
        }
    }
}

#[wasm_bindgen]
impl DadaRange {
    pub(crate) fn from(db: &dada_db::Db, span: FileSpan) -> Self {
        Self {
            start: DadaLineColumn::from(db, span.input_file, span.start),
            end: DadaLineColumn::from(db, span.input_file, span.end),
        }
    }
}
