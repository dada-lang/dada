use dada_ir::word::Word;

use crate::machine::{ObjectData, PermissionData, Value};

use super::{op::MachineOp, Object};

#[extension_trait::extension_trait]
pub(crate) impl<T: ?Sized + MachineOp> DefaultStringify for T {
    /// Converts a given value into a string. This should
    /// eventually be customizable.
    fn stringify_value(&self, db: &dyn crate::Db, value: Value) -> String {
        if let PermissionData::Expired(_) = self[value.permission] {
            "(expired)".to_string()
        } else {
            self.stringify_object(db, value.object)
        }
    }

    // FIXME: There is no way for *users* to write a fn that "inspects" the permission
    // like this. We should maybe just not print them, but it's kind of useful...?
    fn stringify_object(&self, db: &dyn crate::Db, object: Object) -> String {
        tracing::debug!(
            "stringify(object = {:?}, object-data = {:?})",
            object,
            self[object]
        );
        match &self[object] {
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
                self.object_string(db, Some(f.function.name(db)), &f.arguments)
            }
            ObjectData::Instance(i) => self.object_string(db, Some(i.class.name(db)), &i.fields),
            ObjectData::Class(c) => c.name(db).as_str(db).to_string(),
            ObjectData::ThunkRust(r) => format!("{r:?}"),
            ObjectData::Tuple(t) => self.object_string(db, None, &t.fields),
        }
    }

    fn object_string(&self, db: &dyn crate::Db, name: Option<Word>, fields: &[Value]) -> String {
        let mut output = String::new();
        if let Some(name) = name {
            output.push_str(name.as_str(db));
        }
        output.push('(');
        for (field, index) in fields.iter().zip(0..) {
            if index > 0 {
                output.push_str(", ");
            }
            output.push_str(&self.stringify_value(db, *field));
        }
        output.push(')');
        output
    }
}
