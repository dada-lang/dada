//! The "data graph" is constructed to capture the state of the dada
//! program. It is used in the debugger and so forth.

#![allow(dead_code)] // FIXME

use std::fmt::Debug;

use dada_id::{id, tables};
use dada_ir::{class::Class, function::Function, span::FileSpan, word::Word};

use crate::moment::Moment;

mod capture;

pub(crate) struct HeapGraph {
    // 0 is the bottom of the stack, length is the top of the stack.
    stack: Vec<StackFrameNode>,
    tables: Tables,
}

impl std::ops::Deref for HeapGraph {
    type Target = Tables;

    fn deref(&self) -> &Tables {
        &self.tables
    }
}

tables! {
    #[derive(Debug)]
    pub(crate) struct Tables {
        stack_frame_nodes: alloc StackFrameNode => StackFrameNodeData,
        objects: alloc ObjectNode => ObjectNodeData,
        datas: alloc DataNode => DataNodeData,
    }
}

id!(pub(crate) struct StackFrameNode);

#[derive(Debug)]
pub(crate) struct StackFrameNodeData {
    span: FileSpan,
    variables: Vec<NamedValueEdge>,
}

#[derive(Debug)]
pub(crate) struct NamedValueEdge {
    pub(crate) name: Option<Word>,
    pub(crate) value: ValueEdge,
}

id!(pub(crate) struct ObjectNode);

#[derive(Debug)]
pub(crate) struct ObjectNodeData {
    class: Class,
    fields: Vec<ValueEdge>,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ValueEdge {
    ToObject(ObjectNode),
    ToClass(Class),
    ToFunction(Function),
    ToData(DataNode),
}

id!(pub(crate) struct DataNode);

#[derive(Debug)]
pub(crate) struct DataNodeData {
    debug: Box<dyn Debug>,
}

id!(pub(crate) struct PermissionNode);

#[derive(Debug)]
pub(crate) enum PermissionNodeData {
    My {
        granted: Moment,
    },

    Our {
        granted: Moment,
    },

    Leased {
        granted: Moment,
        lessor: PermissionNode,
    },

    Shared {
        granted: Moment,
        lessor: PermissionNode,
    },

    Expired {
        canceled: Moment,
    },
}
