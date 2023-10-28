use std::rc::Rc;

use dada_collections::Map;
use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir::{self, StatementData},
        syntax,
        validated::{self, ExprOrigin},
    },
    input_file::InputFile,
    origin_table::{HasOriginIn, PushOriginIn},
};

/// The "brewery" stores an "under construction" BIR,
/// along with the validated tree we are building it from.
/// New basic blocks, statements, etc can be allocated with the
/// `add` method. The contents of a basic block etc can be accessed
/// and mutated by indexing into the brewery (e.g., `&mut brewery[bb]`)
///
/// The brewery does not track the current location
/// in the IR; a [`Cursor`](`crate::cursor::Cursor`) is used for that.
pub struct Brewery<'me> {
    input_file: InputFile,
    pub(crate) breakpoints: &'me [syntax::Expr],
    validated_tree_data: &'me validated::TreeData,
    validated_origins: &'me validated::Origins,
    tables: &'me mut bir::Tables,
    origins: &'me mut bir::Origins,
    variables: Rc<Map<validated::LocalVariable, bir::LocalVariable>>,
    pub(crate) dummy_terminator: bir::ControlPoint,
}

impl<'me> Brewery<'me> {
    pub fn new(
        db: &'me dyn crate::Db,
        input_file: InputFile,
        breakpoints: &'me [syntax::Expr],
        validated_tree: validated::Tree,
        tables: &'me mut bir::Tables,
        origins: &'me mut bir::Origins,
    ) -> Self {
        let variables = Rc::new(map_variables(db, validated_tree, tables, origins));
        let validated_tree_data = validated_tree.data(db);
        let validated_origins = validated_tree.origins(db);
        let dummy_terminator = add(
            tables,
            origins,
            bir::ControlPointData::Terminator(bir::TerminatorData::Panic),
            *validated_tree_data.root_expr.origin_in(validated_origins),
        );
        Self {
            input_file,
            breakpoints,
            validated_tree_data,
            validated_origins,
            tables,
            origins,
            variables,
            dummy_terminator,
        }
    }

    pub fn expr_is_breakpoint(&self, expr: syntax::Expr) -> Option<usize> {
        self.breakpoints.iter().position(|bp| *bp == expr)
    }

    pub fn validated_tables(&self) -> &'me validated::Tables {
        &self.validated_tree_data.tables
    }

    pub fn input_file(&self) -> InputFile {
        self.input_file
    }

    pub fn origin<K>(&self, of: K) -> K::Origin
    where
        K: HasOriginIn<validated::Origins>,
    {
        of.origin_in(self.validated_origins).clone()
    }

    pub fn bir_origin<K>(&self, of: K) -> K::Origin
    where
        K: HasOriginIn<bir::Origins>,
    {
        of.origin_in(self.origins).clone()
    }

    /// Creates a new "no-op" statement that will start up a new basic block.
    /// The "next" statement will be the dummy terminator.
    pub fn dummy_block(&mut self, origin: ExprOrigin) -> bir::ControlPoint {
        self.add(
            bir::ControlPointData::Statement(StatementData {
                action: bir::ActionData::Noop,
                next: self.dummy_terminator,
            }),
            origin,
        )
    }

    pub fn add<V, O>(&mut self, data: V, origin: impl Into<O>) -> V::Key
    where
        V: dada_id::InternValue<Table = bir::Tables>,
        V::Key: PushOriginIn<bir::Origins, Origin = O>,
    {
        add(self.tables, self.origins, data, origin)
    }

    /// Converts a target-place into a place.
    pub fn place_from_target_place(&mut self, place: bir::TargetPlace) -> bir::Place {
        match self.tables[place] {
            bir::TargetPlaceData::LocalVariable(lv) => {
                self.add(bir::PlaceData::LocalVariable(lv), self.origins[place])
            }
            bir::TargetPlaceData::Dot(owner_place, name) => {
                self.add(bir::PlaceData::Dot(owner_place, name), self.origins[place])
            }
        }
    }

    /// Find the loop context for a given loop expression.
    ///
    /// Panics if that loop context has not been pushed.
    pub fn variable(&self, var: validated::LocalVariable) -> bir::LocalVariable {
        self.variables[&var]
    }
}

fn map_variables(
    db: &dyn crate::Db,
    validated_tree: validated::Tree,
    tables: &mut bir::Tables,
    origins: &mut bir::Origins,
) -> Map<validated::LocalVariable, bir::LocalVariable> {
    let validated_data = validated_tree.data(db);
    let validated_tables = &validated_data.tables;
    let validated_origins = validated_tree.origins(db);
    validated_data
        .max_local_variable()
        .iter()
        .map(|validated_var| {
            let validated_var_data = validated_var.data(validated_tables);
            let validated_var_origin = *validated_var.origin_in(validated_origins);
            let bir_var = add(
                tables,
                origins,
                bir::LocalVariableData {
                    name: validated_var_data
                        .name
                        .map(|n| n.data(validated_tables).word),
                    atomic: validated_var_data.atomic,
                },
                validated_var_origin,
            );
            (validated_var, bir_var)
        })
        .collect()
}

fn add<V, O>(
    tables: &mut bir::Tables,
    origins: &mut bir::Origins,
    data: V,
    origin: impl Into<O>,
) -> V::Key
where
    V: dada_id::InternValue<Table = bir::Tables>,
    V::Key: PushOriginIn<bir::Origins, Origin = O>,
{
    let key = tables.add(data);
    origins.push(key, origin.into());
    key
}

impl<'me, K> std::ops::Index<K> for Brewery<'me>
where
    K: dada_id::InternKey<Table = bir::Tables>,
{
    type Output = K::Value;

    fn index(&self, index: K) -> &Self::Output {
        &self.tables[index]
    }
}

impl<'me, K> std::ops::IndexMut<K> for Brewery<'me>
where
    K: dada_id::InternAllocKey<Table = bir::Tables>,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.tables[index]
    }
}
