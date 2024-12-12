use dada_ir_sym::{
    ir::classes::{SymAggregate, SymAggregateStyle},
    prelude::CheckedFieldTy,
    ir::primitive::SymPrimitiveKind,
    ir::variables::SymVariable,
    ir::types::{SymGenericTerm, SymPlace, SymTy, SymTyKind, SymTyName},
};
use dada_util::Map;
use wasm_encoder::ValType;

use super::Cx;

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

pub(crate) struct FnTypeIndex(u32);

impl From<FnTypeIndex> for u32 {
    fn from(value: FnTypeIndex) -> Self {
        value.0
    }
}

impl<'db> Cx<'db> {
    /// Declares an instantiation of a function with a given set of arguments and returns its index.
    /// If the function is already declared, nothing happens.
    /// If the function is not already declared, it is enqueued for code-generation.
    pub(crate) fn declare_fn_type(
        &mut self,
        inputs: Vec<ValType>,
        outputs: Vec<ValType>,
    ) -> FnTypeIndex {
        let index = self.type_section.len();
        self.type_section.ty().function(inputs, outputs);
        FnTypeIndex(index)
    }

    /// Returns the [`WasmRepr`][] that describes how `of_type` will be represented in WASM.
    pub(super) fn wasm_repr_of_type(
        &self,
        of_type: SymTy<'db>,
        generics: &Generics<'db>,
    ) -> WasmRepr {
        let db = self.db;
        match of_type.kind(db) {
            SymTyKind::Named(ty_name, ty_args) => {
                self.wasm_repr_of_named_type(*ty_name, ty_args, generics)
            }
            SymTyKind::Var(sym_variable) => {
                let result = generics
                    .get(sym_variable)
                    .expect("expected value for each generic type")
                    .assert_type(db);
                self.wasm_repr_of_type(result, generics)
            }
            SymTyKind::Infer(_) => {
                panic!("encountered unresolved inference variable")
            }
            SymTyKind::Never | SymTyKind::Error(_) => WasmRepr::Nothing,
            SymTyKind::Perm(sym_perm, sym_ty) => todo!(),
        }
    }

    /// Returns the [`WasmRepr`][] for a Dada named type.
    fn wasm_repr_of_named_type(
        &self,
        ty_name: SymTyName<'db>,
        ty_args: &Vec<SymGenericTerm<'db>>,
        generics: &Generics<'db>,
    ) -> WasmRepr {
        let db = self.db;
        match ty_name {
            SymTyName::Primitive(sym_primitive) => {
                WasmRepr::Val(self.wasm_valtype_for_primitive_kind(sym_primitive.kind(db)))
            }
            SymTyName::Aggregate(aggr) => match aggr.style(db) {
                // structs  have the fields inlined
                SymAggregateStyle::Struct => WasmRepr::Struct(
                    self.wasm_repr_of_aggr_fields(aggr, ty_args, generics)
                        .collect(),
                ),

                SymAggregateStyle::Class => WasmRepr::Class(
                    self.wasm_repr_of_aggr_fields(aggr, ty_args, generics)
                        .collect(),
                ),
            },
            SymTyName::Future => {
                assert_eq!(ty_args.len(), 1);
                let ty_arg = ty_args[0].assert_type(db);
                WasmRepr::Class(vec![self.wasm_repr_of_type(ty_arg, generics)])
            }
            SymTyName::Tuple { arity } => {
                assert_eq!(ty_args.len(), arity);
                WasmRepr::Struct(
                    ty_args
                        .iter()
                        .map(|term| self.wasm_repr_of_type(term.assert_type(db), generics))
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
            SymPrimitiveKind::Usize | SymPrimitiveKind::Isize => ValType::I32,
            SymPrimitiveKind::Float { bits } => match bits {
                32 => ValType::F32,
                64 => ValType::F64,
                _ => panic!("unexpected number of floating point bits {bits}"),
            },
        }
    }

    /// The WASM representations for the fields of some aggregate type
    /// (could be a struct or a class).
    fn wasm_repr_of_aggr_fields<'a>(
        &'a self,
        aggr: SymAggregate<'db>,
        ty_args: &'a Vec<SymGenericTerm<'db>>,
        generics: &'a Generics<'db>,
    ) -> impl Iterator<Item = WasmRepr> + use<'a, 'db> {
        self.aggr_field_tys(aggr, ty_args)
            .map(move |ty| self.wasm_repr_of_type(ty, generics))
    }

    /// The types of each field of some aggregate type given the values `ty_args` for its generic arguments.
    fn aggr_field_tys<'a>(
        &self,
        aggr: SymAggregate<'db>,
        ty_args: &'a Vec<SymGenericTerm<'db>>,
    ) -> impl Iterator<Item = SymTy<'db>> + use<'a, 'db> {
        let db = self.db;
        aggr.fields(db).map(|f| f.checked_field_ty(db)).map(|ty| {
            let ty = ty.substitute(db, ty_args);
            ty.substitute(db, &[SymGenericTerm::Place(todo!())])
        })
    }
}
