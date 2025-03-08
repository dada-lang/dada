use dada_ir_sym::{
    ir::{
        classes::{SymAggregate, SymAggregateStyle},
        indices::InferVarIndex,
        primitive::SymPrimitiveKind,
        red::{RedInfer, RedInfers, RedTy},
        types::{SymGenericTerm, SymPerm, SymPermKind, SymTy, SymTyKind, SymTyName},
        variables::SymVariable,
    },
    prelude::CheckedFieldTy,
};
use dada_util::Map;
use wasm_encoder::ValType;

/// The WASM representation for a Dada value independent of the place in which it is stored.
/// This isn't really a specific representation, in some sense,
/// but rather enough information to determine how to represent
/// an instance of this value in any of the places it could appear:
///
/// * On the WebAssembly stack or memory, in which case all the
///   [flattened values](`WasmRepr::flatten`) would be pushed/stored one after the other.
/// * In WebAssembly local variables, in which case a [subset of the values](`WasmRepr::local_val_tys`)
///   would be stored in subsequent variables. Note that the data for classes
///   never appears in locals, and so a single value can be partially stored in locals and
///   partially in memory.
///
/// # See also
///
/// The [`WasmPlaceRepr`][] type describes the representation of
/// particular Dada place (which in turn has an associated Dada type).
///
/// [`WasmPlaceRepr`]: `crate::cx::generate_expr::wasm_place_repr::WasmPlaceRepr`
#[derive(Debug)]
pub(crate) enum WasmRepr {
    /// Indicates a single primitive value. This may appear on the WASM stack,
    /// a local value, or the memory, depending on the context in which it appears.
    Val(ValType),

    /// An aggregate value type. The values needed to represent its fields
    /// are found in the `Vec<WasmRepr>` argument.
    Struct(Vec<WasmRepr>),

    /// A class. The data for classes is always stored in WASM memory.
    /// It begins with an (implicit) I32 flag word and then contains
    /// whatever values are needed to represent the fields, stored as a `Vec<WasmRepr>`.
    Class(Vec<WasmRepr>),

    /// No data at all (something zero-sized).
    Nothing,
}

type Generics<'db> = Map<SymVariable<'db>, SymGenericTerm<'db>>;

pub(super) struct WasmReprCx<'g, 'db> {
    db: &'db dyn crate::Db,
    generics: &'g Generics<'db>,
    red_infers: &'g RedInfers<'db>,
}

impl<'g, 'db> WasmReprCx<'g, 'db> {
    pub(super) fn new(
        db: &'db dyn crate::Db,
        generics: &'g Generics<'db>,
        red_infers: &'g RedInfers<'db>,
    ) -> Self {
        Self {
            db,
            generics,
            red_infers,
        }
    }

    /// Returns the [`WasmRepr`][] that describes how `of_type` will be represented in WASM.
    pub(super) fn wasm_repr_of_type(&mut self, of_type: SymTy<'db>) -> WasmRepr {
        let db = self.db;
        match *of_type.kind(db) {
            SymTyKind::Named(ty_name, ref ty_args) => {
                self.wasm_repr_of_named_type(ty_name, ty_args)
            }
            SymTyKind::Var(sym_variable) => self.wasm_repr_of_variable(sym_variable),
            SymTyKind::Infer(infer) => self.wasm_repr_of_infer(infer),
            SymTyKind::Never | SymTyKind::Error(_) => WasmRepr::Nothing,
            SymTyKind::Perm(sym_perm, sym_ty) => self.wasm_repr_of_perm_type(sym_perm, sym_ty),
        }
    }

    fn wasm_repr_of_red_ty(&mut self, red_ty: &RedTy<'db>) -> WasmRepr {
        // Not ideal: Kind of annoying to have this *and* [`Self::wasm_repr_of_type`].
        // Not sure what to do about it though.
        match *red_ty {
            RedTy::Error(_) => WasmRepr::Nothing,
            RedTy::Named(ty_name, ref ty_args) => self.wasm_repr_of_named_type(ty_name, ty_args),
            RedTy::Never => WasmRepr::Nothing,
            RedTy::Infer(infer) => self.wasm_repr_of_infer(infer),
            RedTy::Var(var) => self.wasm_repr_of_variable(var),
            RedTy::Perm => panic!("cannot compute representation of a permission"),
        }
    }

    fn wasm_repr_of_variable(&mut self, sym_variable: SymVariable<'db>) -> WasmRepr {
        let result = self
            .generics
            .get(&sym_variable)
            .expect("expected value for each generic type")
            .assert_type(self.db);
        self.wasm_repr_of_type(result)
    }

    fn wasm_repr_of_infer(&mut self, infer: InferVarIndex) -> WasmRepr {
        match self.red_infers.red_infer(infer) {
            RedInfer::Perm { .. } => {
                panic!("cannot compute representation of a permission inference variable")
            }
            RedInfer::Ty { perm: _, red_ty } => self.wasm_repr_of_red_ty(red_ty),
        }
    }

    fn wasm_repr_of_perm_type(&mut self, sym_perm: SymPerm<'db>, sym_ty: SymTy<'db>) -> WasmRepr {
        let db = self.db;
        match *sym_perm.kind(db) {
            // A leased type is a pointer to the data.
            SymPermKind::Leased(_) => self.wasm_pointer(),

            // Owned or shared types are copies of the data.
            SymPermKind::My | SymPermKind::Our | SymPermKind::Shared(_) => {
                self.wasm_repr_of_type(sym_ty)
            }

            // Substitute generics.
            SymPermKind::Var(sym_variable) => {
                let result = self
                    .generics
                    .get(&sym_variable)
                    .expect("expected value for each generic type")
                    .assert_perm(db);
                self.wasm_repr_of_perm_type(result, sym_ty)
            }

            // Error types are zero-sized.
            SymPermKind::Error(_) => WasmRepr::Nothing,

            SymPermKind::Apply(left, _) => self.wasm_repr_of_perm_type(left, sym_ty),

            SymPermKind::Infer(_infer_var_index) => unreachable!(),
        }
    }

    /// Returns the [`WasmRepr`][] for a Dada named type.
    fn wasm_repr_of_named_type(
        &mut self,
        ty_name: SymTyName<'db>,
        ty_args: &Vec<SymGenericTerm<'db>>,
    ) -> WasmRepr {
        let db = self.db;
        match ty_name {
            SymTyName::Primitive(sym_primitive) => {
                WasmRepr::Val(self.wasm_valtype_for_primitive_kind(sym_primitive.kind(db)))
            }
            SymTyName::Aggregate(aggr) => match aggr.style(db) {
                // structs  have the fields inlined
                SymAggregateStyle::Struct => {
                    WasmRepr::Struct(self.wasm_repr_of_aggr_fields(aggr, ty_args))
                }

                SymAggregateStyle::Class => {
                    WasmRepr::Class(self.wasm_repr_of_aggr_fields(aggr, ty_args))
                }
            },
            SymTyName::Future => {
                assert_eq!(ty_args.len(), 1);
                let ty_arg = ty_args[0].assert_type(db);
                WasmRepr::Class(vec![self.wasm_repr_of_type(ty_arg)])
            }
            SymTyName::Tuple { arity } => {
                assert_eq!(ty_args.len(), arity);
                WasmRepr::Struct(
                    ty_args
                        .iter()
                        .map(|term| self.wasm_repr_of_type(term.assert_type(db)))
                        .collect(),
                )
            }
        }
    }

    /// The WASM [`ValType`][] for a Dada primtive. Note that small Dada values like `i16` or whatever
    /// are just promoted up to `I32` because we are lazy.
    fn wasm_valtype_for_primitive_kind(&self, primitive: SymPrimitiveKind) -> ValType {
        match primitive {
            SymPrimitiveKind::Bool => ValType::I32,
            SymPrimitiveKind::Char => ValType::I32,
            SymPrimitiveKind::Int { bits } | SymPrimitiveKind::Uint { bits } => match bits {
                0..=32 => ValType::I32,
                33..=64 => ValType::I64,
                _ => panic!("unexpectedly large number of integer bits {bits}"),
            },
            SymPrimitiveKind::Usize | SymPrimitiveKind::Isize => self.pointer_val_type(),
            SymPrimitiveKind::Float { bits } => match bits {
                32 => ValType::F32,
                64 => ValType::F64,
                _ => panic!("unexpected number of floating point bits {bits}"),
            },
        }
    }

    /// The WASM representations for the fields of some aggregate type
    /// (could be a struct or a class).
    fn wasm_repr_of_aggr_fields(
        &mut self,
        aggr: SymAggregate<'db>,
        ty_args: &Vec<SymGenericTerm<'db>>,
    ) -> Vec<WasmRepr> {
        self.aggr_field_tys(aggr, ty_args)
            .iter()
            .map(|ty| self.wasm_repr_of_type(*ty))
            .collect()
    }

    /// The types of each field of some aggregate type given the values `ty_args` for its generic arguments.
    fn aggr_field_tys<'a>(
        &self,
        aggr: SymAggregate<'db>,
        ty_args: &'a Vec<SymGenericTerm<'db>>,
    ) -> Vec<SymTy<'db>> {
        let db = self.db;
        aggr.fields(db)
            .map(|f| f.checked_field_ty(db))
            .map(|ty| {
                let ty = ty.substitute(db, ty_args);
                #[expect(unreachable_code)]
                ty.substitute(db, &[SymGenericTerm::Place(todo!())])
            })
            .collect()
    }

    /// The WASM representation for a pointer value.
    fn wasm_pointer(&self) -> WasmRepr {
        WasmRepr::Val(self.pointer_val_type())
    }

    /// The [`ValType`][] for a pointer value.
    /// For now, hardcoded to [`ValType::I32`][] but if/when 64-bit wasm exists this could change.
    fn pointer_val_type(&self) -> ValType {
        ValType::I32
    }
}
