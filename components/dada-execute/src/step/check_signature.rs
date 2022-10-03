use crate::{
    error::DiagnosticBuilderExt,
    machine::{
        op::MachineOp, ExpectedClassTy, ExpectedPermission, ExpectedPermissionKind, ExpectedTy,
        ObjectData, Permission, PermissionData, Value,
    },
};
use dada_collections::{Map, Set};
use dada_ir::{
    error,
    signature::{self, KnownPermissionKind},
    storage::{Joint, Leased},
    word::Word,
};
use derive_new::new;

use super::{traversal::report_traversing_expired_permission, Stepper};

impl Stepper<'_> {
    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn check_signature(
        &mut self,
        input_values: &[Value],
        signature: &signature::Signature,
    ) -> eyre::Result<Option<ExpectedTy>> {
        let values =
            GenericsInference::new(self.db, self.machine, signature).infer(input_values)?;
        tracing::debug!(?values);
        SignatureChecker::new(self.db, self.machine, signature, &values, input_values)
            .check_inputs()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn check_return_value(
        &self,
        machine_value: Value,
        expected_ty: &ExpectedTy,
    ) -> eyre::Result<()> {
        ExpectationChecker::new(self.db, self.machine).check_expected_ty(machine_value, expected_ty)
    }
}

#[derive(Debug, Default)]
pub(crate) struct GenericsValues {
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
    fn check_inputs(self) -> eyre::Result<Option<ExpectedTy>> {
        assert_eq!(self.input_values.len(), self.signature.inputs.len());

        self.check_where_clauses()?;

        for (input_value, input_ty) in self.input_values.iter().zip(&self.signature.inputs) {
            if let Some(ty) = &input_ty.ty {
                self.check_value_against_signature(*input_value, ty)?;
            }
        }

        let expected_return_ty = self
            .signature
            .output
            .as_ref()
            .map(|o| self.expected_ty_from_signature(o));

        tracing::debug!(?expected_return_ty);

        Ok(expected_return_ty)
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
        let expected_ty = self.expected_ty_from_signature(signature_ty);
        ExpectationChecker::new(self.db, self.machine)
            .check_expected_ty(machine_value, &expected_ty)
    }

    fn expected_ty_from_signature(&self, signature_ty: &signature::Ty) -> ExpectedTy {
        match signature_ty {
            signature::Ty::Parameter(_) => {
                unimplemented!("type parameters")
            }
            signature::Ty::Class(class_ty) => {
                let permission = self.expected_permission_from_signature(&class_ty.permission);

                ExpectedTy::Class(ExpectedClassTy {
                    permission,
                    class: class_ty.class,
                    generics: class_ty
                        .generics
                        .iter()
                        .map(|t| self.expected_ty_from_signature(t))
                        .collect(),
                })
            }
            signature::Ty::Error => ExpectedTy::Error,
        }
    }

    fn expected_permission_from_signature(
        &self,
        signature_permission: &signature::Permission,
    ) -> ExpectedPermission {
        match signature_permission {
            signature::Permission::Parameter(p) => {
                let values = &self.values.permissions[p];
                ExpectedPermission {
                    kind: ExpectedPermissionKind::Member,
                    declared_permissions: values.iter().copied().collect(),
                }
            }
            signature::Permission::Known(kp) => {
                let signature::KnownPermission { kind, paths } = kp;

                let declared_permissions = paths
                    .iter()
                    .map(|path| self.signature_path_to_permission(path))
                    .collect();

                ExpectedPermission {
                    kind: match kind {
                        KnownPermissionKind::Given => ExpectedPermissionKind::Given,
                        KnownPermissionKind::Leased => ExpectedPermissionKind::Leased,
                        KnownPermissionKind::Shared => ExpectedPermissionKind::Shared,
                    },
                    declared_permissions,
                }
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
}

#[derive(new)]
struct ExpectationChecker<'s> {
    db: &'s dyn crate::Db,
    machine: &'s dyn MachineOp,
}

impl ExpectationChecker<'_> {
    fn check_expected_ty(
        &self,
        machine_value: Value,
        expected_ty: &ExpectedTy,
    ) -> eyre::Result<()> {
        match expected_ty {
            ExpectedTy::Class(class_ty) => {
                self.check_expected_permission(machine_value.permission, &class_ty.permission)?;

                // FIXME: To support class generics and things, we have
                // to traverse the fields, at least if `machine_value` has a joint
                // permission. For example, if we had `P Vec[_]` being matched
                // against an actual value of `P Vec[Q String]`, we have to
                // walk the values in `Vec` and add each String permission to `P`.

                match &self.machine[machine_value.object] {
                    ObjectData::Instance(i) => {
                        if i.class != class_ty.class {
                            let span = self.machine.pc().span(self.db);
                            return Err(error!(
                                span,
                                "expected an instance of `{}`, but got an instance of `{}`",
                                class_ty.class.name(self.db).as_str(self.db),
                                i.class.name(self.db).as_str(self.db),
                            )
                            .eyre(self.db));
                        }
                    }

                    id => {
                        let span = self.machine.pc().span(self.db);
                        return Err(error!(
                            span,
                            "expected an instance of `{}`, but got {}",
                            class_ty.class.name(self.db).as_str(self.db),
                            id.kind_str(self.db),
                        )
                        .eyre(self.db));
                    }
                }

                // FIXME: check generics

                Ok(())
            }
            ExpectedTy::Error => Ok(()),
        }
    }

    fn check_expected_permission(
        &self,
        machine_permission: Permission,
        expected_permission: &ExpectedPermission,
    ) -> eyre::Result<()> {
        let ExpectedPermission {
            kind,
            declared_permissions,
        } = expected_permission;
        match kind {
            ExpectedPermissionKind::Member => {
                assert!(declared_permissions.contains(&machine_permission)); // ensured by inference
                Ok(())
            }
            ExpectedPermissionKind::Given => {
                self.matches_given(machine_permission, declared_permissions)
            }
            ExpectedPermissionKind::Leased => {
                self.matches_leased(machine_permission, declared_permissions)
            }
            ExpectedPermissionKind::Shared => {
                self.matches_shared(machine_permission, declared_permissions)
            }
        }
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
        declared_permissions: &[Permission],
    ) -> eyre::Result<()> {
        let is_shared = declared_permissions.iter().any(|&p| self.is_shared(p));
        let lessors: Vec<_> = declared_permissions
            .iter()
            .flat_map(|&p| self.new_lessor(p))
            .collect();
        self.matches_test("given", permission_in_value, is_shared, &lessors)
    }

    fn matches_shared(
        &self,
        permission_in_value: Permission,
        declared_permissions: &[Permission],
    ) -> eyre::Result<()> {
        let lessors: Vec<_> = declared_permissions
            .iter()
            .flat_map(|&p| self.new_lessor(p))
            .collect();
        self.matches_test("shared", permission_in_value, true, &lessors)
    }

    fn matches_leased(
        &self,
        permission_in_value: Permission,
        declared_permissions: &[Permission],
    ) -> eyre::Result<()> {
        self.matches_test("leased", permission_in_value, false, declared_permissions)
    }

    /// True if the permission `perm_target` of the value being returned matches the
    /// characteristics desired by its return type:
    ///
    /// * `is_shared` -- if false, i.e., return type demands a unique return, then `perm_target` must be unique
    /// * `lessors` -- `perm_target` must be leased from one of the lessors in this list,
    #[tracing::instrument(level = "debug", skip(self))]
    fn matches_test(
        &self,
        keyword: &str,
        permission_in_value: Permission,
        declared_ty_is_shared: bool,
        declared_lessors: &[Permission],
    ) -> eyre::Result<()> {
        let valid_permission_in_value = self.machine[permission_in_value].assert_valid();

        // If the return type demands a unique value, but a shared type was returned, false.
        if declared_ty_is_shared && !self.is_shared(permission_in_value) {
            let span = self.machine.pc().span(self.db);
            return Err(error!(
                span,
                "expected a shared value, got a `{}` value",
                valid_permission_in_value.as_str()
            )
            .eyre(self.db));
        } else if !declared_ty_is_shared && self.is_shared(permission_in_value) {
            let span = self.machine.pc().span(self.db);
            return Err(error!(
                span,
                "expected a `{}` value, got a `{}` value",
                keyword,
                valid_permission_in_value.as_str()
            )
            .eyre(self.db));
        }

        if self.is_leased(permission_in_value) && declared_lessors.is_empty() {
            let expected_kind = if declared_ty_is_shared { "our" } else { "my" };
            let span = self.machine.pc().span(self.db);
            return Err(error!(
                span,
                "expected a `{}` value, got a `{}` value",
                expected_kind,
                valid_permission_in_value.as_str()
            )
            .eyre(self.db));
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

            let mode = if declared_ty_is_shared {
                "shared"
            } else {
                "leased"
            };
            let span = self.machine.pc().span(self.db);
            return Err(error!(span, "not {mode} from the right place").eyre(self.db));
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

    #[tracing::instrument(skip(self))]
    fn is_transitive_lessor_of(&self, lessor: Permission, lessee: Permission) -> bool {
        if lessor == lessee {
            return true;
        }

        let lessor = self.machine[lessor].assert_valid();
        lessor
            .tenants
            .iter()
            .chain(&lessor.easements)
            .any(|&t| self.is_transitive_lessor_of(t, lessee))
    }
}
