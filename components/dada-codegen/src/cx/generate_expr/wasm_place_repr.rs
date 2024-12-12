use std::sync::Arc;

use dada_ir_sym::{
    ir::classes::SymField,
    ir::exprs::{SymPlaceExpr, SymPlaceExprKind},
    ir::variables::SymVariable,
    ir::types::{SymTy, SymTyKind, SymTyName},
};
use wasm_encoder::{Instruction, ValType};

use crate::cx::wasm_repr::WasmRepr;

use super::ExprCodegen;

/// The WASM representation for a Dada place. Dada places can be
/// spread across the WASM local variables and
/// The [`EmplacedWasmRepr`][] type tells you which values are stored
/// where.
#[derive(Debug)]
pub enum WasmPlaceRepr {
    /// A primitive value stored in a WASM local variable.
    Local(WasmLocal, ValType),
    Heap(WasmPointer, ValType),
    Struct(Vec<Arc<WasmPlaceRepr>>),
    Class(WasmPointer, Vec<Arc<WasmPlaceRepr>>),
    Nowhere,
}

impl<'cx, 'db> ExprCodegen<'cx, 'db> {
    /// Returns a [`WasmPointer`][] to the current start of a callee's stack frame.
    /// This value is only valid until [`Self::insert_variable`][] is next called.
    pub(super) fn next_stack_frame(&self) -> WasmPointer {
        WasmPointer {
            base_variable: self.wasm_stack_pointer,
            offset: self.wasm_stack_frame_size,
        }
    }

    /// Introduce the variable `lv` into scope and create a place for it.
    /// This can allocate more stack space in WASM memory.
    /// You can find this place by invoking [`Self::local`][] later on.
    pub(super) fn insert_variable(&mut self, lv: SymVariable<'db>, ty: SymTy<'db>) {
        let ty_repr = self.cx.wasm_repr_of_type(ty, &self.generics);
        let emplaced_repr = self.emplace_local(&ty_repr);
        self.variables.insert(lv, emplaced_repr);
    }

    /// The representation of the place represented by `local_variable`.
    pub(super) fn place_for_local(&self, local_variable: SymVariable<'db>) -> Arc<WasmPlaceRepr> {
        self.variables[&local_variable].clone()
    }

    /// The representation of the given Dada place.
    pub(super) fn place(&self, place: SymPlaceExpr<'db>) -> Arc<WasmPlaceRepr> {
        let db = self.cx.db;
        match *place.kind(db) {
            SymPlaceExprKind::Var(v) => self.place_for_local(v).into(),
            SymPlaceExprKind::Error(_) => Arc::new(WasmPlaceRepr::Nowhere),
            SymPlaceExprKind::Field(owner, field) => {
                let owner_place = self.place(owner);
                self.field_place(owner_place, owner.ty(db), field)
            }
        }
    }

    /// Push the value found in `place` onto the WASM stack.
    pub(super) fn push_from(&mut self, place: &WasmPlaceRepr) {
        match *place {
            WasmPlaceRepr::Local(wasm_local, val_type) => {
                self.push_from_local(val_type, wasm_local)
            }
            WasmPlaceRepr::Heap(wasm_pointer, val_type) => {
                self.push_from_memory(val_type, wasm_pointer)
            }
            WasmPlaceRepr::Struct(ref fields) => {
                fields.iter().for_each(|r| self.push_from(r));
            }
            WasmPlaceRepr::Class(flags, ref fields) => {
                self.push_from_memory(ValType::I32, flags);
                fields.iter().for_each(|r| self.push_from(r));
            }
            WasmPlaceRepr::Nowhere => (),
        }
    }

    /// Push a shared copy the value found in `place` onto the WASM stack.
    pub(super) fn push_shared_from(&mut self, place: &WasmPlaceRepr) {
        match *place {
            WasmPlaceRepr::Struct(ref fields) => {
                fields.iter().for_each(|r| self.push_shared_from(r));
            }
            WasmPlaceRepr::Class(flags, ref fields) => {
                // Push a 0 for the flags if shared
                self.instructions.push(Instruction::I32Const(0));

                self.push_from_memory(ValType::I32, flags);
                fields.iter().for_each(|r| self.push_shared_from(r));
            }
            WasmPlaceRepr::Local(..) | WasmPlaceRepr::Heap(..) | WasmPlaceRepr::Nowhere => {
                self.push_from(place);
            }
        }
    }

    /// Push a shared copy the value found in `place` onto the WASM stack.
    pub(super) fn push_leased_from(&mut self, place: &WasmPlaceRepr) {
        match *place {
            WasmPlaceRepr::Class(flags, _) => {
                self.push_pointer(flags);
            }
            _ => panic!("can only lease classes"),
        }
    }

    /// Given that a value of type `value_ty` is on the wasm stack, pop it and store it into `to_place`.
    pub(super) fn pop_and_store(&mut self, to_place: &WasmPlaceRepr) {
        match *to_place {
            WasmPlaceRepr::Local(wasm_local, val_type) => self.pop_to_local(val_type, wasm_local),
            WasmPlaceRepr::Heap(wasm_pointer, val_type) => {
                self.pop_to_memory(val_type, wasm_pointer)
            }
            WasmPlaceRepr::Struct(ref fields) => {
                fields.iter().rev().for_each(|r| self.pop_and_store(r));
            }
            WasmPlaceRepr::Class(flags, ref fields) => {
                fields.iter().rev().for_each(|r| self.pop_and_store(r));
                self.pop_to_memory(ValType::I32, flags);
            }
            WasmPlaceRepr::Nowhere => (),
        }
    }

    /// Representation for the place storing a given field found in
    /// an owner of type `owner_ty` that is stored in `owner_place`.
    fn field_place(
        &self,
        owner_place_repr: Arc<WasmPlaceRepr>,
        owner_ty: SymTy<'db>,
        field: SymField<'db>,
    ) -> Arc<WasmPlaceRepr> {
        let db = self.cx.db;
        match owner_ty.kind(db) {
            SymTyKind::Var(sym_variable) => self.field_place(
                owner_place_repr,
                self.generics[&sym_variable].assert_type(db),
                field,
            ),
            SymTyKind::Infer(_) => panic!("unresolved inference variable"),
            SymTyKind::Never | SymTyKind::Error(_) => match &*owner_place_repr {
                WasmPlaceRepr::Nowhere => owner_place_repr,
                _ => panic!("unexpeced place for {owner_ty:?}: {owner_place_repr:?}"),
            },
            SymTyKind::Named(ty_name, _) => match *ty_name {
                SymTyName::Future => match &*owner_place_repr {
                    WasmPlaceRepr::Class(_, vec) => vec[0].clone(),
                    WasmPlaceRepr::Nowhere => owner_place_repr,
                    _ => panic!("unexpeced place for {owner_ty:?}: {owner_place_repr:?}"),
                },
                SymTyName::Primitive(_) => panic!("primitive types do not have fields"),
                SymTyName::Tuple { arity: _ } => todo!(),
                SymTyName::Aggregate(aggr) => {
                    // Where is the owner's data stored?
                    match &*owner_place_repr {
                        WasmPlaceRepr::Struct(fields) | WasmPlaceRepr::Class(_, fields) => {
                            let field_index = aggr
                                .fields(db)
                                .take_while(|f: &SymField<'_>| *f != field)
                                .count();
                            fields[field_index].clone()
                        }
                        WasmPlaceRepr::Nowhere => owner_place_repr,
                        _ => panic!("unexpeced place for {owner_ty:?}: {owner_place_repr:?}"),
                    }
                }
            },
            SymTyKind::Perm(sym_perm, sym_ty) => todo!(),
        }
    }

    /// Returns the representation of a "local" storing a value of type `repr`.
    /// A "local" place is one that uses WASM local variables as much as possible.
    fn emplace_local(&mut self, repr: &WasmRepr) -> Arc<WasmPlaceRepr> {
        match repr {
            WasmRepr::Val(val_type) => {
                let local = self.fresh_local_index(*val_type);
                Arc::new(WasmPlaceRepr::Local(local, *val_type))
            }
            WasmRepr::Struct(vec) => Arc::new(WasmPlaceRepr::Struct(
                vec.iter().map(|r| self.emplace_local(r)).collect(),
            )),
            WasmRepr::Class(_) => self.emplace_memory(repr),
            WasmRepr::Nothing => Arc::new(WasmPlaceRepr::Nowhere),
        }
    }

    /// The representation for a Dada place found in WASM memory
    /// that stores values with representation `repr`.
    fn emplace_memory(&mut self, repr: &WasmRepr) -> Arc<WasmPlaceRepr> {
        match repr {
            WasmRepr::Val(val_type) => {
                let pointer = self.fresh_memory_slot(*val_type);
                Arc::new(WasmPlaceRepr::Heap(pointer, *val_type))
            }
            WasmRepr::Struct(vec) => Arc::new(WasmPlaceRepr::Struct(
                vec.iter().map(|r| self.emplace_memory(r)).collect(),
            )),
            WasmRepr::Class(vec) => {
                let flag_word = self.fresh_memory_slot(ValType::I32);
                Arc::new(WasmPlaceRepr::Class(
                    flag_word,
                    vec.iter().map(|r| self.emplace_memory(r)).collect(),
                ))
            }
            WasmRepr::Nothing => Arc::new(WasmPlaceRepr::Nowhere),
        }
    }

    /// Create a fresh local index storing a value of type `v`.
    fn fresh_local_index(&mut self, v: ValType) -> WasmLocal {
        let index = u32::try_from(self.wasm_locals.len()).expect("too many locals");
        self.wasm_locals.push(v);
        WasmLocal { index }
    }

    /// Create a fresh slot in memory storing a value of type `v`.
    fn fresh_memory_slot(&mut self, v: ValType) -> WasmPointer {
        let offset = self.wasm_stack_frame_size;
        self.wasm_stack_frame_size += val_type_size_in_bytes(v);
        WasmPointer {
            base_variable: self.wasm_stack_pointer,
            offset,
        }
    }

    /// Push a value of type `val_type` found in `local`.
    fn push_from_local(&mut self, val_type: wasm_encoder::ValType, local: WasmLocal) {
        assert_eq!(self.wasm_locals[local.index as usize], val_type);
        self.instructions.push(Instruction::LocalGet(local.index));
    }

    /// Pop a value of type `val_type` and store it in `local`.
    fn pop_to_local(&mut self, v: ValType, local: WasmLocal) {
        assert_eq!(self.wasm_locals[local.index as usize], v);
        self.instructions.push(Instruction::LocalSet(local.index));
    }

    /// Push a value of type `val_type` found in the given memory slot.
    fn push_from_memory(
        &mut self,
        v: ValType,
        WasmPointer {
            base_variable,
            offset,
        }: WasmPointer,
    ) {
        let offset = offset as u64;
        self.instructions.push(match v {
            ValType::I32 => Instruction::I32Load(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::I64 => Instruction::I64Load(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::F32 => Instruction::F32Load(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::F64 => Instruction::F64Load(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::V128 | ValType::Ref(_) => panic!("unexpected val type {v:?}"),
        });
    }

    /// Push a pointer itself onto the WASM stack (not the data it refers to).
    pub(super) fn push_pointer(
        &mut self,
        WasmPointer {
            base_variable,
            offset,
        }: WasmPointer,
    ) {
        self.push_from_local(ValType::I32, base_variable);
        self.instructions.push(Instruction::I32Const(offset as i32));
        self.instructions.push(Instruction::I32Add);
    }

    /// Pop a value of type `val_type` and store it to the given memory slot.
    fn pop_to_memory(
        &mut self,
        v: ValType,
        WasmPointer {
            base_variable,
            offset,
        }: WasmPointer,
    ) {
        let offset = offset as u64;
        self.instructions.push(match v {
            ValType::I32 => Instruction::I32Store(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::I64 => Instruction::I64Store(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::F32 => Instruction::F32Store(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::F64 => Instruction::F64Store(wasm_encoder::MemArg {
                offset,
                align: 4,
                memory_index: base_variable.index,
            }),
            ValType::V128 | ValType::Ref(_) => panic!("unexpected val type {v:?}"),
        });
    }
}

impl WasmRepr {
    /// Primitive WASM values needed for a value with this representation stored on the WASM stack or in memory.
    pub fn flatten(&self) -> Vec<ValType> {
        match self {
            WasmRepr::Val(val_type) => vec![*val_type],

            // Structs are just each field one after the other.
            WasmRepr::Struct(fields) => fields.iter().flat_map(|r| r.local_val_tys()).collect(),

            // Classes begin with an `I32` flag word.
            WasmRepr::Class(fields) => std::iter::once(ValType::I32)
                .chain(fields.iter().flat_map(|r| r.local_val_tys()))
                .collect(),

            WasmRepr::Nothing => vec![],
        }
    }

    /// Returns the types of the WASM local variables that would be used to store a value with this representation.
    /// Any data found inside of a class is stored in memory and hence not represented in the return type.
    pub fn local_val_tys(&self) -> Vec<ValType> {
        match self {
            WasmRepr::Val(val_type) => vec![*val_type],
            WasmRepr::Struct(fields) => fields.iter().flat_map(|r| r.local_val_tys()).collect(),
            WasmRepr::Class(_) => vec![],
            WasmRepr::Nothing => vec![],
        }
    }
}

fn val_type_size_in_bytes(v: ValType) -> u32 {
    match v {
        ValType::I32 => 4,
        ValType::I64 => 8,
        ValType::F32 => 4,
        ValType::F64 => 8,
        ValType::V128 => 16,
        ValType::Ref(_) => panic!("ref values do not have a size in bytes"),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WasmLocal {
    pub index: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct WasmPointer {
    base_variable: WasmLocal,
    offset: u32,
}
