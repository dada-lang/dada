//! The "data graph" is constructed to capture the state of the dada
//! program. It is used in the debugger and so forth.

#![allow(dead_code)] // FIXME

use std::fmt::Debug;

use dada_id::{id, tables};
use dada_ir::{
    class::Class, code::bir::LocalVariable, function::Function, span::FileSpan, word::Word,
};

use crate::{moment::Moment, StackFrame};

mod capture;
mod graphviz;

pub struct HeapGraph {
    // 0 is the bottom of the stack, length is the top of the stack.
    stack: Vec<StackFrameNode>,
    tables: Tables,
}

impl HeapGraph {
    pub(crate) fn new(db: &dyn crate::Db, top: &StackFrame<'_>) -> Self {
        let mut this = Self {
            stack: vec![],
            tables: Default::default(),
        };
        this.capture_from(db, top);
        this
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
    function: Function,
    variables: Vec<LocalVariableEdge>,
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
    debug: Box<dyn Debug + Send + Sync>,
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
