use dada_collections::Map;
use dada_ir::{class::Class, code::validated, func::Function, word::Word};
use dada_parse::prelude::*;

pub(crate) struct Scope {
    filename: Word,
    local_variables: Map<Word, validated::LocalVariable>,
}

pub(crate) enum LookupResult {
    LocalVariable(validated::LocalVariable),
    Function(Function),
    Class(Class),
}

impl Scope {
    pub(crate) fn root(filename: Word) -> Self {
        Self {
            filename,
            local_variables: Map::default(),
        }
    }

    pub(crate) fn subscope(&self) -> Self {
        Self {
            filename: self.filename,
            local_variables: self.local_variables.clone(),
        }
    }

    pub(crate) fn insert(&mut self, name: Word, local_variable: validated::LocalVariable) {
        self.local_variables.insert(name, local_variable);
    }

    pub(crate) fn lookup(&self, name: Word) -> Option<LookupResult> {
        if let Some(local_variable) = self.local_variables.get(&name) {
            return Some(LookupResult::LocalVariable(*local_variable));
        }

        todo!()
    }
}
