use dada_ir_ast::span::Span;
use serde::Serialize;

use crate::{
    check::red::{Chain, RedTy},
    ir::indices::InferVarIndex,
};

use super::{InferenceVarBounds, InferenceVarData};

// Stripped down version of `InferenceVarData` that excludes `ArcOrElse` objects.
// Suitable for serialization and debugging.

#[derive(Serialize)]
struct InferenceVarDataExport<'a, 'db> {
    span: Span<'db>,
    is: Vec<bool>,
    isnt: Vec<bool>,
    bounds: InferenceVarBoundsExport<'a, 'db>,
}

#[derive(Serialize)]
enum InferenceVarBoundsExport<'a, 'db> {
    Perm {
        lower: Vec<&'a Chain<'db>>,
        upper: Vec<&'a Chain<'db>>,
    },

    Ty {
        perm: InferVarIndex,
        lower: Option<&'a RedTy<'db>>,
        upper: Option<&'a RedTy<'db>>,
    },
}

impl Serialize for InferenceVarData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self {
            span,
            is,
            isnt,
            bounds,
        } = self;

        let bounds = match bounds {
            InferenceVarBounds::Perm { lower, upper } => InferenceVarBoundsExport::Perm {
                lower: lower.iter().map(|pair| &pair.0).collect(),
                upper: upper.iter().map(|pair| &pair.0).collect(),
            },
            InferenceVarBounds::Ty { perm, lower, upper } => InferenceVarBoundsExport::Ty {
                perm: *perm,
                lower: lower.as_ref().map(|pair| &pair.0),
                upper: upper.as_ref().map(|pair| &pair.0),
            },
        };

        let export = InferenceVarDataExport {
            span: *span,
            is: is.iter().map(|option| option.is_some()).collect(),
            isnt: isnt.iter().map(|option| option.is_some()).collect(),
            bounds,
        };

        Serialize::serialize(&export, serializer)
    }
}
