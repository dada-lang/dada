use dada_ir::{
    storage_mode::{Joint, Leased},
    word::Word,
};

use crate::machine::{ObjectData, Permission, PermissionData, Value};

use super::op::MachineOp;

#[extension_trait::extension_trait]
pub(crate) impl<T: ?Sized + MachineOp> DefaultStringify for T {
    /// Converts a given value into a string. This should
    /// eventually be customizable.
    fn stringify(&self, db: &dyn crate::Db, value: Value) -> String {
        let Some(p) = self.permission_str(value.permission) else {
            return "(expired)".to_string();
        };

        match &self[value.object] {
            ObjectData::String(s) => s.to_string(),
            ObjectData::Bool(v) => format!("{}", v),
            ObjectData::SignedInt(v) => format!("{}_i", v),
            ObjectData::Float(v) => format!("{}", v),
            ObjectData::UnsignedInt(v) => format!("{}_u", v),
            ObjectData::Int(v) => format!("{}", v),
            ObjectData::Unit(_) => "()".to_string(),
            ObjectData::Intrinsic(i) => i.as_str(db).to_string(),
            ObjectData::Function(f) => f.name(db).as_str(db).to_string(),
            ObjectData::ThunkFn(f) => {
                self.object_string(db, p, Some(f.function.name(db).word(db)), &f.arguments)
            }
            ObjectData::Instance(i) => {
                self.object_string(db, p, Some(i.class.name(db).word(db)), &i.fields)
            }
            ObjectData::Class(c) => c.name(db).as_str(db).to_string(),
            ObjectData::ThunkRust(r) => format!("{p} {r:?}"),
            ObjectData::Tuple(t) => self.object_string(db, p, None, &t.fields),
        }
    }

    fn object_string(
        &self,
        db: &dyn crate::Db,
        permission: &str,
        name: Option<Word>,
        fields: &[Value],
    ) -> String {
        let mut output = String::new();
        output.push_str(permission);
        if let Some(name) = name {
            if !permission.is_empty() {
                output.push(' ');
            }
            output.push_str(name.as_str(db));
        }
        output.push('(');
        for (field, index) in fields.iter().zip(0..) {
            if index > 0 {
                output.push_str(", ");
            }
            output.push_str(&self.stringify(db, *field));
        }
        output.push(')');
        output
    }

    fn permission_str(&self, permission: Permission) -> Option<&str> {
        match &self[permission] {
            PermissionData::Expired(_) => None,
            PermissionData::Valid(valid) => Some(match (valid.joint, valid.leased) {
                (Joint::No, Leased::No) => "my",
                (Joint::Yes, Leased::No) => "our",
                (Joint::No, Leased::Yes) => "leased",
                (Joint::Yes, Leased::Yes) => "our leased",
            }),
        }
    }
}
