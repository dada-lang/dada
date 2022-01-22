//! Code to capture the state of the db as a HeapGraph.

use dada_id::InternKey;
use dada_ir::code::bir::Place;

use crate::{data::Instance, execute::StackFrame, interpreter::Interpreter, value::Value};

use super::{
    DataNodeData, HeapGraph, LocalVariableEdge, ObjectNode, ObjectNodeData, StackFrameNodeData,
    ValueEdge,
};

impl HeapGraph {
    ///
    pub(super) fn capture_from(
        &mut self,
        interpreter: &Interpreter<'_>,
        top: &StackFrame<'_>,
        in_flight_value: Option<Place>,
    ) {
        if let Some(parent) = top.parent_stack_frame {
            self.capture_from(interpreter, parent, None);
        }

        let db = interpreter.db();
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

        let in_flight_value = in_flight_value.map(|p| {
            top.with_place(interpreter, p, |value, _| Ok(self.value_node(db, value)))
                .unwrap()
        });

        let data = StackFrameNodeData {
            function: top.function,
            span,
            variables,
            in_flight_value,
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
