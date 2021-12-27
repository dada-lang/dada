use crate::{code::Code, parameter::UnparsedParameters, span::FileSpan, word::Word};

salsa::entity2! {
    entity Function in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        effect: Effect,
        unparsed_parameters: UnparsedParameters,
        code: Code,
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        write!(f, "{}", self.name(db).as_str(db))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Effect {
    None,
    Async,
}

salsa::entity2! {
    entity Variable in crate::Jar {
        #[id] name: Word,
    }
}
