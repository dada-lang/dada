use crate::{
    code::UnparsedCode,
    effect::Effect,
    input_file::InputFile,
    return_type::ReturnType,
    span::FileSpan,
    word::{SpannedWord, Word},
};

#[salsa::tracked]
pub struct Function {
    #[id]
    name: SpannedWord,

    /// Declared effect for the function body -- e.g., `async fn` would have
    /// this be `async`. This can affect validation and code generation.
    effect: Effect,

    /// If this func has a declared effect, this is the span of that keyword (e.g., `async`)
    /// Otherwise, it is the span of the `fn` keyword.
    effect_span: FileSpan,

    /// Return type of the function.
    return_type: ReturnType,

    /// The body and parameters of functions are only parsed
    /// on demand by invoking (e.g.) `syntax_tree` from the
    /// `dada_parse` crate.
    ///
    /// If this is `None`, then the syntax-tree and parameter
    /// list that would've been parsed must be set explicitly
    /// by the creator of the function. This is used for synthesizing
    /// a 'main' function from a module, for example.
    unparsed_code: Option<UnparsedCode>,

    /// Overall span of the function (including the code)
    span: FileSpan,
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        write!(f, "{}", self.name(db).as_str(db))
    }
}

impl Function {
    pub fn input_file(self, db: &dyn crate::Db) -> InputFile {
        self.span(db).input_file
    }
}

#[salsa::tracked]
pub struct Variable {
    #[id]
    name: Word,
}
