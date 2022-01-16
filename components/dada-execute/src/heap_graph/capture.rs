//! Code to capture the state of the interpreter as a HeapGraph.

use dada_id::InternKey;

use crate::{data::Instance, execute::StackFrame, interpreter::Interpreter, value::Value};

use super::{
    DataNodeData, HeapGraph, NamedValueEdge, ObjectNode, ObjectNodeData, StackFrameNodeData,
    ValueEdge,
};

impl HeapGraph {
    ///
    pub fn capture_from(&mut self, interpreter: &Interpreter<'_>, top: &StackFrame<'_>) {
        if let Some(parent) = top.parent_stack_frame {
            self.capture_from(interpreter, parent);
        }

        let span = top.current_span(interpreter);
        let bir_data = top.bir.data(interpreter.db());

        let variables = top
            .local_variables
            .iter_enumerated()
            .map(|(local_variable, value)| {
                let local_variable_data = local_variable.data(&bir_data.tables);
                let value_node = self.value_node(interpreter, value);
                NamedValueEdge {
                    name: local_variable_data.name,
                    value: value_node,
                }
            })
            .collect::<Vec<_>>();

        let data = StackFrameNodeData { span, variables };
        let stack_frame = self.tables.add(data);
        self.stack.push(stack_frame);
    }

    fn value_node(
        &mut self,
        interpreter: &crate::interpreter::Interpreter,
        value: &Value,
    ) -> ValueEdge {
        value.peek(|_permission, data| match data {
            crate::data::Data::Instance(i) => ValueEdge::ToObject(self.object_node(interpreter, i)),
            crate::data::Data::Class(c) => ValueEdge::ToClass(*c),
            crate::data::Data::Function(f) => ValueEdge::ToFunction(*f),
            crate::data::Data::Intrinsic(i) => {
                self.data_edge(interpreter, &i.as_str(interpreter.db()))
            }
            crate::data::Data::Thunk(_thunk) => self.data_edge(interpreter, &"<thunk>"), // FIXME
            crate::data::Data::Tuple(_tuple) => self.data_edge(interpreter, &"<tuple>"), // FIXME
            crate::data::Data::Bool(b) => self.data_edge(interpreter, b),
            crate::data::Data::Uint(v) => self.data_edge(interpreter, v),
            crate::data::Data::Int(i) => self.data_edge(interpreter, i),
            crate::data::Data::Float(f) => self.data_edge(interpreter, f),
            crate::data::Data::String(w) => {
                self.data_edge(interpreter, &w.as_str(interpreter.db()).to_string())
            }
            crate::data::Data::Unit(u) => self.data_edge(interpreter, u),
        })
    }

    fn data_edge(
        &mut self,
        _interpreter: &crate::interpreter::Interpreter,
        d: &(impl std::fmt::Debug + Clone + 'static),
    ) -> ValueEdge {
        let b = DataNodeData {
            debug: Box::new(d.clone()),
        };
        ValueEdge::ToData(self.tables.add(b))
    }

    fn object_node(
        &mut self,
        interpreter: &crate::interpreter::Interpreter,
        instance: &Instance,
    ) -> ObjectNode {
        let fields = instance
            .fields
            .iter()
            .map(|field| self.value_node(interpreter, field))
            .collect::<Vec<_>>();
        self.tables.add(ObjectNodeData {
            class: instance.class,
            fields,
        })
    }
}
