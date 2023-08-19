use std::fmt::Debug;

use dada_collections::IndexVec;
use dada_ir::code::bir;

use super::{
    assert_invariants::AssertInvariants, ExpectedTy, Frame, FrameIndex, Machine, Object,
    ObjectData, Permission, PermissionData, ProgramCounter, ValidPermissionData, Value,
};

pub(crate) trait MachineOp:
    std::ops::IndexMut<Object, Output = ObjectData>
    + std::ops::IndexMut<Permission, Output = PermissionData>
    + std::ops::IndexMut<bir::LocalVariable, Output = Value>
    + std::ops::Index<FrameIndex, Output = Frame>
    + Debug
{
    /// Gives a frozen view onto the state of the machine.
    fn view(&self) -> &Machine;

    fn frames(&self) -> &IndexVec<FrameIndex, Frame>;
    fn push_frame(
        &mut self,
        db: &dyn crate::Db,
        bir: bir::Bir,
        arguments: Vec<Value>,
        expected_return_ty: Option<ExpectedTy>,
    );
    fn clear_frame(&mut self);
    fn pop_frame(&mut self) -> Frame;
    fn top_frame(&self) -> Option<&Frame>;
    fn top_frame_index(&self) -> Option<FrameIndex>;

    fn object(&self, object: Object) -> &ObjectData;
    fn object_mut(&mut self, object: Object) -> &mut ObjectData;
    fn take_object(&mut self, object: Object) -> ObjectData;
    fn new_object(&mut self, data: ObjectData) -> Object;
    fn unit_object(&self) -> Object;
    fn all_objects(&self) -> Vec<Object>;

    fn permission(&self, permission: Permission) -> &PermissionData;
    fn permission_mut(&mut self, permission: Permission) -> &mut PermissionData;
    fn take_permission(&mut self, permission: Permission) -> PermissionData;
    fn new_permission(&mut self, data: ValidPermissionData) -> Permission;
    fn expired_permission(&mut self, origin: Option<ProgramCounter>) -> Permission;
    fn all_permissions(&self) -> Vec<Permission>;

    // Access locals from the top-most stack frame (panics if stack is empty).
    fn local(&self, local_variable: bir::LocalVariable) -> &Value;
    fn local_mut(&mut self, local_variable: bir::LocalVariable) -> &mut Value;

    // Get and set the program counter from the top-most stack frame.
    fn pc(&self) -> ProgramCounter;
    fn set_pc(&mut self, pc: ProgramCounter);

    // Read PC from top-most frame, or None if stack is empty.
    fn opt_pc(&self) -> Option<ProgramCounter>;

    /// Clones the machine into a snapshot of the underlying data.
    /// Used for heapgraphs and introspection.
    fn snapshot(&self) -> Machine;
}

impl MachineOp for Machine {
    fn view(&self) -> &Machine {
        self
    }

    fn frames(&self) -> &IndexVec<FrameIndex, Frame> {
        &self.stack.frames
    }

    fn push_frame(
        &mut self,
        db: &dyn crate::Db,
        bir: bir::Bir,
        arguments: Vec<Value>,
        expected_return_ty: Option<ExpectedTy>,
    ) {
        let bir_data = bir.data(db);

        let expired_permission = self.expired_permission(None);

        // Give each local variable an expired value with no permissions to start.
        let mut locals: IndexVec<bir::LocalVariable, Value> = bir_data
            .max_local_variable()
            .iter()
            .map(|_| Value {
                object: self.unit_object(),
                permission: expired_permission,
            })
            .collect();

        let num_parameters = bir_data.num_parameters;
        assert_eq!(
            num_parameters,
            arguments.len(),
            "wrong number of parameters provided"
        );
        for (local_variable, argument) in
            bir::LocalVariable::range(0, num_parameters).zip(arguments)
        {
            locals[local_variable] = argument;
        }

        self.stack.frames.push(Frame {
            pc: ProgramCounter {
                bir,
                control_point: bir_data.start_point,
            },
            locals,
            expected_return_ty,
        });
    }

    /// Clear the permission from all local variables on the frame.
    #[track_caller]
    fn clear_frame(&mut self) {
        let expired_permission = self.expired_permission(None);
        let top_frame = self.stack.frames.last_mut().unwrap();
        for v in &mut top_frame.locals {
            v.permission = expired_permission;
        }
    }

    #[track_caller]
    fn pop_frame(&mut self) -> Frame {
        self.stack.frames.pop().unwrap()
    }

    fn top_frame(&self) -> Option<&Frame> {
        self.stack.frames.last()
    }

    fn top_frame_index(&self) -> Option<FrameIndex> {
        let l = self.stack.frames.len();
        if l == 0 {
            None
        } else {
            Some(FrameIndex::from(l - 1))
        }
    }

    #[track_caller]
    fn object(&self, object: Object) -> &ObjectData {
        self.heap
            .objects
            .get(object.index)
            .unwrap_or_else(|| panic!("object not found: {object:?}"))
    }

    #[track_caller]
    fn object_mut(&mut self, object: Object) -> &mut ObjectData {
        self.heap
            .objects
            .get_mut(object.index)
            .unwrap_or_else(|| panic!("object not found: {object:?}"))
    }

    #[track_caller]
    fn take_object(&mut self, object: Object) -> ObjectData {
        self.heap
            .objects
            .remove(object.index)
            .unwrap_or_else(|| panic!("object not found: {object:?}"))
    }

    fn new_object(&mut self, data: ObjectData) -> Object {
        if let ObjectData::Unit(()) = data {
            return self.unit_object;
        }
        self.heap.new_object(data)
    }

    fn unit_object(&self) -> Object {
        self.unit_object
    }

    fn all_objects(&self) -> Vec<Object> {
        self.heap.all_objects()
    }

    #[track_caller]
    fn permission(&self, permission: Permission) -> &PermissionData {
        self.heap
            .permissions
            .get(permission.index)
            .unwrap_or_else(|| panic!("object not found: {permission:?}"))
    }

    #[track_caller]
    fn permission_mut(&mut self, permission: Permission) -> &mut PermissionData {
        self.heap.permissions.get_mut(permission.index).unwrap()
    }

    #[track_caller]
    fn take_permission(&mut self, permission: Permission) -> PermissionData {
        self.heap
            .permissions
            .remove(permission.index)
            .unwrap_or_else(|| panic!("permission not found: {permission:?}"))
    }

    fn new_permission(&mut self, data: ValidPermissionData) -> Permission {
        self.heap.new_permission(PermissionData::Valid(data))
    }

    fn all_permissions(&self) -> Vec<Permission> {
        self.heap.all_permissions()
    }

    fn expired_permission(&mut self, place: Option<ProgramCounter>) -> Permission {
        self.heap.new_permission(PermissionData::Expired(place))
    }

    fn local(&self, local_variable: bir::LocalVariable) -> &Value {
        &self.stack.frames.last().unwrap().locals[local_variable]
    }

    fn local_mut(&mut self, local_variable: bir::LocalVariable) -> &mut Value {
        &mut self.stack.frames.last_mut().unwrap().locals[local_variable]
    }

    fn opt_pc(&self) -> Option<ProgramCounter> {
        self.stack.frames.last().map(|f| f.pc)
    }

    fn pc(&self) -> ProgramCounter {
        self.stack.frames.last().unwrap().pc
    }

    fn set_pc(&mut self, pc: ProgramCounter) {
        self.stack.frames.last_mut().unwrap().pc = pc;
    }

    fn snapshot(&self) -> Machine {
        self.clone()
    }
}

impl std::ops::Index<FrameIndex> for Machine {
    type Output = Frame;

    fn index(&self, index: FrameIndex) -> &Self::Output {
        &self.stack.frames[index]
    }
}

impl std::ops::Index<Object> for Machine {
    type Output = ObjectData;

    fn index(&self, index: Object) -> &Self::Output {
        self.object(index)
    }
}

impl std::ops::IndexMut<Object> for Machine {
    fn index_mut(&mut self, index: Object) -> &mut Self::Output {
        self.object_mut(index)
    }
}

impl std::ops::Index<Permission> for Machine {
    type Output = PermissionData;

    fn index(&self, index: Permission) -> &Self::Output {
        self.permission(index)
    }
}

impl std::ops::IndexMut<Permission> for Machine {
    fn index_mut(&mut self, index: Permission) -> &mut Self::Output {
        self.permission_mut(index)
    }
}

impl std::ops::Index<bir::LocalVariable> for Machine {
    type Output = Value;

    fn index(&self, local: bir::LocalVariable) -> &Self::Output {
        self.local(local)
    }
}

impl std::ops::IndexMut<bir::LocalVariable> for Machine {
    fn index_mut(&mut self, local: bir::LocalVariable) -> &mut Self::Output {
        self.local_mut(local)
    }
}

#[extension_trait::extension_trait]
pub(crate) impl MachineOpExtMut for &mut dyn MachineOp {
    fn my_value(&mut self, pc: ProgramCounter, data: impl Into<ObjectData>) -> Value {
        let permission = self.new_permission(ValidPermissionData::my(pc));
        let object = self.new_object(data.into());
        Value { object, permission }
    }

    fn our_value(&mut self, pc: ProgramCounter, data: impl Into<ObjectData>) -> Value {
        let permission = self.new_permission(ValidPermissionData::our(pc));
        let object = self.new_object(data.into());
        Value { object, permission }
    }
}

#[extension_trait::extension_trait]
pub(crate) impl MachineOpExt for &dyn MachineOp {
    fn assert_invariants(self, db: &dyn crate::Db) -> eyre::Result<()> {
        AssertInvariants::new(db, self).assert_all_ok()
    }
}
