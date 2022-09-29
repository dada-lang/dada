use crate::{
    error::DiagnosticBuilderExt,
    machine::{op::MachineOp, Permission, PermissionData, Value},
};
use dada_collections::{Map, Set};
use dada_ir::{
    error, signature,
    storage::{Joint, Leased},
    word::Word,
};
use derive_new::new;

use super::{traversal::report_traversing_expired_permission, Stepper};

impl Stepper<'_> {
    pub(super) fn check_signature(
        &mut self,
        input_values: &[Value],
        signature: &signature::Signature,
    ) -> eyre::Result<()> {
        let values =
            GenericsInference::new(self.db, self.machine, signature).infer(input_values)?;
        SignatureChecker::new(self.db, self.machine, signature, &values, input_values)
            .check_inputs()?;
        Ok(())
    }
}

#[derive(Default)]
struct GenericsValues {
    permissions: Map<signature::ParameterIndex, Set<Permission>>,
}

struct GenericsInference<'s> {
    db: &'s dyn crate::Db,
    machine: &'s dyn MachineOp,
    signature: &'s signature::Signature,
    values: GenericsValues,
}

impl<'s> GenericsInference<'s> {
    fn new(
        db: &'s dyn crate::Db,
        machine: &'s dyn MachineOp,
        signature: &'s signature::Signature,
    ) -> Self {
        Self {
            db,
            signature,
            machine,
            values: Default::default(),
        }
    }

    fn infer(mut self, input_values: &[Value]) -> eyre::Result<GenericsValues> {
        for generic in &self.signature.generics {
            self.init_generic(generic);
        }

        for (input_value, input_ty) in input_values.iter().zip(&self.signature.inputs) {
            if let Some(ty) = &input_ty.ty {
                self.infer_generics_from_input_value(*input_value, ty)?;
            }
        }

        Ok(self.values)
    }

    fn init_generic(&mut self, generic: &'s signature::GenericParameter) {
        match generic.kind {
            signature::GenericParameterKind::Permission => {
                self.values
                    .permissions
                    .insert(generic.index, Default::default());
            }
            signature::GenericParameterKind::Type => unimplemented!("type parameters"),
        }
    }

    fn infer_generics_from_input_value(
        &mut self,
        machine_value: Value,
        ty: &signature::Ty,
    ) -> eyre::Result<()> {
        match ty {
            signature::Ty::Parameter(_) => {
                unimplemented!("type parameters")
            }
            signature::Ty::Class(class_ty) => {
                self.infer_generics_from_permission(
                    machine_value.permission,
                    &class_ty.permission,
                )?;

                // FIXME: To support class generics and things, we have
                // to traverse the fields, at least if `machine_value` has a joint
                // permission. For example, if we had `P Vec[_]` being matched
                // against an actual value of `P Vec[Q String]`, we have to
                // walk the values in `Vec` and add each String permission to `P`.

                Ok(())
            }
            signature::Ty::Error => Ok(()),
        }
    }

    fn infer_generics_from_permission(
        &mut self,
        machine_permission: Permission,
        signature_permission: &signature::Permission,
    ) -> eyre::Result<()> {
        if let PermissionData::Expired(expired_at) = &self.machine[machine_permission] {
            let span = self.machine.pc().span(self.db);
            return Err(report_traversing_expired_permission(
                self.db,
                span,
                *expired_at,
            ));
        };

        if let signature::Permission::Parameter(index) = signature_permission {
            self.values
                .permissions
                .get_mut(index)
                .unwrap()
                .insert(machine_permission);
        }

        Ok(())
    }
}

#[derive(new)]
struct SignatureChecker<'s> {
    db: &'s dyn crate::Db,
    machine: &'s dyn MachineOp,
    signature: &'s signature::Signature,
    values: &'s GenericsValues,
    input_values: &'s [Value],
}

impl SignatureChecker<'_> {
    fn check_inputs(self) -> eyre::Result<()> {
        assert_eq!(self.input_values.len(), self.signature.inputs.len());

        self.check_where_clauses()?;

        for (input_value, input_ty) in self.input_values.iter().zip(&self.signature.inputs) {
            if let Some(ty) = &input_ty.ty {
                self.check_value_against_signature(*input_value, ty)?;
            }
        }

        Ok(())
    }

    fn check_where_clauses(&self) -> eyre::Result<()> {
        for where_clause in &self.signature.where_clauses {
            self.check_where_clause(where_clause)?;
        }
        Ok(())
    }

    fn check_where_clause(&self, where_clause: &signature::WhereClause) -> eyre::Result<()> {
        match where_clause {
            signature::WhereClause::IsShared(p) => self.check_permission_against_where_clause(
                "shared",
                Some(Joint::Yes),
                None,
                &self.values.permissions[p],
            ),
            signature::WhereClause::IsLeased(p) => self.check_permission_against_where_clause(
                "leased",
                Some(Joint::No),
                Some(Leased::Yes),
                &self.values.permissions[p],
            ),
        }
    }

    fn check_permission_against_where_clause(
        &self,
        expected_label: &str,
        expected_joint: Option<Joint>,
        expected_leased: Option<Leased>,
        permissions: &Set<Permission>,
    ) -> eyre::Result<()> {
        for &permission in permissions {
            let v = self.machine[permission].assert_valid();
            let bad_joint = expected_joint.map(|e| e != v.joint).unwrap_or(false);
            let bad_leased = expected_leased.map(|e| e != v.leased).unwrap_or(false);
            if bad_joint || bad_leased {
                let pc_span = self.machine.pc().span(self.db);

                let actual_label = v.as_str();

                // FIXME: we need to decide how to thread span and other information
                // so we can give a decent error here. Maybe need to change the
                // validated signature into something with tables.

                return Err(error!(
                    pc_span,
                    "expected a `{expected_label}` value, but got a `{actual_label}` value"
                )
                .eyre(self.db));
            }
        }
        Ok(())
    }

    fn check_value_against_signature(
        &self,
        machine_value: Value,
        signature_ty: &signature::Ty,
    ) -> eyre::Result<()> {
        match signature_ty {
            signature::Ty::Parameter(_) => {
                unimplemented!("type parameters")
            }
            signature::Ty::Class(class_ty) => {
                self.check_permission_against_signature(
                    machine_value.permission,
                    &class_ty.permission,
                )?;

                // FIXME: To support class generics and things, we have
                // to traverse the fields, at least if `machine_value` has a joint
                // permission. For example, if we had `P Vec[_]` being matched
                // against an actual value of `P Vec[Q String]`, we have to
                // walk the values in `Vec` and add each String permission to `P`.

                Ok(())
            }
            signature::Ty::Error => Ok(()),
        }
    }

    fn check_permission_against_signature(
        &self,
        machine_permission: Permission,
        signature_permission: &signature::Permission,
    ) -> eyre::Result<()> {
        match signature_permission {
            signature::Permission::Parameter(p) => {
                let values = &self.values.permissions[p];
                assert!(values.contains(&machine_permission)); // ensured by inference
                Ok(())
            }
            signature::Permission::Known(kp) => {
                let signature::KnownPermission { kind, paths } = kp;

                let permissions = paths
                    .iter()
                    .map(|path| self.signature_path_to_permission(path))
                    .collect();

                match kind {
                    signature::KnownPermissionKind::Given => {
                        self.matches_given(machine_permission, permissions)?
                    }
                    signature::KnownPermissionKind::Leased => {
                        self.matches_leased(machine_permission, permissions)?
                    }
                    signature::KnownPermissionKind::Shared => {
                        self.matches_shared(machine_permission, permissions)?
                    }
                }

                Ok(())
            }
        }
    }

    fn input_value_named(&self, name: Word) -> Value {
        // Find index of the parameter with this name:
        // there must be one, or signature
        // verification would've failed.
        let index = self
            .signature
            .inputs
            .iter()
            .zip(0..)
            .flat_map(|(i, n)| if i.name == name { Some(n) } else { None })
            .next()
            .unwrap();

        self.input_values[index]
    }

    fn signature_path_to_permission(&self, path: &signature::Path) -> Permission {
        let signature::Path {
            variable_name,
            field_names,
        } = path;

        let value = self.input_value_named(*variable_name);

        assert!(
            field_names.elements(self.db).is_empty(),
            "TODO: paths with fields"
        ); // FIXME

        value.permission
    }

    /// This is a weird function. The intuition is that it returns:
    ///
    /// * If you give a permission `perm`, returns the lessor on the new permission.
    ///
    /// Not coincidentally, this is also the same as the lessor if you share a value with the permission `P`.
    ///
    /// It is NOT the same as the lessor you get if you lease a place with the permission `P`, because that
    /// has `P` as the lessor.
    fn new_lessor(&self, machine_permission: Permission) -> Option<Permission> {
        match self.machine[machine_permission].assert_valid().leased {
            Leased::Yes => Some(machine_permission),
            Leased::No => None,
        }
    }

    fn is_shared(&self, permission: Permission) -> bool {
        bool::from(self.machine[permission].assert_valid().joint)
    }

    fn is_leased(&self, permission: Permission) -> bool {
        bool::from(self.machine[permission].assert_valid().leased)
    }

    fn matches_given(
        &self,
        permission_in_value: Permission,
        permissions_in_declared_ty: Vec<Permission>,
    ) -> eyre::Result<()> {
        let is_shared = permissions_in_declared_ty
            .iter()
            .any(|&p| self.is_shared(p));
        let lessors = permissions_in_declared_ty
            .iter()
            .flat_map(|&p| self.new_lessor(p))
            .collect();
        self.matches_test(permission_in_value, is_shared, lessors)
    }

    fn matches_shared(
        &self,
        permission_in_value: Permission,
        permissions_in_declared_ty: Vec<Permission>,
    ) -> eyre::Result<()> {
        let lessors = permissions_in_declared_ty
            .iter()
            .flat_map(|&p| self.new_lessor(p))
            .collect();
        self.matches_test(permission_in_value, true, lessors)
    }

    fn matches_leased(
        &self,
        permission_in_value: Permission,
        permissions_in_declared_ty: Vec<Permission>,
    ) -> eyre::Result<()> {
        self.matches_test(permission_in_value, false, permissions_in_declared_ty)
    }

    /// True if the permission `perm_target` of the value being returned matches the
    /// characteristics desired by its return type:
    ///
    /// * `is_shared` -- if false, i.e., return type demands a unique return, then `perm_target` must be unique
    /// * `lessors` -- `perm_target` must be leased from one of the lessors in this list,
    fn matches_test(
        &self,
        permission_in_value: Permission,
        declared_ty_is_shared: bool,
        declared_lessors: Vec<Permission>,
    ) -> eyre::Result<()> {
        // If the return type demands a unique value, but a shared type was returned, false.
        if !declared_ty_is_shared && self.is_shared(permission_in_value) {
            let span = self.machine.pc().span(self.db);
            return Err(error!(span, "foo").eyre(self.db));
        }

        // If the value returned has a lessor...
        if self.is_leased(permission_in_value) {
            // ...then `perm_target` must be leased from a member of `lessors`.
            if declared_lessors
                .iter()
                .any(|&l| self.is_transitive_lessor_of(l, permission_in_value))
            {
                return Ok(());
            }

            let span = self.machine.pc().span(self.db);
            return Err(error!(span, "not leased from the right place").eyre(self.db));
        }

        // Otherwise, the return value is owned. That is ok if the return type is
        // shared or `my`, but we can't have an owned return value when the return
        // type is something like `leased{a}`. In that case the value HAS to be leased
        // from `a`. This is required because the compiler will represent it as a pointer
        // to `a`, so we can't substitute an owned value.
        if declared_ty_is_shared || declared_lessors.is_empty() {
            Ok(())
        } else {
            let span = self.machine.pc().span(self.db);
            Err(error!(span, "expected a `leased` value, but `my` value returned").eyre(self.db))
        }
    }

    fn is_transitive_lessor_of(&self, lessor: Permission, lessee: Permission) -> bool {
        lessor == lessee
            || self.machine[lessor]
                .assert_valid()
                .tenants
                .iter()
                .any(|&t| self.is_transitive_lessor_of(t, lessee))
    }
}
