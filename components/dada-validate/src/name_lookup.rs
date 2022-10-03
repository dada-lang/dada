use dada_collections::Map;
use dada_ir::{
    class::Class, function::Function, input_file::InputFile, intrinsic::Intrinsic, item::Item,
    word::Word,
};
use dada_parse::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;

/// A scope manages name lookups. The type LV is the type used to represent local variables.
pub(crate) struct Scope<'me, LV> {
    db: &'me dyn crate::Db,
    names: Map<Word, Definition<LV>>,
    inserted: Vec<LV>,
}

/// Root definitions at the top of the file. Never contains local variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootDefinitions {
    names: Map<Word, Definition<NoLocalVariable>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Definition<LV> {
    LocalVariable(LV),
    Function(Function),
    Class(Class),
    Intrinsic(Intrinsic),
}

impl<LV> Definition<LV> {
    pub(crate) fn plural_description(&self) -> &str {
        match self {
            Definition::LocalVariable(_) => "variables",
            Definition::Function(_) => "functions",
            Definition::Class(_) => "classes",
            Definition::Intrinsic(_) => "functions",
        }
    }

    pub(crate) fn from_root_definition(d: Definition<NoLocalVariable>) -> Self {
        match d {
            Definition::LocalVariable(nlv) => match nlv {},
            Definition::Function(f) => Definition::Function(f),
            Definition::Class(c) => Definition::Class(c),
            Definition::Intrinsic(i) => Definition::Intrinsic(i),
        }
    }
}

impl<LV> From<Item> for Definition<LV> {
    fn from(value: Item) -> Self {
        match value {
            Item::Function(f) => Definition::Function(f),
            Item::Class(c) => Definition::Class(c),
        }
    }
}

impl<LV> TryInto<Item> for Definition<LV> {
    type Error = ();
    fn try_into(self) -> Result<Item, ()> {
        match self {
            Definition::LocalVariable(_) => Err(()),
            Definition::Intrinsic(_) => Err(()),
            Definition::Function(f) => Ok(Item::Function(f)),
            Definition::Class(c) => Ok(Item::Class(c)),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum NoLocalVariable {}

impl<'me, LV> Scope<'me, LV>
where
    LV: Debug + Copy + Hash + Eq,
{
    /// Constructs the root scope for a file, reporting errors if there are
    /// duplicate items.
    pub(crate) fn root(db: &'me dyn crate::Db, root_definitions: &RootDefinitions) -> Self {
        let names = root_definitions
            .names
            .iter()
            .map(|(&word, &rd)| (word, Definition::from_root_definition(rd)))
            .collect();
        Self {
            db,
            names,
            inserted: vec![],
        }
    }

    pub(crate) fn subscope(&self) -> Self {
        Self {
            db: self.db,
            names: self.names.clone(),
            inserted: vec![],
        }
    }

    /// Inserts a local variable into the scope. Returns any definition that is now shadowed as a result.
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(crate) fn insert(&mut self, name: Word, local_variable: LV) -> Option<Definition<LV>> {
        self.inserted.push(local_variable);
        self.names
            .insert(name, Definition::LocalVariable(local_variable))
    }

    /// Tracks a temporary that is created; they don't affect name resolution, but they get
    /// dropped at the same time as local variables in the surrounding scope.
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(crate) fn insert_temporary(&mut self, local_variable: LV) {
        self.inserted.push(local_variable);
    }

    /// Lookup the given name in the scope.
    pub(crate) fn lookup(&self, name: Word) -> Option<Definition<LV>> {
        self.names.get(&name).copied()
    }

    /// Get the vector of inserted names from this scope (replacing it with `vec![]`);
    /// used when exiting the scope, see [`Validator::exit_subscope`].
    pub(crate) fn take_inserted(&mut self) -> Vec<LV> {
        std::mem::take(&mut self.inserted)
    }
}

impl RootDefinitions {
    pub fn new(db: &dyn crate::Db, input_file: InputFile) -> Self {
        let items = input_file.items(db);
        let mut names: Map<Word, Definition<NoLocalVariable>> = Map::default();

        // Populate the names table with the global definitions to start
        for &item in items {
            let name = item.name(db);

            if let Some(&other_definition) = names.get(&name) {
                let other_item: Item = other_definition.try_into().unwrap();
                dada_ir::error!(
                    item.name_span(db),
                    "already have a {} named `{}`",
                    other_item.kind_str(),
                    name.as_str(db),
                )
                .primary_label(format!("ignoring this {} for now", item.kind_str()))
                .secondary_label(
                    other_item.name_span(db),
                    format!("the {} is here", other_item.kind_str()),
                )
                .emit(db);
            } else {
                names.insert(name, Definition::from(item));
            }
        }

        // Populate with intrinsics from the prelude (these can be shadowed, so don't error if
        // user generates something with the same name)
        for &intrinsic in Intrinsic::ALL {
            names.insert(intrinsic.name(db), Definition::Intrinsic(intrinsic));
        }

        RootDefinitions { names }
    }
}
