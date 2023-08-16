//! Defines the "abstract machine" that executes a Dada program.

use dada_collections::IndexVec;
use dada_id::id;
use dada_ir::{
    class::Class,
    code::bir,
    function::Function,
    intrinsic::Intrinsic,
    span::FileSpan,
    storage::{Joint, Leased},
};
use generational_arena::Arena;

use crate::thunk::RustThunk;

pub mod assert_invariants;
pub mod op;
pub mod stringify;

/// The abstract machine that executes a Dada program. Stores the state of
/// all values as well as the stack with all the currently executing functions.
///
/// Most parts of the code don't interact with this struct directly.
/// Instead they interact through the [`op::MachineOp`] trait. The idea of introducing
/// this separation was that we may want to allow dynamically swapping in "proxy machines",
/// for example to log or trace each action that is taken at a breakpoint.
#[derive(Clone, Debug)]
pub struct Machine {
    pub heap: Heap,
    pub stack: Stack,

    /// For convenience, store a single unit object,
    pub unit_object: Object,
}

impl Default for Machine {
    fn default() -> Self {
        let mut heap = Heap::default();
        let unit_object = heap.new_object(ObjectData::Unit(()));
        Self {
            heap,
            stack: Default::default(),
            unit_object,
        }
    }
}

/// A value is a reference to an object.
/// It combines the object itself with a permission.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub object: Object,
    pub permission: Permission,
}

#[derive(Clone, Debug, Default)]
pub struct Heap {
    pub objects: Arena<ObjectData>,
    pub permissions: Arena<PermissionData>,
}

impl Heap {
    fn new_object(&mut self, data: ObjectData) -> Object {
        let o = Object {
            index: self.objects.insert(data),
        };

        tracing::debug!("new object: {:?} = {:?}", o, &self.objects[o.index]);

        o
    }

    /// Returns the data for a given object, or `None` if the object
    /// does not exist or has been freed.
    ///
    /// If you know the object exists, prefer to do `machine[object]`.
    pub fn object_data(&self, object: Object) -> Option<&ObjectData> {
        self.objects.get(object.index)
    }

    fn all_objects(&self) -> Vec<Object> {
        let mut vec: Vec<_> = self
            .objects
            .iter()
            .map(|(index, _)| Object { index })
            .collect();
        vec.sort();
        vec
    }

    fn new_permission(&mut self, data: PermissionData) -> Permission {
        let p = Permission {
            index: self.permissions.insert(data),
        };

        tracing::debug!("new permission: {:?} = {:?}", p, &self.permissions[p.index]);

        p
    }

    fn all_permissions(&self) -> Vec<Permission> {
        let mut vec: Vec<_> = self
            .permissions
            .iter()
            .map(|(index, _)| Permission { index })
            .collect();
        vec.sort();
        vec
    }

    /// Returns the data for a given permission, or `None` if the permission
    /// does not exist or has been freed.
    ///
    /// If you know the permission exists, prefer to do `machine[permission]`.
    pub(crate) fn permission_data(&self, permission: Permission) -> Option<&PermissionData> {
        self.permissions.get(permission.index)
    }
}

/// An "object" is a piece of data in the heap.
///
/// It could be one of the primitive object types
/// (like an integer or string) or an instance of
/// a user-provided class.
///
/// This struct is just an index; to get the object's
/// data you combine it with a machine `m` via indexing,
/// like `m[object]`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Object {
    index: generational_arena::Index,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let (a, b) = self.index.into_raw_parts();
        fmt.debug_tuple("Object").field(&a).field(&b).finish()
    }
}

/// The data stored in an object.
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectData {
    /// An instance of a class.
    Instance(Instance),

    /// A reference to a class itself.
    Class(Class),

    /// A reference to a function.
    Function(Function),

    /// A reference to an intrinsic, like `print`.
    Intrinsic(Intrinsic),

    /// The value returned by an `async fn` -- captures the function
    /// that was called along with its arguments. When this value is
    /// awaited, the function is actually pushed onto the stack.
    ThunkFn(ThunkFn),

    /// A thunk defined by Rust code -- wraps a Rust future. Used by
    /// intrinsics.
    ThunkRust(RustThunk),

    /// A tuple of objects like `(a, b, c)`.
    Tuple(Tuple),

    /// Boolean.
    Bool(bool),

    /// Unsigned integer.
    UnsignedInt(u64),

    /// Unsuffixed integer
    Int(u64),

    /// Signed integer.
    SignedInt(i64),

    /// Floating point.
    Float(f64),

    /// String.
    String(String),

    /// Zero-sized unit value.
    Unit(()),
}

impl ObjectData {
    pub fn kind_str(&self, db: &dyn crate::Db) -> String {
        match self {
            ObjectData::Instance(i) => format!("an instance of `{}`", i.class.name(db).as_str(db)),
            ObjectData::Class(_) => "a class".to_string(),
            ObjectData::Function(_) => "a function".to_string(),
            ObjectData::Intrinsic(_) => "a function".to_string(),
            ObjectData::ThunkFn(f) => {
                format!("a suspended call to `{}`", f.function.name(db).as_str(db))
            }
            ObjectData::ThunkRust(_) => "a thunk".to_string(),
            ObjectData::Tuple(_) => "a tuple".to_string(),
            ObjectData::Bool(_) => "a boolean".to_string(),
            ObjectData::UnsignedInt(_) => "an unsigned integer".to_string(),
            ObjectData::Int(_) => "an integer".to_string(),
            ObjectData::SignedInt(_) => "a signed integer".to_string(),
            ObjectData::Float(_) => "a float".to_string(),
            ObjectData::String(_) => "a string".to_string(),
            ObjectData::Unit(()) => "nothing".to_string(),
        }
    }
}

macro_rules! object_data_from_impls {
    ($($variant_name:ident($ty:ty),)*) => {
        $(
            impl From<$ty> for ObjectData {
                fn from(data: $ty) -> Self {
                    ObjectData::$variant_name(data)
                }
            }
        )*
    }
}

object_data_from_impls! {
    Instance(Instance),
    Class(Class),
    Function(Function),
    Intrinsic(Intrinsic),
    ThunkFn(ThunkFn),
    ThunkRust(RustThunk),
    Tuple(Tuple),
    Bool(bool),
    UnsignedInt(u64),
    SignedInt(i64),
    Float(f64),
    String(String),
    Unit(()),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub class: Class,
    pub fields: Vec<Value>,
}

/// When you invoke an async function, the result is
/// a ThunkFn. This stores the arguments that
/// were provided, waiting for an `await` to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThunkFn {
    pub function: Function,
    pub arguments: Vec<Value>,
    pub expected_return_ty: Option<ExpectedTy>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tuple {
    #[allow(dead_code)]
    pub fields: Vec<Value>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Permission {
    index: generational_arena::Index,
}

impl std::fmt::Debug for Permission {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let (a, b) = self.index.into_raw_parts();
        fmt.debug_tuple("Permission").field(&a).field(&b).finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionData {
    /// No permission: if the place is non-none, executing this place is
    /// what caused the permission to be revoked. If None, the permission
    /// was never granted (e.g., uninitialized memory).
    Expired(Option<ProgramCounter>),

    Valid(ValidPermissionData),
}

impl PermissionData {
    pub fn valid(&self) -> Option<&ValidPermissionData> {
        match self {
            PermissionData::Expired(_) => None,
            PermissionData::Valid(v) => Some(v),
        }
    }

    pub fn expired(&self) -> Option<Option<ProgramCounter>> {
        match self {
            PermissionData::Expired(e) => Some(*e),
            PermissionData::Valid(_) => None,
        }
    }

    pub fn joint_if_valid(&self) -> Option<Joint> {
        self.valid().map(|v| v.joint)
    }

    #[track_caller]
    pub fn assert_valid(&self) -> &ValidPermissionData {
        match self {
            PermissionData::Expired(_) => unreachable!(),
            PermissionData::Valid(v) => v,
        }
    }

    #[track_caller]
    pub fn assert_valid_mut(&mut self) -> &mut ValidPermissionData {
        match self {
            PermissionData::Expired(_) => unreachable!(),
            PermissionData::Valid(v) => v,
        }
    }

    #[track_caller]
    pub fn tenants(&self) -> &[Permission] {
        self.valid().map(|v| &v.tenants[..]).unwrap_or(&[])
    }
}

/// The data for a valid permission; each permission
/// is attached to a particular reference from some
/// place (memory location) `p` to some object `o`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidPermissionData {
    /// When the permission was granted.
    pub pc: ProgramCounter,

    /// A *joint* permission indicates whether this particular
    /// place permits other permissions to `o`.
    ///
    /// Note that even if this is false, the place may be
    /// located in a jointly reachable location. See
    /// [`crate::step::traversal::PlaceTraversal`] for more
    /// information.
    pub joint: Joint,

    /// A *leased* permission indicates whether this particular
    /// place owns `o` or is leasing it.
    ///
    /// Note that even if this is false, the permission may be
    /// located in a leased location.
    pub leased: Leased,

    /// An *easement* is another permission that we have given
    /// to sublease some object reachable from us. Example:
    /// leasing a string from a `leased Vec[leased String]`
    /// requires an easement on the vector. This easement is revoked
    /// when the lease is revoked.
    pub easements: Vec<Permission>,

    /// A *tenant* is another permission that we have given
    /// a lease (or sublease, if we ourselves are leased) to
    /// access `o`. This could be a shared
    /// or exclusive lease. Accesses to the fields of `o`
    /// through this permission may cancel the tenants' leases.
    pub tenants: Vec<Permission>,
}

impl ValidPermissionData {
    /// The data for a new "uniquely owned" permission.
    pub fn my(pc: ProgramCounter) -> Self {
        ValidPermissionData {
            pc,
            joint: Joint::No,
            leased: Leased::No,
            tenants: vec![],
            easements: vec![],
        }
    }

    /// The data for a new "jointly owned" permission. Used for literals.
    pub fn our(pc: ProgramCounter) -> Self {
        ValidPermissionData {
            pc,
            joint: Joint::Yes,
            leased: Leased::No,
            tenants: vec![],
            easements: vec![],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match (self.joint, self.leased) {
            (Joint::No, Leased::No) => "my",
            (Joint::No, Leased::Yes) => "leased",
            (Joint::Yes, Leased::No) => "our",
            (Joint::Yes, Leased::Yes) => "shared",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stack {
    pub frames: IndexVec<FrameIndex, Frame>,
}

id!(pub struct FrameIndex);

#[derive(Clone, Debug)]
pub struct Frame {
    pub pc: ProgramCounter,
    pub locals: IndexVec<bir::LocalVariable, Value>,
    pub expected_return_ty: Option<ExpectedTy>,
}

/// Describes a type we expect a value to have.
///
/// This is returned in the `Frame` to describe the
/// expected return type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpectedTy {
    Class(ExpectedClassTy),
    Error,
}

/// Describes the expected type we expect an instance to have.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpectedClassTy {
    /// Permissions used to access the object.
    pub permission: ExpectedPermission,

    /// Class of the object (e.g., `String`).
    pub class: Class,

    /// Generic parameters (if any) to the class.
    pub generics: Vec<ExpectedTy>,
}

/// Describes the expected permission on a returned object.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpectedPermission {
    pub kind: ExpectedPermissionKind,
    pub declared_permissions: Vec<Permission>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpectedPermissionKind {
    Member,
    Given,
    Leased,
    Shared,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProgramCounter {
    /// The BIR we are interpreting.
    pub bir: bir::Bir,

    /// The current basic block.
    pub basic_block: bir::BasicBlock,

    /// The index of the statement to execute next within the
    /// basic block, or -- if equal to the number of statements -- indicates
    /// we are about to execute the terminator.
    pub statement: usize,
}

impl ProgramCounter {
    pub fn move_to_block(self, basic_block: bir::BasicBlock) -> ProgramCounter {
        Self::at_block(self.bir, basic_block)
    }

    pub fn at_block(bir: bir::Bir, basic_block: bir::BasicBlock) -> ProgramCounter {
        Self {
            bir,
            basic_block,
            statement: 0,
        }
    }

    /// True if this PC represents a `return` terminator.
    pub fn is_return(&self, db: &dyn crate::Db) -> bool {
        let bir_data = self.bir.data(db);
        let basic_block_data = &bir_data.tables[self.basic_block];
        if self.statement < basic_block_data.statements.len() {
            return false;
        }

        let data = &bir_data.tables[basic_block_data.terminator];

        matches!(data, bir::TerminatorData::Return(_))
    }

    pub fn span(&self, db: &dyn crate::Db) -> FileSpan {
        // FIXME: This code is copied/adapter from Stepper::span_from_bir,
        // it seems like we could create some helper functions, maybe on the
        // Bir type itself.

        let bir_data = self.bir.data(db);
        let basic_block_data = &bir_data.tables[self.basic_block];
        let origins = self.bir.origins(db);
        let syntax_expr = if self.statement < basic_block_data.statements.len() {
            origins[basic_block_data.statements[self.statement]]
        } else {
            origins[basic_block_data.terminator]
        };

        self.bir.span_of(db, syntax_expr)
    }
}
