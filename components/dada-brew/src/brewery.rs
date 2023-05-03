use std::rc::Rc;

use dada_collections::Map;
use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir, syntax,
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
    db: &'me dyn crate::Db,
    input_file: InputFile,
    pub(crate) breakpoints: &'me [syntax::Expr],
    validated_tree_data: &'me validated::TreeData,
    validated_origins: &'me validated::Origins,
    tables: &'me mut bir::Tables,
    origins: &'me mut bir::Origins,
    loop_contexts: Map<validated::Expr, LoopContext>,
    variables: Rc<Map<validated::LocalVariable, bir::LocalVariable>>,
    dummy_terminator: bir::Terminator,

    /// The "temporary stack". This is used to track temporaries that
    /// were created during the brewing process and clear them out
    /// so that we don't artificially extend the lifetime of objects
    /// during interpretation.
    ///
    /// The basic strategy is as follows:
    ///
    /// * Upon starting to brew an expression, we record the length of this
    ///   stack.
    /// * During the brewing process, any new temporary is pushed onto this
    ///   stack.
    /// * When we've finished brewing an expression, we can pop off any temporaries
    ///   pushed during that time and clear their values to nil.
    temporaries: Vec<bir::LocalVariable>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct LoopContext {
    pub continue_block: bir::BasicBlock,
    pub break_block: bir::BasicBlock,
    pub loop_value: bir::TargetPlace,
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
            bir::TerminatorData::Panic,
            *validated_tree_data.root_expr.origin_in(validated_origins),
        );
        Self {
            db,
            input_file,
            breakpoints,
            validated_tree_data,
            validated_origins,
            tables,
            origins,
            loop_contexts: Default::default(),
            variables,
            dummy_terminator,
            temporaries: vec![],
        }
    }

    pub fn expr_is_breakpoint(&self, expr: syntax::Expr) -> Option<usize> {
        self.breakpoints.iter().position(|bp| *bp == expr)
    }

    pub fn validated_tables(&self) -> &'me validated::Tables {
        &self.validated_tree_data.tables
    }

    /// Create a "sub-brewery" that clones the current state
    /// and which shares the same output tables/origins as the
    /// original.
    ///
    /// This is used to brew loops. The idea is that the loop
    /// can mutate owned fields like `loop_contexts` without affecting
    /// the outer brewery. An alternative would be to "pop" the changes
    /// to `loop_contexts`.
    ///
    /// The subbrewery contains a fresh temporary stack; the assumption
    /// is that the subbrewery will be used to brew complete
    /// expressions and hence the stack will just extend
    /// the parent's stack.
    pub fn subbrewery(&mut self) -> Brewery<'_> {
        Brewery {
            db: self.db,
            input_file: self.input_file,
            breakpoints: self.breakpoints,
            validated_tree_data: self.validated_tree_data,
            validated_origins: self.validated_origins,
            tables: self.tables,
            origins: self.origins,
            loop_contexts: self.loop_contexts.clone(),
            variables: self.variables.clone(),
            dummy_terminator: self.dummy_terminator,
            temporaries: vec![],
        }
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

    /// Returns a new basic block with no statements and a "panic" terminator.
    /// This is meant to be used by `cursor`, which should ensure the terminator
    /// is overwritten.
    pub fn dummy_block(&mut self, origin: ExprOrigin) -> bir::BasicBlock {
        self.add(
            bir::BasicBlockData {
                statements: vec![],
                terminator: self.dummy_terminator,
            },
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

    /// Number of temporaries on the "temporary stack".
    ///
    /// See the comments on the `temporaries` field for more information.
    pub fn temporary_stack_len(&self) -> usize {
        self.temporaries.len()
    }

    /// Push a temporary onto the "temporary stack".
    ///
    /// See the comments on the `temporaries` field for more information.
    pub fn push_temporary(&mut self, lv: bir::LocalVariable) {
        tracing::debug!("pushing temporary: {:?}", lv);
        self.temporaries.push(lv);
    }

    /// Pop a temporary from the "temporary stack".
    ///
    /// See the comments on the `temporaries` field for more information.
    #[track_caller]
    pub fn pop_temporary(&mut self) -> bir::LocalVariable {
        self.temporaries.pop().unwrap()
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
