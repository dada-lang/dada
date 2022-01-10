use crate::{code::Code, filename::Filename, span::FileSpan, word::Word};

salsa::entity2! {
    entity Function in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        code: Code,

        /// If this func has a declared effect, this is the span of that keyword (e.g., `async`)
        /// Otherwise, it is the span of the `fn` keyword.
        effect_span: FileSpan,
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        write!(f, "{}", self.name(db).as_str(db))
    }
}

impl Function {
    pub fn filename(self, db: &dyn crate::Db) -> Filename {
        self.code(db).filename(db)
    }
}

salsa::entity2! {
    entity Variable in crate::Jar {
        #[id] name: Word,
    }
}
