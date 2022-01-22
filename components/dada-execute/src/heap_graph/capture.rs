//! Code to capture the state of the db as a HeapGraph.

use dada_id::InternKey;

use crate::{data::Instance, execute::StackFrame, value::Value};

use super::{
    DataNodeData, HeapGraph, LocalVariableEdge, ObjectNode, ObjectNodeData, StackFrameNodeData,
    ValueEdge,
};

impl HeapGraph {
    ///
    pub(super) fn capture_from(&mut self, db: &dyn crate::Db, top: &StackFrame<'_>) {
        if let Some(parent) = top.parent_stack_frame {
            self.capture_from(db, parent);
        }

        let span = top.current_span(db);
        let bir_data = top.bir.data(db);

        let variables = top
            .local_variables
            .iter_enumerated()
            .map(|(local_variable, value)| {
                let local_variable_data = local_variable.data(&bir_data.tables);
                let value_node = self.value_node(db, value);
                LocalVariableEdge {
                    id: local_variable,
                    name: local_variable_data.name,
                    value: value_node,
                }
            })
            .collect::<Vec<_>>();

        let data = StackFrameNodeData {
            function: top.function,
            span,
            variables,
        };
        let stack_frame = self.tables.add(data);
        self.stack.push(stack_frame);
    }

    fn value_node(&mut self, db: &dyn crate::Db, value: &Value) -> ValueEdge {
        value.peek(|_permission, data| match data {
            crate::data::Data::Instance(i) => ValueEdge::ToObject(self.object_node(db, i)),
            crate::data::Data::Class(c) => ValueEdge::ToClass(*c),
            crate::data::Data::Function(f) => ValueEdge::ToFunction(*f),
            crate::data::Data::Intrinsic(i) => self.data_edge(db, &i.as_str(db)),
            crate::data::Data::Thunk(_thunk) => self.data_edge(db, &"<thunk>"), // FIXME
            crate::data::Data::Tuple(_tuple) => self.data_edge(db, &"<tuple>"), // FIXME
            crate::data::Data::Bool(b) => self.data_edge(db, b),
            crate::data::Data::Uint(v) => self.data_edge(db, v),
            crate::data::Data::Int(i) => self.data_edge(db, i),
            crate::data::Data::Float(f) => self.data_edge(db, f),
            crate::data::Data::String(w) => self.data_edge(db, &w.as_str(db).to_string()),
            crate::data::Data::Unit(u) => self.data_edge(db, u),
        })
    }

    fn data_edge(
        &mut self,
        _db: &dyn crate::Db,
        d: &(impl std::fmt::Debug + Send + Sync + Clone + 'static),
    ) -> ValueEdge {
        let b = DataNodeData {
            debug: Box::new(d.clone()),
        };
        ValueEdge::ToData(self.tables.add(b))
    }

    fn object_node(&mut self, db: &dyn crate::Db, instance: &Instance) -> ObjectNode {
        let fields = instance
            .fields
            .iter()
            .map(|field| self.value_node(db, field))
            .collect::<Vec<_>>();
        self.tables.add(ObjectNodeData {
            class: instance.class,
            fields,
        })
    }
}
