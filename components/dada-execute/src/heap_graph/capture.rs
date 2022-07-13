//! Code to capture the state of the db as a HeapGraph.

use dada_collections::Map;
use dada_id::InternKey;
use dada_ir::storage::{Joint, Leased};

use crate::machine::{
    op::MachineOp, op::MachineOpExt, stringify::DefaultStringify, Frame, Object, ObjectData,
    Permission, PermissionData, Reservation, Value,
};

use super::{
    DataNodeData, HeapGraph, LocalVariableEdge, ObjectNode, ObjectNodeData, ObjectType,
    PermissionNode, PermissionNodeData, PermissionNodeLabel, PermissionNodeSource,
    StackFrameNodeData, ValueEdge, ValueEdgeData, ValueEdgeTarget,
};

pub(super) struct HeapGraphCapture<'me> {
    db: &'me dyn crate::Db,
    graph: &'me mut HeapGraph,
    machine: &'me dyn MachineOp,
    instances: Map<Object, ObjectNode>,
    permissions: Map<Permission, PermissionNode>,
    reservations: Map<Reservation, ObjectNode>,
}

impl<'me> HeapGraphCapture<'me> {
    pub(super) fn new(
        db: &'me dyn crate::Db,
        graph: &'me mut HeapGraph,
        machine: &'me dyn MachineOp,
    ) -> Self {
        Self {
            db,
            graph,
            machine,
            instances: Default::default(),
            permissions: Default::default(),
            reservations: Default::default(),
        }
    }

    pub(super) fn capture(mut self, in_flight_value: Option<Value>) {
        let Some((top, others)) = self.machine.frames().split_last() else {
            return;
        };

        for frame in others {
            self.push_frame(frame, None);
        }

        self.push_frame(top, in_flight_value);
    }

    fn push_frame(&mut self, frame: &Frame, in_flight_value: Option<Value>) {
        let span = frame.pc.span(self.db);
        let bir_data = frame.pc.bir.data(self.db);

        let variables = frame
            .locals
            .iter_enumerated()
            .map(|(local_variable, &value)| {
                let local_variable_data = local_variable.data(&bir_data.tables);
                let value_node = self.value_edge(value);
                LocalVariableEdge {
                    id: local_variable,
                    name: local_variable_data.name,
                    value: value_node,
                }
            })
            .collect::<Vec<_>>();

        let in_flight_value = in_flight_value.map(|p| self.value_edge(p));

        let data = StackFrameNodeData {
            function_name: frame.pc.bir.function_name(self.db),
            span,
            variables,
            in_flight_value,
        };
        let stack_frame = self.graph.tables.add(data);
        self.graph.stack.push(stack_frame);
    }

    fn value_edge(&mut self, value: Value) -> ValueEdge {
        let permission = self.permission_node(value.permission);

        let target = match &self.machine[value.permission] {
            PermissionData::Expired(_) => ValueEdgeTarget::Expired,
            PermissionData::Valid(_) => self.value_edge_target(value.object),
        };

        self.graph.tables.add(ValueEdgeData { permission, target })
    }

    fn value_edge_target(&mut self, object: Object) -> ValueEdgeTarget {
        let db = self.db;
        match &self.machine[object] {
            ObjectData::Instance(instance) => ValueEdgeTarget::Object(self.instance_node(
                object,
                ObjectType::Class(instance.class),
                &instance.fields,
            )),
            ObjectData::ThunkFn(thunk) => ValueEdgeTarget::Object(self.instance_node(
                object,
                ObjectType::Thunk(thunk.function),
                &thunk.arguments,
            )),
            ObjectData::ThunkRust(thunk) => ValueEdgeTarget::Object(self.instance_node(
                object,
                ObjectType::RustThunk(thunk.description),
                &thunk.arguments,
            )),
            ObjectData::Tuple(_tuple) => self.data_target(db, object, &"<tuple>"), // FIXME
            ObjectData::Reservation(reservation) => {
                ValueEdgeTarget::Object(self.reservation_node(object, *reservation))
            }
            ObjectData::Class(c) => ValueEdgeTarget::Class(*c),
            ObjectData::Function(f) => ValueEdgeTarget::Function(*f),
            ObjectData::Intrinsic(_)
            | ObjectData::Bool(_)
            | ObjectData::UnsignedInt(_)
            | ObjectData::Int(_)
            | ObjectData::SignedInt(_)
            | ObjectData::Float(_)
            | ObjectData::String(_)
            | ObjectData::Unit(_) => {
                let string = DefaultStringify::stringify_object(self.machine, self.db, "", object);
                self.data_target(db, object, &string)
            }
        }
    }

    fn data_target(
        &mut self,
        _db: &dyn crate::Db,
        object: Object,
        d: &(impl std::fmt::Debug + Send + Sync + Clone + 'static),
    ) -> ValueEdgeTarget {
        let b = DataNodeData {
            object,
            debug: Box::new(d.clone()),
        };
        ValueEdgeTarget::Data(self.graph.tables.add(b))
    }

    fn permission_node(&mut self, permission: Permission) -> PermissionNode {
        // Watch out for cycles, but also cache.
        if let Some(n) = self.permissions.get(&permission) {
            return *n;
        }

        let data = &self.machine[permission];

        let label = match data {
            PermissionData::Expired(_) => PermissionNodeLabel::Expired,
            PermissionData::Valid(valid) => match (valid.joint, valid.leased) {
                (Joint::No, Leased::No) => PermissionNodeLabel::My,
                (Joint::Yes, Leased::No) => PermissionNodeLabel::Our,
                (Joint::No, Leased::Yes) => PermissionNodeLabel::Leased,
                (Joint::Yes, Leased::Yes) => PermissionNodeLabel::Shleased,
            },
        };

        let node = self.graph.tables.add(PermissionNodeData {
            source: PermissionNodeSource::Permission(permission),
            label,
            lessor: None,
            tenants: vec![],
        });

        self.permissions.insert(permission, node);

        for &tenant in data.tenants() {
            let tenant_node = self.permission_node(tenant);
            self.graph.tables[node].tenants.push(tenant_node);
            assert!(self.graph.tables[tenant_node].lessor.is_none());
            self.graph.tables[tenant_node].lessor = Some(node);
        }

        node
    }

    fn instance_node(
        &mut self,
        object: Object,
        ty: ObjectType,
        field_values: &[Value],
    ) -> ObjectNode {
        // Detect cycles and prevent redundant work.
        if let Some(n) = self.instances.get(&object) {
            return *n;
        }

        let node = self.graph.tables.add(ObjectNodeData {
            object,
            ty,
            fields: Default::default(),
        });

        // Insert this into the cache lest evaluating a field leads back here!
        self.instances.insert(object, node);

        let fields = field_values
            .iter()
            .map(|&field| self.value_edge(field))
            .collect::<Vec<_>>();

        self.graph.tables[node].fields = fields;

        node
    }

    fn reservation_node(&mut self, object: Object, reservation: Reservation) -> ObjectNode {
        // Detect cycles and prevent redundant work.
        if let Some(n) = self.reservations.get(&reservation) {
            return *n;
        }

        let node = self.graph.tables.add(ObjectNodeData {
            object,
            ty: ObjectType::Reservation,
            fields: Default::default(),
        });

        self.reservations.insert(reservation, node);

        let mut fields = vec![];
        match self.machine.peek_reservation(self.db, reservation) {
            Ok(object) => {
                let permission = self.graph.tables.add(PermissionNodeData {
                    source: PermissionNodeSource::Reservation(reservation),
                    label: PermissionNodeLabel::Reserved,
                    tenants: vec![],
                    lessor: None,
                });
                let target = self.value_edge_target(object);
                fields.push(self.graph.tables.add(ValueEdgeData { permission, target }));
            }

            Err(_err) => { /* should not happen, just ignore I guess */ }
        }
        self.graph.tables[node].fields = fields;

        node
    }
}
