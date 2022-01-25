//! The "data graph" is constructed to capture the state of the dada
//! program. It is used in the debugger and so forth.

#![allow(dead_code)] // FIXME

use std::fmt::Debug;

use dada_id::{id, tables};
use dada_ir::{
    class::Class,
    code::bir::{LocalVariable, Place},
    function::Function,
    span::FileSpan,
    word::Word,
};

use crate::{interpreter::Interpreter, StackFrame};

mod capture;
mod graphviz;

pub struct HeapGraph {
    // 0 is the bottom of the stack, length is the top of the stack.
    stack: Vec<StackFrameNode>,
    tables: Tables,
}

impl HeapGraph {
    pub(crate) fn new(
        interpreter: &Interpreter<'_>,
        top: &StackFrame<'_>,
        place: Option<Place>,
    ) -> Self {
        let mut this = Self {
            stack: vec![],
            tables: Default::default(),
        };
        let cache = &mut capture::Cache::default();
        this.capture_from(interpreter, cache, top, place);
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
    class: Class,
    fields: Vec<ValueEdge>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct ValueEdge {
    permission: PermissionNode,
    target: ValueEdgeTarget,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum ValueEdgeTarget {
    Object(ObjectNode),
    Class(Class),
    Function(Function),
    Data(DataNode),
}

id!(pub(crate) struct DataNode);

#[derive(Debug)]
pub(crate) struct DataNodeData {
    debug: Box<dyn Debug + Send + Sync>,
}

id!(pub(crate) struct PermissionNode);

#[derive(Debug)]
pub(crate) struct PermissionNodeData {
    label: PermissionNodeLabel,

    /// If `Some`, then this permission is leased by somebody else.
    tenant: Option<PermissionNode>,

    /// If `Some`, then this permission is leased from the given
    /// permission, which must be a unique (my or leased) permission.
    lessor: Option<PermissionNode>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum PermissionNodeLabel {
    My,
    Our,
    Leased,
    Shared,
    Expired,
}

impl PermissionNodeLabel {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            PermissionNodeLabel::My => "my",
            PermissionNodeLabel::Our => "our",
            PermissionNodeLabel::Leased => "leased",
            PermissionNodeLabel::Shared => "shared",
            PermissionNodeLabel::Expired => "expired",
        }
    }
}
