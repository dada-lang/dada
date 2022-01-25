use dada_ir::{
    filename::Filename,
    span::{FileSpan, Offset},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct DadaDiagnostic {
    severity: dada_ir::diagnostic::Severity,
    primary_label: DadaLabel,
    #[allow(dead_code)]
    secondary_labels: Vec<DadaLabel>,
    #[allow(dead_code)]
    children: Vec<DadaDiagnostic>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct DadaLabel {
    pub start: DadaLocation,
    pub end: DadaLocation,
    message: String,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct DadaLocation {
    pub line0: u32,
    pub column0: u32,
}

#[wasm_bindgen]
impl DadaDiagnostic {
    pub(crate) fn from(db: &dada_db::Db, diagnostic: &dada_ir::diagnostic::Diagnostic) -> Self {
        let primary_label = DadaLabel::from(db, diagnostic.span, &diagnostic.message);
        let secondary_labels = diagnostic
            .labels
            .iter()
            .map(|l| DadaLabel::from(db, l.span, &l.message))
            .collect();
        let children = diagnostic
            .children
            .iter()
            .map(|c| DadaDiagnostic::from(db, c))
            .collect();
        DadaDiagnostic {
            severity: diagnostic.severity,
            primary_label,
            secondary_labels,
            children,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn severity(&self) -> String {
        match self.severity {
            dada_ir::diagnostic::Severity::Help => "help",
            dada_ir::diagnostic::Severity::Note => "note",
            dada_ir::diagnostic::Severity::Warning => "warning",
            dada_ir::diagnostic::Severity::Error => "error",
        }
        .to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn primary_label(&self) -> DadaLabel {
        self.primary_label.clone()
    }
}

#[wasm_bindgen]
impl DadaLabel {
    pub(crate) fn from(db: &dada_db::Db, span: FileSpan, message: &str) -> Self {
        let start = DadaLocation::from(db, span.filename, span.start);
        let end = DadaLocation::from(db, span.filename, span.end);
        DadaLabel {
            start,
            end,
            message: message.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[wasm_bindgen]
impl DadaLocation {
    pub(crate) fn from(db: &dada_db::Db, filename: Filename, position: Offset) -> Self {
        let lc = dada_ir::lines::line_column(db, filename, position);
        Self {
            line0: lc.line0(),
            column0: lc.column0(),
        }
    }
}
