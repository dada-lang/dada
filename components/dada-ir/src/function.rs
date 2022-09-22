#![allow(clippy::too_many_arguments)] // omg clippy, mind your own business

use crate::{
    code::{
        syntax::{EffectKeyword, Signature},
        UnparsedCode,
    },
    effect::Effect,
    input_file::InputFile,
    return_type::ReturnType,
    span::{Anchored, Span},
    word::Word,
};

#[salsa::tracked]
pub struct Function {
    #[id]
    name: Word,

    input_file: InputFile,

    /// The function signature.
    #[return_ref]
    signature: FunctionSignature,

    /// Return type of the function.
    return_type: ReturnType,

    /// The body and parameters of functions are only parsed
    /// on demand by invoking (e.g.) `syntax_tree` from the
    /// `dada_parse` crate.
    ///
    /// If this is `None`, then the syntax-tree that would've
    /// been parsed must be specified explicitly by the
    /// creator of the function. This is used for synthesizing
    /// a 'main' function from a module, for example.
    unparsed_code: Option<UnparsedCode>,

    /// Overall span of the function (including the code)
    span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum FunctionSignature {
    /// The signature is derived from the following syntax written by the user.
    Syntax(Signature),

    /// The signature is the generated signature of the main function.
    Main,
}

impl Anchored for Function {
    fn input_file(&self, db: &dyn crate::Db) -> InputFile {
        Function::input_file(*self, db)
    }
}

impl Function {
    /// Returns the span of the function name.
    ///
    /// If this is a synthesized function (i.e., `main`),
    /// then we just return the entire source from
    /// which the function was synthesized.
    pub fn name_span(self, db: &dyn crate::Db) -> Span {
        match self.signature(db) {
            FunctionSignature::Syntax(s) => s.spans[s.name],

            FunctionSignature::Main => self.span(db),
        }
    }

    /// Returns the "effect" of the function -- is it async? atomic? Default?
    pub fn effect(self, db: &dyn crate::Db) -> Effect {
        match self.signature(db) {
            FunctionSignature::Syntax(s) => match s.effect {
                Some(EffectKeyword::Async(_)) => Effect::Async,
                Some(EffectKeyword::Atomic(_)) => Effect::Atomic,
                None => Effect::Default,
            },

            FunctionSignature::Main => Effect::Async,
        }
    }

    /// Returns the span where the effect was declared (if any).
    /// If there is no declared effect (e.g., just `fn foo()`), returns the span of the `fn` keyword.
    ///
    /// (In the case of a synthetic main function, returns the span of the entire function.)
    pub fn effect_span(self, db: &dyn crate::Db) -> Span {
        match self.signature(db) {
            FunctionSignature::Syntax(s) => match s.effect {
                Some(EffectKeyword::Async(k)) => s.spans[k],
                Some(EffectKeyword::Atomic(k)) => s.spans[k],
                None => s.spans[s.fn_decl],
            },

            FunctionSignature::Main => self.span(db),
        }
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        write!(f, "{}", self.name(db).as_str(db))
    }
}

#[salsa::tracked]
pub struct Variable {
    #[id]
    name: Word,
}
