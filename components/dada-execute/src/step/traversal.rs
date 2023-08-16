use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir::{self, LocalVariable},
        syntax,
    },
    error,
    origin_table::HasOriginIn,
    span::FileSpan,
    storage::{Atomic, Joint, Leased},
    word::Word,
};
use dada_validate::prelude::*;

use crate::{
    error::DiagnosticBuilderExt,
    ext::DadaExecuteClassExt,
    machine::{
        op::MachineOpExtMut, Object, ObjectData, Permission, PermissionData, ProgramCounter, Value,
    },
};

use super::{address::Address, Stepper};

/// Permissions accumulated along a traversal.
#[derive(Debug)]
pub(super) struct AccumulatedPermissions {
    /// Every permission that was traversed
    /// to reach the given destination
    /// (either a place or an object).
    pub(super) traversed: Vec<Permission>,

    /// Did we traverse through a leased permission?
    ///
    /// If `Leased::No`, the data is owned.
    pub(super) leased: Leased,

    /// Did we traversed through joint storage or a
    /// joint permission?
    ///
    /// If `Joint::No`, the data is uniquely reached through this path.
    pub(super) joint: Joint,

    /// Did we pass through atomic storage?
    pub(super) atomic: Atomic,
}

impl AccumulatedPermissions {
    /// Returns the permissions one would have when accessing
    /// a uniquely owned location with the given "atomic"-ness.
    pub fn unique(atomic: Atomic) -> Self {
        AccumulatedPermissions {
            traversed: vec![],
            leased: Leased::No,
            joint: Joint::No,
            atomic,
        }
    }
}

/// A traversal to a place (i.e., a traversal that terminates
/// in the location of a value). This is in contrast
/// to `ObjectTraversal`, which represents a traversal
/// that takes the additional step to reach an object.
///
/// To see the difference, think of this example
/// ([source](https://is.gd/c7o2zB)). In this diagram,
/// `pair` is a local variable and the boxes represent
/// Objects (they are named a1, a2, a3, and so on).
/// The internals of the boxes are fields.
///
/// ```notrust
///           ┌──────┐
///  pair─────┤[Pair]│   ┌───────┐
///           │ a ───┼──►│[Point]│
///           │      │   │ x ────┼─► a4 = 22
///           │ b ───┼─┐ │       │
///           │      │ │ │ y ────┼─► a5 = 44
///           └──────┘ │ └───────┘
///           a1       │ a2
///                    │
///                    │
///                    │ ┌───────┐
///                    └►│[Point]│
///                      │ x ────┼─► a6 = 66
///                      │       │
///                      │ y ────┼─► a7 = 88
///                      └───────┘
///                      a3
/// ```
///
/// `Stepper::traverse_to_place` invoked on `pair.a` would
/// yield a [`PlaceTraversal`] that accumulates the permissions
/// from the reference `pair` and which points to `&mut a1.a`.
///
/// `Stepper::traverse_to_object` invoked on `pair.a` would
/// point to `a2` and would hence include the permissions
/// from the outgoing edge from the field `a` to the object
/// `a2`.
#[derive(Debug)]
pub(super) struct PlaceTraversal {
    pub(super) accumulated_permissions: AccumulatedPermissions,
    pub(super) address: Address,
}

/// See [`PlaceTraversal`] for detailed explanation.
#[derive(Debug)]
pub(super) struct ObjectTraversal {
    pub(super) accumulated_permissions: AccumulatedPermissions,
    pub(super) object: Object,
}

impl Stepper<'_> {
    /// Returns a traversal that reaches the location `place`.
    /// The result includes the accumulated permissions as well as
    /// a `&mut Value` that represents where the place is stored
    /// in memory.
    ///
    /// If this returns `Ok`, the place is at least potentially *accessible*,
    /// though some of the objects along the way may currently be leased. If the place
    /// tries to dereference an expired permission, returns `Err`.
    pub(super) fn traverse_to_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<PlaceTraversal> {
        match place.data(table) {
            bir::PlaceData::LocalVariable(lv) => Ok(self.traverse_to_local_variable(table, *lv)),

            bir::PlaceData::Function(f) => Ok(self.traverse_to_constant(ObjectData::Function(*f))),
            bir::PlaceData::Class(c) => Ok(self.traverse_to_constant(ObjectData::Class(*c))),
            bir::PlaceData::Intrinsic(i) => {
                Ok(self.traverse_to_constant(ObjectData::Intrinsic(*i)))
            }
            bir::PlaceData::Dot(owner_place, field_name) => {
                let ObjectTraversal {
                    mut accumulated_permissions,
                    object: owner_object,
                } = self.traverse_to_object(table, *owner_place)?;
                let (field_atomic, field_index) =
                    self.object_field(place, owner_object, *field_name)?;

                accumulated_permissions.atomic |= field_atomic;

                Ok(PlaceTraversal {
                    accumulated_permissions,
                    address: Address::Field(owner_object, field_index),
                })
            }
        }
    }

    pub(super) fn traverse_to_local_variable(
        &mut self,
        table: &bir::Tables,
        lv: LocalVariable,
    ) -> PlaceTraversal {
        let lv_data = lv.data(table);
        let permissions = AccumulatedPermissions::unique(lv_data.atomic);
        PlaceTraversal {
            accumulated_permissions: permissions,
            address: Address::Local(lv),
        }
    }

    /// Returns a traversal that reaches the object located at `place`.
    /// This includes and accounts for the permissions from the reference
    /// to the object.
    ///
    /// If this returns `Ok`, the data in the object is at least potentially *accessible*,
    /// though some of the objects along the way may currently be leased. If the place
    /// tries to dereference an expired permission, returns `Err`.
    pub(super) fn traverse_to_object(
        &mut self,
        table: &bir::Tables,
        bir_place: bir::Place,
    ) -> eyre::Result<ObjectTraversal> {
        let PlaceTraversal {
            accumulated_permissions,
            address,
        } = self.traverse_to_place(table, bir_place)?;
        let Value { permission, object } = self.peek(address);
        let permissions =
            self.accumulate_permission(bir_place, accumulated_permissions, permission)?;

        Ok(ObjectTraversal {
            accumulated_permissions: permissions,
            object,
        })
    }

    pub(super) fn traverse_to_object_field(
        &mut self,
        place: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
        object_traversal: ObjectTraversal,
        field_name: Word,
    ) -> eyre::Result<PlaceTraversal> {
        let ObjectTraversal {
            mut accumulated_permissions,
            object: owner_object,
        } = object_traversal;
        let (field_atomic, field_index) = self.object_field(place, owner_object, field_name)?;

        accumulated_permissions.atomic |= field_atomic;

        Ok(PlaceTraversal {
            accumulated_permissions,
            address: Address::Field(owner_object, field_index),
        })
    }

    fn object_field(
        &mut self,
        place: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
        owner_object: Object,
        field_name: Word,
    ) -> eyre::Result<(Atomic, usize)> {
        // FIXME: Execute this before we create the mutable ref to `self.machine`,
        // even though we might not need it. The borrow checker is grumpy the ref
        // to self.machine is returned from the function and so it fails to analyze
        // it very well. We could fix this by refactoring `span_from_bir` into a
        // helper that is more parsimonius in what it takes as its inputs, but meh.
        let place_span = self.span_from_bir(place);
        match &mut self.machine[owner_object] {
            ObjectData::Instance(instance) => {
                if let Some(index) = instance.class.field_index(self.db, field_name) {
                    let atomic = instance.class.structure(self.db).field_atomic(index);
                    Ok((atomic, index))
                } else {
                    Err(Self::no_such_field(
                        self.db,
                        place_span,
                        instance.class,
                        field_name,
                    ))
                }
            }
            ObjectData::Tuple(tuple) => {
                let field_name_str = field_name.as_str(self.db);
                if let Ok(index) = field_name_str.parse::<usize>() {
                    if index < tuple.fields.len() && field_name_str == index.to_string() {
                        return Ok((Atomic::No, index));
                    }
                }
                Err(error!(place_span, "no field named `{}`", field_name_str).eyre(self.db))
            }
            owner_data => Err(Self::unexpected_kind(
                self.db,
                place_span,
                owner_data,
                "something with fields",
            )),
        }
    }

    fn traverse_to_constant(&mut self, object_data: ObjectData) -> PlaceTraversal {
        let object = self.machine.our_value(self.machine.pc(), object_data);
        let permissions = AccumulatedPermissions {
            traversed: vec![],
            leased: Leased::No,
            joint: Joint::Yes,
            atomic: Atomic::No,
        };
        PlaceTraversal {
            accumulated_permissions: permissions,
            address: Address::Constant(object),
        }
    }

    fn accumulate_permission(
        &mut self,
        place: bir::Place,
        accumulated_permissions: AccumulatedPermissions,
        permission: Permission,
    ) -> eyre::Result<AccumulatedPermissions> {
        // No matter what, we will traverse this permission.
        let mut traversed = accumulated_permissions.traversed;
        traversed.push(permission);

        let atomic = accumulated_permissions.atomic;

        match &self.machine[permission] {
            PermissionData::Expired(expired_at) => {
                tracing::debug!("encountered expired permission: {:?}", permission);
                let place_span = self.span_from_bir(place);
                Err(report_traversing_expired_permission(
                    self.db,
                    place_span,
                    *expired_at,
                ))
            }
            PermissionData::Valid(v) => {
                match v.joint {
                    Joint::Yes => {
                        // When we traverse into a joint permission, the path we took
                        // to get there is less relevant. For example, if we lease
                        // a shared object, we don't need to get a lease on the
                        // context where we *found* it.

                        Ok(AccumulatedPermissions {
                            traversed,
                            atomic,
                            leased: v.leased,
                            joint: Joint::Yes,
                        })
                    }

                    Joint::No => {
                        // Traversing to a non-joint permission: the context itself
                        // must be joint.

                        // Joint if the surrounding context is joint.
                        let joint = accumulated_permissions.joint;

                        // Leased if this permission is leased or we passed through
                        // a leased permission.
                        let leased = v.leased | accumulated_permissions.leased;

                        Ok(AccumulatedPermissions {
                            traversed,
                            atomic,
                            leased,
                            joint,
                        })
                    }
                }
            }
        }
    }
}

pub(super) fn report_traversing_expired_permission(
    db: &dyn crate::Db,
    place_span: FileSpan,
    expired_at: Option<ProgramCounter>,
) -> eyre::Report {
    match expired_at {
        None => error!(place_span, "accessing uninitialized memory").eyre(db),
        Some(expired_at) => {
            let expired_at_span = expired_at.span(db);

            let secondary_label = if expired_at.is_return(db) {
                "lease was cancelled when this function returned"
            } else {
                "lease was cancelled here"
            };

            error!(place_span, "your lease to this object was cancelled")
                .primary_label("cancelled lease used here")
                .secondary_label(expired_at_span, secondary_label)
                .eyre(db)
        }
    }
}

#[extension_trait::extension_trait]
impl PermissionExt for Permission {
    fn if_leased(self, l: Leased) -> Option<Permission> {
        match l {
            Leased::Yes => Some(self),
            Leased::No => None,
        }
    }
}
