use crate::{
    error::DiagnosticBuilderExt,
    machine::{op::MachineOp, Permission, PermissionData, Value},
};
use dada_collections::{Map, Set};
use dada_ir::{
    error, signature,
    storage::{Joint, Leased},
};

use super::{traversal::report_traversing_expired_permission, Stepper};

impl Stepper<'_> {
    pub(super) fn check_signature(
        &mut self,
        inputs: &[Value],
        signature: &signature::Signature,
    ) -> eyre::Result<()> {
        let mut checker = SignatureChecker::new(self.db, self.machine, signature);
        checker.check_inputs(inputs)?;
        Ok(())
    }
}

struct SignatureChecker<'s> {
    db: &'s dyn crate::Db,
    machine: &'s dyn MachineOp,
    signature: &'s signature::Signature,
    generic_permission_values: Map<signature::ParameterIndex, Set<Permission>>,
}

impl<'s> SignatureChecker<'s> {
    fn new(
        db: &'s dyn crate::Db,
        machine: &'s dyn MachineOp,
        signature: &'s signature::Signature,
    ) -> Self {
        let mut this = Self {
            db,
            signature,
            machine,
            generic_permission_values: Default::default(),
        };

        for generic in &signature.generics {
            this.init_generic(generic);
        }

        this
    }

    fn init_generic(&mut self, generic: &'s signature::GenericParameter) {
        match generic.kind {
            signature::GenericParameterKind::Permission => {
                self.generic_permission_values
                    .insert(generic.index, Default::default());
            }
            signature::GenericParameterKind::Type => unimplemented!("type parameters"),
        }
    }

    fn check_inputs(&mut self, input_values: &[Value]) -> eyre::Result<()> {
        assert_eq!(input_values.len(), self.signature.inputs.len());

        // First: infer the values of any generic parameters.
        for (input_value, input_ty) in input_values.iter().zip(&self.signature.inputs) {
            if let Some(ty) = &input_ty.ty {
                self.infer_generics_from_input_value(*input_value, ty)?;
            }
        }

        self.check_where_clauses()?;

        // UP NEXT: now that we know the values of the generics, we can check the
        // declared permissions against the actual permissions we got.
        //
        //

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
                &self.generic_permission_values[p],
            ),
            signature::WhereClause::IsLeased(p) => self.check_permission_against_where_clause(
                "leased",
                Some(Joint::No),
                Some(Leased::Yes),
                &self.generic_permission_values[p],
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
            match &self.machine[permission] {
                PermissionData::Expired(_) => {
                    unreachable!("expired machine permission as value of generic parameter")
                }
                PermissionData::Valid(v) => {
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
            }
        }
        Ok(())
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
            self.generic_permission_values
                .get_mut(index)
                .unwrap()
                .insert(machine_permission);
        }

        Ok(())
    }
}
