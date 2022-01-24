//! Code to capture the state of the db as a HeapGraph.

use dada_collections::Map;
use dada_id::InternKey;
use dada_ir::code::bir::Place;

use crate::{
    data::Instance,
    execute::StackFrame,
    interpreter::Interpreter,
    permission::{Permission, PermissionData},
    value::Value,
};

use super::{
    DataNodeData, HeapGraph, LocalVariableEdge, ObjectNode, ObjectNodeData, PermissionNode,
    PermissionNodeData, PermissionNodeLabel, StackFrameNodeData, ValueEdge, ValueEdgeTarget,
};

#[derive(Default)]
pub(super) struct Cache {
    instances: Map<*const Instance, ObjectNode>,
    permissions: Map<*const PermissionData, PermissionNode>,
}

impl HeapGraph {
    ///
    pub(super) fn capture_from(
        &mut self,
        interpreter: &Interpreter<'_>,
        cache: &mut Cache,
        top: &StackFrame<'_>,
        in_flight_value: Option<Place>,
    ) {
        if let Some(parent) = top.parent_stack_frame {
            self.capture_from(interpreter, cache, parent, None);
        }

        let db = interpreter.db();
        let span = top.current_span(db);
        let bir_data = top.bir.data(db);

        let variables = top
            .local_variables
            .iter_enumerated()
            .map(|(local_variable, value)| {
                let local_variable_data = local_variable.data(&bir_data.tables);
                let value_node = self.value_node(cache, db, value);
                LocalVariableEdge {
                    id: local_variable,
                    name: local_variable_data.name,
                    value: value_node,
                }
            })
            .collect::<Vec<_>>();

        let in_flight_value = in_flight_value.map(|p| {
            top.with_place(interpreter, p, |value, _| {
                Ok(self.value_node(cache, db, value))
            })
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

    fn value_node(&mut self, cache: &mut Cache, db: &dyn crate::Db, value: &Value) -> ValueEdge {
        value.peek(|permission, data| {
            let permission = self.permission_node(cache, db, permission);
            let target = match data {
                crate::data::Data::Instance(i) => {
                    ValueEdgeTarget::Object(self.object_node(cache, db, i))
                }
                crate::data::Data::Class(c) => ValueEdgeTarget::Class(*c),
                crate::data::Data::Function(f) => ValueEdgeTarget::Function(*f),
                crate::data::Data::Intrinsic(i) => self.data_target(db, &i.as_str(db)),
                crate::data::Data::Thunk(_thunk) => self.data_target(db, &"<thunk>"), // FIXME
                crate::data::Data::Tuple(_tuple) => self.data_target(db, &"<tuple>"), // FIXME
                crate::data::Data::Bool(b) => self.data_target(db, b),
                crate::data::Data::Uint(v) => self.data_target(db, v),
                crate::data::Data::Int(i) => self.data_target(db, i),
                crate::data::Data::Float(f) => self.data_target(db, f),
                crate::data::Data::String(w) => self.data_target(db, &w.as_str(db).to_string()),
                crate::data::Data::Unit(u) => self.data_target(db, u),
            };
            ValueEdge { permission, target }
        })
    }

    fn data_target(
        &mut self,
        _db: &dyn crate::Db,
        d: &(impl std::fmt::Debug + Send + Sync + Clone + 'static),
    ) -> ValueEdgeTarget {
        let b = DataNodeData {
            debug: Box::new(d.clone()),
        };
        ValueEdgeTarget::Data(self.tables.add(b))
    }

    fn permission_node(
        &mut self,
        cache: &mut Cache,
        db: &dyn crate::Db,
        permission: &Permission,
    ) -> PermissionNode {
        // The permision-data values have a unique location in memory, so
        // use that to detect cycles and the like.
        let data_ptr: *const PermissionData = permission.peek_data();
        if let Some(n) = cache.permissions.get(&data_ptr) {
            return *n;
        }

        let label = if permission.is_valid() {
            match permission.peek_data() {
                PermissionData::My(_) => PermissionNodeLabel::My,
                PermissionData::Leased(_) => PermissionNodeLabel::Leased,
                PermissionData::Our(_) => PermissionNodeLabel::Our,
                PermissionData::Shared(_) => PermissionNodeLabel::Shared,
            }
        } else {
            PermissionNodeLabel::Expired
        };

        let node = self.tables.add(PermissionNodeData {
            label,
            lessor: None,
        });

        cache.permissions.insert(data_ptr, node);

        if let Some(tenant) = permission.peek_data().peek_tenant() {
            assert!(permission.is_valid());
            let tenant_node = self.permission_node(cache, db, &tenant);
            self.tables[tenant_node].lessor = Some(node);
        }

        node
    }

    fn object_node(
        &mut self,
        cache: &mut Cache,
        db: &dyn crate::Db,
        instance: &Instance,
    ) -> ObjectNode {
        // The `instance` values have a unique location in memory, so
        // use that to detect cycles and the like.
        let instance_ptr: *const Instance = instance;
        if let Some(n) = cache.instances.get(&instance_ptr) {
            return *n;
        }

        let node = self.tables.add(ObjectNodeData {
            class: instance.class,
            fields: Default::default(),
        });

        // Insert this into the cache lest evaluating a field leads back here!
        cache.instances.insert(instance_ptr, node);

        let fields = instance
            .fields
            .iter()
            .map(|field| self.value_node(cache, db, field))
            .collect::<Vec<_>>();

        self.tables[node].fields = fields;

        node
    }
}
