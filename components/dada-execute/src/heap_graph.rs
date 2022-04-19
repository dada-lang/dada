//! The "data graph" is constructed to capture the state of the dada
//! program. It is used in the debugger and so forth.

#![allow(dead_code)] // FIXME

use std::fmt::Debug;

use dada_id::{id, tables};
use dada_ir::{
    class::Class, code::bir::LocalVariable, function::Function, span::FileSpan, word::Word,
};

use crate::machine::{op::MachineOp, Machine, Object, Permission, Reservation, Value};

mod capture;
mod graphviz;

pub struct HeapGraph {
    /// Snapshot of the machine that this is a graph of
    ///
    /// FIXME: it's not obvious that we need the other fields of this struct.
    /// We could generate the graphviz directly from this.
    machine: Machine,

    /// 0 is the bottom of the stack, length is the top of the stack.
    stack: Vec<StackFrameNode>,

    /// Stores the data for the various bits of graphviz.
    tables: Tables,
}

impl HeapGraph {
    pub(crate) fn new(
        db: &dyn crate::Db,
        machine: &dyn MachineOp,
        in_flight_value: Option<Value>,
    ) -> Self {
        let mut this = Self {
            machine: machine.snapshot(),
            stack: vec![],
            tables: Default::default(),
        };
        let capture = capture::HeapGraphCapture::new(db, &mut this, machine);
        capture.capture(in_flight_value);
        this
    }
}

tables! {
    #[derive(Debug)]
    pub(crate) struct Tables {
        stack_frame_nodes: alloc StackFrameNode => StackFrameNodeData,
        objects: alloc ObjectNode => ObjectNodeData,
        datas: alloc DataNode => DataNodeData,
        permissions: alloc PermissionNode => PermissionNodeData,
        value_edges: alloc ValueEdge => ValueEdgeData,
    }
}

id!(pub(crate) struct StackFrameNode);

#[derive(Debug)]
pub(crate) struct StackFrameNodeData {
    span: FileSpan,
    function: Function,
    variables: Vec<LocalVariableEdge>,
    in_flight_value: Option<ValueEdge>,
}

#[derive(Debug)]
pub(crate) struct LocalVariableEdge {
    pub(crate) id: LocalVariable,
    pub(crate) name: Option<Word>,
    pub(crate) value: ValueEdge,
}

id!(pub(crate) struct ObjectNode);

#[derive(Debug)]
pub(crate) struct ObjectNodeData {
    object: Object,
    ty: ObjectType,
    fields: Vec<ValueEdge>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum ObjectType {
    Class(Class),
    Thunk(Function),
    RustThunk(&'static str),
    Reservation,
}

id!(pub(crate) struct ValueEdge);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct ValueEdgeData {
    permission: PermissionNode,
    target: ValueEdgeTarget,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum ValueEdgeTarget {
    Object(ObjectNode),
    Class(Class),
    Function(Function),
    Data(DataNode),
    Expired,
}

id!(pub(crate) struct DataNode);

#[derive(Debug)]
pub(crate) struct DataNodeData {
    object: Object,
    debug: Box<dyn Debug + Send + Sync>,
}

id!(pub(crate) struct PermissionNode);

#[derive(Debug)]
pub(crate) struct PermissionNodeData {
    source: PermissionNodeSource,

    label: PermissionNodeLabel,

    /// If non-empty, then this permission is leased by somebody else.
    tenants: Vec<PermissionNode>,

    /// If `Some`, then this permission is leased from the given
    /// permission, which must be a unique (my or leased) permission.
    lessor: Option<PermissionNode>,
}

#[derive(Debug)]
pub(crate) enum PermissionNodeSource {
    Permission(Permission),
    Reservation(Reservation),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum PermissionNodeLabel {
    My,
    Our,
    Leased,
    Shleased,
    Reserved,
    Expired,
}

impl PermissionNodeLabel {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            PermissionNodeLabel::My => "my",
            PermissionNodeLabel::Our => "our",
            PermissionNodeLabel::Leased => "leased",
            PermissionNodeLabel::Shleased => "shleased",
            PermissionNodeLabel::Reserved => "reserved",
            PermissionNodeLabel::Expired => "expired",
        }
    }
}
