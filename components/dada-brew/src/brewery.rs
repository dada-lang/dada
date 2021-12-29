use std::rc::Rc;

use dada_collections::Map;
use dada_id::InternKey;
use dada_ir::{
    code::{bir, syntax, validated},
    origin_table::{HasOriginIn, PushOriginIn},
};

pub(crate) struct Brewery<'me> {
    db: &'me dyn crate::Db,
    validated_tree_data: &'me validated::TreeData,
    validated_origins: &'me validated::Origins,
    tables: &'me mut bir::Tables,
    origins: &'me mut bir::Origins,
    loop_contexts: Map<validated::Expr, LoopContext>,
    variables: Rc<Map<validated::LocalVariable, bir::LocalVariable>>,
    dummy_terminator: bir::Terminator,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Hash)]
pub(crate) struct LoopContext {
    pub(crate) continue_block: bir::BasicBlock,
    pub(crate) break_block: bir::BasicBlock,
    pub(crate) loop_value: bir::Place,
}

impl<'me> Brewery<'me> {
    pub fn new(
        db: &'me dyn crate::Db,
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
            bir::TerminatorData::Panic,
            *validated_tree_data.root_expr.origin_in(validated_origins),
        );
        Self {
            db,
            validated_tree_data,
            validated_origins,
            tables,
            origins,
            loop_contexts: Default::default(),
            variables,
            dummy_terminator,
        }
    }

    pub fn validated_tables(&self) -> &'me validated::Tables {
        &self.validated_tree_data.tables
    }

    /// Create a "sub-brewery" that has the same output
    /// tables but independent loop contexts and other
    /// scoped information.
    pub fn subbrewery(&mut self) -> Brewery<'_> {
        Brewery {
            db: self.db,
            validated_tree_data: self.validated_tree_data,
            validated_origins: self.validated_origins,
            tables: self.tables,
            origins: self.origins,
            loop_contexts: self.loop_contexts.clone(),
            variables: self.variables.clone(),
            dummy_terminator: self.dummy_terminator,
        }
    }

    pub fn origin<K>(&self, of: K) -> K::Origin
    where
        K: HasOriginIn<validated::Origins>,
    {
        of.origin_in(self.validated_origins).clone()
    }

    /// Returns a new basic block with no statements and a "panic" terminator.
    /// This is meant to be used by `cursor`, which should ensure the terminator
    /// is overwritten.
    pub fn dummy_block(&mut self, origin: syntax::Expr) -> bir::BasicBlock {
        self.add(
            bir::BasicBlockData {
                statements: vec![],
                terminator: self.dummy_terminator,
            },
            origin,
        )
    }

    pub fn add<V, O>(&mut self, data: V, origin: O) -> V::Key
    where
        V: dada_id::InternValue<Table = bir::Tables>,
        V::Key: PushOriginIn<bir::Origins, Origin = O>,
    {
        add(self.tables, self.origins, data, origin)
    }

    /// Find the loop context for a given loop expression.
    ///
    /// Panics if that loop context has not been pushed.
    pub fn loop_context(&self, loop_expr: validated::Expr) -> LoopContext {
        self.loop_contexts[&loop_expr]
    }

    /// Push a new loop context into the brewery; typically this is done in a "subbrewery".
    pub fn push_loop_context(&mut self, loop_expr: validated::Expr, loop_context: LoopContext) {
        let old_value = self.loop_contexts.insert(loop_expr, loop_context);
        assert!(old_value.is_none());
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
                    name: validated_var_data.name,
                    storage_mode: validated_var_data.storage_mode,
                },
                validated_var_origin,
            );
            (validated_var, bir_var)
        })
        .collect()
}

fn add<V, O>(tables: &mut bir::Tables, origins: &mut bir::Origins, data: V, origin: O) -> V::Key
where
    V: dada_id::InternValue<Table = bir::Tables>,
    V::Key: PushOriginIn<bir::Origins, Origin = O>,
{
    let key = tables.add(data);
    origins.push(key, origin);
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
