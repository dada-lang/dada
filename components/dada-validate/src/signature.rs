use crate::name_lookup::Scope;
use crate::validate::root_definitions;
use dada_id::InternKey;
use dada_ir::class::Class;
use dada_ir::code::syntax::{self, LocalVariableDeclData, Signature};
use dada_ir::diagnostic::ErrorReported;
use dada_ir::error;
use dada_ir::function::{Function, FunctionSignature};
use dada_ir::signature::{
    self, ClassStructure, Field, GenericParameter, GenericParameterKind, KnownPermissionKind,
    ParameterIndex, Permission,
};
use dada_ir::span::Anchored;
use dada_ir::storage::Atomic;
use dada_ir::word::Words;
use derive_new::new;

use crate::name_lookup::Definition;

#[salsa::tracked(return_ref)]
pub(crate) fn validate_class_structure(
    db: &dyn crate::Db,
    class: Class,
) -> signature::ClassStructure {
    // FIXME: This setup is a bit bonkers. What should be happening is that
    // we should parse the class-signature-syntax along with other details
    // of the class to create the class-structure, and then derive the
    // Signature of the constructor from that. Right now we do it the other
    // way. I expect eventually we'll want to have fields not declared in the constructor.

    let class_signature_syntax = class.signature_syntax(db);
    let tables = &class_signature_syntax.tables;

    // Compute the signature of the class constructor
    let class_signature = validate_class_signature(db, class);

    //
    let fields = class_signature_syntax
        .parameters
        .iter()
        .zip(&class_signature.inputs)
        .map(|(lv, input_ty)| {
            let lv_data = lv.data(tables);
            let atomic = Atomic::from(lv_data.atomic);
            Field {
                atomic,
                name: input_ty.name,
                ty: input_ty.ty.clone(),
            }
        })
        .collect();

    ClassStructure::new(
        class_signature.generics.clone(),
        class_signature.where_clauses.clone(),
        fields,
    )
}

#[salsa::tracked(return_ref)]
pub(crate) fn validate_function_signature(
    db: &dyn crate::Db,
    function: Function,
) -> signature::Signature {
    match function.signature_syntax(db) {
        FunctionSignature::Main => signature::Signature {
            generics: vec![],
            where_clauses: vec![],
            inputs: vec![],
            output: None,
        },

        FunctionSignature::Syntax(s) => signature(db, &function, s),
    }
}

#[salsa::tracked(return_ref)]
pub(crate) fn validate_class_signature(db: &dyn crate::Db, class: Class) -> signature::Signature {
    let s = class.signature_syntax(db);
    signature(db, &class, s)
}

fn signature(
    db: &dyn crate::Db,
    anchor: &dyn Anchored,
    signature: &syntax::Signature,
) -> signature::Signature {
    let Signature { tables, spans, .. } = signature;

    // get the root definitions around this function
    let input_file = anchor.input_file(db);
    let root_definitions = root_definitions(db, input_file);

    // assemble the name resolution scope; it should include the parameters
    // as local variables
    let scope = Scope::root(db, root_definitions);
    let validator = SignatureValidator::new(db, anchor, tables, spans, scope, vec![], vec![]);
    validator.validate_signature(signature)
}

#[derive(new)]
struct SignatureValidator<'s> {
    db: &'s dyn crate::Db,
    anchor: &'s dyn Anchored,
    tables: &'s syntax::Tables,
    spans: &'s syntax::Spans,
    scope: Scope<'s, ()>,
    generic_parameters: Vec<signature::GenericParameter>,
    where_clauses: Vec<signature::WhereClause>,
}

impl SignatureValidator<'_> {
    fn validate_signature(mut self, signature: &syntax::Signature) -> signature::Signature {
        // Step 1: Insert parameter into the name resolution scope.
        for parameter in &signature.parameters {
            let LocalVariableDeclData { name, .. } = parameter.data(self.tables);
            let word = name.data(self.tables).word;
            self.scope.insert(word, ());
        }

        // Now resolve their types.
        let inputs: Vec<_> = signature
            .parameters
            .iter()
            .map(|parameter| {
                let LocalVariableDeclData { name, ty, .. } = parameter.data(self.tables);
                let word = name.data(self.tables).word;
                let ty = ty.map(|ty| self.validate_ty(ty));
                signature::InputTy { name: word, ty }
            })
            .collect();

        let output = self.validate_output_ty(signature.return_type);

        signature::Signature {
            generics: self.generic_parameters,
            where_clauses: self.where_clauses,
            inputs,
            output,
        }
    }

    fn validate_output_ty(&mut self, return_ty: Option<syntax::ReturnTy>) -> Option<signature::Ty> {
        Some(self.validate_ty(return_ty?.data(self.tables).ty?))
    }

    fn validate_ty(&mut self, ty: syntax::Ty) -> signature::Ty {
        let syntax::TyData { perm, path } = ty.data(self.tables);

        match (self.validate_opt_perm(*perm), self.validate_ty_path(*path)) {
            (Ok(permission), Ok(class)) => signature::Ty::Class(signature::ClassTy {
                permission,
                class,
                generics: vec![], // FIXME: not supporting generics yet really
            }),
            (Err(ErrorReported), _) | (_, Err(ErrorReported)) => signature::Ty::Error,
        }
    }

    fn validate_ty_path(&mut self, ty: syntax::Path) -> Result<Class, ErrorReported> {
        let syntax::PathData {
            start_name,
            dot_names,
        } = ty.data(self.tables);

        // FIXME: Eventually we'll want to parse more than just classes here
        let class_name = start_name.data(self.tables).word;
        let class = match self.scope.lookup(class_name) {
            Some(Definition::Class(class)) => class,
            _ => {
                return Err(error!(
                    self.spans[*start_name].anchor_to(self.db, self.anchor),
                    "expected the name of a class"
                )
                .emit(self.db));
            }
        };

        if let Some(dot_name) = dot_names.first() {
            return Err(error!(
                self.spans[*dot_name].anchor_to(self.db, self.anchor),
                "dotted type names not yet supported"
            )
            .emit(self.db));
        }

        Ok(class)
    }

    fn validate_opt_perm(
        &mut self,
        perm: Option<syntax::Perm>,
    ) -> Result<Permission, ErrorReported> {
        let Some(perm) = perm else {
            return Ok(self.add_generic_permission(KnownPermissionKind::Given));
        };

        match perm.data(self.tables) {
            // My and our are short for given/shared permissions with no lessors.
            syntax::PermData::My => Ok(Permission::Known(signature::KnownPermission {
                kind: signature::KnownPermissionKind::Given,
                paths: vec![],
            })),
            syntax::PermData::Our => Ok(Permission::Known(signature::KnownPermission {
                kind: signature::KnownPermissionKind::Shared,
                paths: vec![],
            })),

            // If user writes `shared String` or `leased String`, we convert
            // that to `P String` where `shared P` or `leased P`.
            syntax::PermData::Shared(None) => {
                Ok(self.add_generic_permission(KnownPermissionKind::Shared))
            }
            syntax::PermData::Leased(None) => {
                Ok(self.add_generic_permission(KnownPermissionKind::Leased))
            }

            // Otherwise, if they wrote `shared{..}` or `leased{..}`, convert the paths
            syntax::PermData::Shared(Some(paths)) => {
                Ok(Permission::Known(signature::KnownPermission {
                    kind: signature::KnownPermissionKind::Shared,
                    paths: self.validate_permission_paths(*paths)?,
                }))
            }
            syntax::PermData::Leased(Some(paths)) => {
                if paths.data(self.tables).paths.is_empty() {
                    return Err(error!(
                        self.spans[perm].anchor_to(self.db, self.anchor),
                        "leased permissions need at least one path"
                    )
                    .emit(self.db));
                }
                Ok(Permission::Known(signature::KnownPermission {
                    kind: signature::KnownPermissionKind::Leased,
                    paths: self.validate_permission_paths(*paths)?,
                }))
            }
        }
    }

    /// Add a new generic permission `P` and return a reference to it.
    /// Based on the `kind`, we may also add a where-clause like `P: shared` or `P: leased`.
    fn add_generic_permission(&mut self, kind: signature::KnownPermissionKind) -> Permission {
        let index = ParameterIndex::from(self.generic_parameters.len());
        let param = GenericParameter::new(GenericParameterKind::Permission, None, index);
        self.generic_parameters.push(param);

        let new_where_clause = match kind {
            KnownPermissionKind::Given => None,
            KnownPermissionKind::Shared => Some(signature::WhereClause::IsShared(index)),
            KnownPermissionKind::Leased => Some(signature::WhereClause::IsLeased(index)),
        };
        self.where_clauses.extend(new_where_clause);

        Permission::Parameter(index)
    }

    fn validate_permission_paths(
        &self,
        paths: syntax::PermPaths,
    ) -> Result<Vec<signature::Path>, ErrorReported> {
        let syntax::PermPathsData { paths } = paths.data(self.tables);
        paths
            .iter()
            .map(|p| self.validate_permission_path(p))
            .collect()
    }

    fn validate_permission_path(
        &self,
        path: &syntax::Path,
    ) -> Result<signature::Path, ErrorReported> {
        let syntax::PathData {
            start_name,
            dot_names,
        } = path.data(self.tables);

        let variable_name = start_name.data(self.tables).word;

        // A permission path must start with a local variable (for now, anyway)
        match self.scope.lookup(variable_name) {
            Some(Definition::LocalVariable(_)) => { /* good */ }
            _ => {
                return Err(error!(
                    self.spans[*start_name].anchor_to(self.db, self.anchor),
                    "permission path should be a local variable"
                )
                .emit(self.db));
            }
        }

        let field_names =
            Words::from_iter(self.db, dot_names.iter().map(|n| n.data(self.tables).word));

        Ok(signature::Path::new(variable_name, field_names))
    }
}
