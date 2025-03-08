use std::sync::Arc;

use dada_ir_ast::{ast::PermissionOp, diagnostic::Reported};
use dada_ir_sym::ir::exprs::{SymBinaryOp, SymExpr, SymExprKind, SymLiteral, SymMatchArm};
use dada_ir_sym::ir::red::RedInfers;
use dada_ir_sym::ir::types::{SymGenericTerm, SymTy, SymTyKind};
use dada_ir_sym::{
    ir::primitive::SymPrimitiveKind, ir::subst::Subst, ir::types::SymTyName,
    ir::variables::SymVariable,
};
use dada_util::Map;
use wasm_encoder::{Instruction, ValType};
use wasm_place_repr::{WasmLocal, WasmPlaceRepr};

use super::wasm_repr::WasmReprCx;
use super::{Cx, wasm_repr::WasmRepr};

pub(crate) mod wasm_place_repr;

pub(crate) struct ExprCodegen<'cx, 'db> {
    cx: &'cx mut Cx<'db>,

    /// Values of any generic variables
    generics: Map<SymVariable<'db>, SymGenericTerm<'db>>,

    /// Values for any inference variables
    infers: RedInfers<'db>,

    /// Accumulates wasm locals. We make no effort to reduce the number of local variables created.
    wasm_locals: Vec<wasm_encoder::ValType>,

    /// Local variable that stores starting address in our stack frame
    wasm_stack_pointer: WasmLocal,

    /// Values we are putting onto the stack frame (actually located in the WASM heap)
    wasm_stack_frame_size: u32,

    /// Maps each Dada variable to a range of wasm locals. Note that a single value can be inlined into multiple wasm locals.
    variables: Map<SymVariable<'db>, Arc<WasmPlaceRepr>>,

    /// Accumulates wasm instructions.
    instructions: Vec<Instruction<'static>>,
}

impl<'cx, 'db> ExprCodegen<'cx, 'db> {
    pub fn new(
        cx: &'cx mut Cx<'db>,
        generics: Map<SymVariable<'db>, SymGenericTerm<'db>>,
        infers: RedInfers<'db>,
    ) -> Self {
        // Initially there is one local variable, the stack pointer.
        Self {
            cx,
            generics,
            infers,
            wasm_locals: vec![ValType::I32],
            variables: Default::default(),
            instructions: Default::default(),
            wasm_stack_frame_size: 0,
            wasm_stack_pointer: WasmLocal { index: 0 },
        }
    }

    pub fn into_function(self) -> wasm_encoder::Function {
        let mut f = wasm_encoder::Function::new_with_locals_types(self.wasm_locals);
        for instruction in self.instructions {
            f.instruction(&instruction);
        }
        f
    }

    /// Returns the [`WasmRepr`][] for a Dada type.
    pub fn wasm_repr_of_type(&self, ty: SymTy<'db>) -> WasmRepr {
        let db = self.cx.db;
        let mut wrcx = WasmReprCx::new(db, &self.generics);
        wrcx.wasm_repr_of_type(ty)
    }

    pub fn pop_arguments(&mut self, inputs: &[SymVariable<'db>], input_tys: &[SymTy<'db>]) {
        assert_eq!(inputs.len(), input_tys.len());
        for (&input, &input_ty) in inputs.iter().zip(input_tys).rev() {
            self.insert_variable(input, input_ty);
            self.pop_and_store(&self.place_for_local(input));
        }
        self.instructions
            .push(Instruction::LocalSet(self.wasm_stack_pointer.index));
    }

    /// Generate code to execute the expression, leaving the result on the top of the wasm stack.
    pub fn push_expr(&mut self, expr: SymExpr<'db>) {
        let db = self.cx.db;
        match *expr.kind(db) {
            SymExprKind::Semi(object_expr, object_expr1) => {
                self.push_expr(object_expr);
                self.pop_and_drop(object_expr.ty(db));
                self.push_expr(object_expr1);
            }
            SymExprKind::Tuple(ref elements) => {
                // the representation of a tuple is inlined onto the stack (like any other struct type)
                for &element in elements {
                    self.push_expr(element);
                }
            }
            SymExprKind::Primitive(literal) => self.push_literal(expr.ty(db), literal),
            SymExprKind::LetIn {
                lv,
                ty,
                initializer,
                body,
            } => {
                self.insert_variable(lv, ty);

                if let Some(initializer) = initializer {
                    self.push_expr(initializer);
                    self.pop_and_store(&self.variables[&lv].clone());
                } else {
                    // FIXME: should zero out the values
                }

                self.push_expr(body);
            }
            SymExprKind::Await {
                future,
                await_keyword: _,
            } => {
                self.push_expr(future);
                // FIXME: for now we just ignore futures and execute everything synchronously
            }
            SymExprKind::Assign { place, value } => {
                let wasm_place = self.place(place);
                self.push_expr(value);

                // FIXME: have to drop the old value

                self.pop_and_store(&wasm_place);
            }
            SymExprKind::PermissionOp(permission_op, object_place_expr) => {
                let wasm_place_repr = self.place(object_place_expr);
                match permission_op {
                    PermissionOp::Lease => {
                        self.push_leased_from(&wasm_place_repr);
                    }

                    PermissionOp::Share => {
                        self.push_shared_from(&wasm_place_repr);
                    }

                    PermissionOp::Give => {
                        self.push_from(&wasm_place_repr);
                    }
                }
            }
            SymExprKind::Call {
                function,
                ref substitution,
                ref arg_temps,
            } => {
                let fn_args = substitution.subst_vars(db, &self.generics);
                let fn_index = self.cx.declare_fn(function, fn_args);

                // First push the stack pointer for the new function;
                self.push_pointer(self.next_stack_frame());

                // Now push each of the arguments in turn.
                for arg_temp in arg_temps {
                    let place = self.variables[arg_temp].clone();
                    self.push_from(&place);
                }

                self.instructions.push(Instruction::Call(fn_index.0));
            }
            SymExprKind::Return(object_expr) => {
                self.push_expr(object_expr);
                self.instructions.push(Instruction::Return);
            }
            SymExprKind::Not {
                operand,
                op_span: _,
            } => {
                self.push_expr(operand);
                self.instructions.push(Instruction::I32Const(1));
                self.instructions.push(Instruction::I32Xor);
            }
            SymExprKind::BinaryOp(binary_op, object_expr, object_expr1) => {
                self.push_expr(object_expr);
                self.push_expr(object_expr1);
                self.execute_binary_op(binary_op, object_expr.ty(db), object_expr.ty(db));
            }
            SymExprKind::Aggregate { ty, ref fields } => {
                let wasm_repr = self.wasm_repr_of_type(ty);
                match wasm_repr {
                    WasmRepr::Struct(field_reprs) => {
                        assert_eq!(fields.len(), field_reprs.len());
                        for &field in fields {
                            self.push_expr(field);
                        }
                    }
                    WasmRepr::Class(field_reprs) => {
                        assert_eq!(fields.len(), field_reprs.len());

                        // push flag word
                        self.instructions.push(Instruction::I32Const(1));

                        for &field in fields {
                            self.push_expr(field);
                        }
                    }
                    WasmRepr::Val(_) | WasmRepr::Nothing => {
                        panic!("not an aggregate: {ty:?}")
                    }
                }
            }
            SymExprKind::Match { ref arms } => {
                self.push_match_expr(expr.ty(db), arms);
            }
            SymExprKind::Error(reported) => self.push_error(reported),
            #[expect(unused_variables)]
            SymExprKind::ByteLiteral(sym_byte_literal) => todo!(),
        }
    }

    fn pop_and_drop(&mut self, _of_type: SymTy<'db>) {
        // currently everything is stack allocated, no dropping required
    }

    pub(super) fn pop_and_return(&mut self, _of_type: SymTy<'db>) {
        self.instructions.push(Instruction::Return);
    }

    /// Push the correct instructions to execute `binary_op` on operands of type `lhs_ty` and `rhs_ty`
    fn execute_binary_op(
        &mut self,
        binary_op: SymBinaryOp,
        lhs_ty: SymTy<'db>,
        rhs_ty: SymTy<'db>,
    ) {
        match self.primitive_kind(lhs_ty) {
            Ok(prim_kind) => {
                assert_eq!(self.primitive_kind(rhs_ty), Ok(prim_kind));
                self.execute_binary_op_on_primitives(binary_op, prim_kind)
            }
            Err(e) => match e {
                NotPrimitive::DeadCode => (),
                NotPrimitive::OtherType => panic!(
                    "don't know how to execute a binary op on ({:?}, {:?})",
                    lhs_ty, rhs_ty
                ),
            },
        }
    }

    /// Push the correct instructions to execute `binary_op` on operands of type `prim_kind`
    fn execute_binary_op_on_primitives(
        &mut self,
        binary_op: SymBinaryOp,
        prim_kind: SymPrimitiveKind,
    ) {
        let instruction = match (prim_kind, binary_op) {
            (SymPrimitiveKind::Char, SymBinaryOp::Add)
            | (SymPrimitiveKind::Char, SymBinaryOp::Sub)
            | (SymPrimitiveKind::Char, SymBinaryOp::Mul)
            | (SymPrimitiveKind::Char, SymBinaryOp::Div)
            | (SymPrimitiveKind::Bool, SymBinaryOp::Add)
            | (SymPrimitiveKind::Bool, SymBinaryOp::Sub)
            | (SymPrimitiveKind::Bool, SymBinaryOp::Mul)
            | (SymPrimitiveKind::Bool, SymBinaryOp::Div) => {
                panic!("invalid primitive binary op: {binary_op:?}, {prim_kind:?}")
            }

            (SymPrimitiveKind::Char, SymBinaryOp::GreaterThan)
            | (SymPrimitiveKind::Bool, SymBinaryOp::GreaterThan) => Instruction::I32GtU,

            (SymPrimitiveKind::Char, SymBinaryOp::LessThan)
            | (SymPrimitiveKind::Bool, SymBinaryOp::LessThan) => Instruction::I32LtU,

            (SymPrimitiveKind::Char, SymBinaryOp::GreaterEqual)
            | (SymPrimitiveKind::Bool, SymBinaryOp::GreaterEqual) => Instruction::I32GeU,

            (SymPrimitiveKind::Char, SymBinaryOp::LessEqual)
            | (SymPrimitiveKind::Bool, SymBinaryOp::LessEqual) => Instruction::I32GeU,

            (SymPrimitiveKind::Char, SymBinaryOp::EqualEqual)
            | (SymPrimitiveKind::Bool, SymBinaryOp::EqualEqual) => Instruction::I32Eq,

            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Add) if bits <= 32 => Instruction::I32Add,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Sub) if bits <= 32 => Instruction::I32Sub,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Mul) if bits <= 32 => Instruction::I32Mul,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Div) if bits <= 32 => {
                Instruction::I32DivS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::GreaterThan) if bits <= 32 => {
                Instruction::I32GtS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::LessThan) if bits <= 32 => {
                Instruction::I32LtS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::GreaterEqual) if bits <= 32 => {
                Instruction::I32GeS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::LessEqual) if bits <= 32 => {
                Instruction::I32LeS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::EqualEqual) if bits <= 32 => {
                Instruction::I32Eq
            }

            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Add) if bits <= 64 => Instruction::I64Add,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Sub) if bits <= 64 => Instruction::I64Sub,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Mul) if bits <= 64 => Instruction::I64Mul,
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::Div) if bits <= 64 => {
                Instruction::I64DivS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::GreaterThan) if bits <= 64 => {
                Instruction::I64GtS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::LessThan) if bits <= 64 => {
                Instruction::I64LtS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::GreaterEqual) if bits <= 64 => {
                Instruction::I64GeS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::LessEqual) if bits <= 64 => {
                Instruction::I64LeS
            }
            (SymPrimitiveKind::Int { bits }, SymBinaryOp::EqualEqual) if bits <= 64 => {
                Instruction::I64Eq
            }

            (SymPrimitiveKind::Isize, SymBinaryOp::Add) => Instruction::I32Add,
            (SymPrimitiveKind::Isize, SymBinaryOp::Sub) => Instruction::I32Sub,
            (SymPrimitiveKind::Isize, SymBinaryOp::Mul) => Instruction::I32Mul,
            (SymPrimitiveKind::Isize, SymBinaryOp::Div) => Instruction::I32DivS,
            (SymPrimitiveKind::Isize, SymBinaryOp::GreaterThan) => Instruction::I32GtS,
            (SymPrimitiveKind::Isize, SymBinaryOp::LessThan) => Instruction::I32LtS,
            (SymPrimitiveKind::Isize, SymBinaryOp::GreaterEqual) => Instruction::I32GeS,
            (SymPrimitiveKind::Isize, SymBinaryOp::LessEqual) => Instruction::I32LeS,
            (SymPrimitiveKind::Isize, SymBinaryOp::EqualEqual) => Instruction::I32Eq,

            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Add) if bits <= 32 => {
                Instruction::I32Add
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Sub) if bits <= 32 => {
                Instruction::I32Sub
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Mul) if bits <= 32 => {
                Instruction::I32Mul
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Div) if bits <= 32 => {
                Instruction::I32DivU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::GreaterThan) if bits <= 32 => {
                Instruction::I32GtU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::LessThan) if bits <= 32 => {
                Instruction::I32LtU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::GreaterEqual) if bits <= 32 => {
                Instruction::I32GeU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::LessEqual) if bits <= 32 => {
                Instruction::I32LeU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::EqualEqual) if bits <= 32 => {
                Instruction::I32Eq
            }

            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Add) if bits <= 64 => {
                Instruction::I64Add
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Sub) if bits <= 64 => {
                Instruction::I64Sub
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Mul) if bits <= 64 => {
                Instruction::I64Mul
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::Div) if bits <= 64 => {
                Instruction::I64DivU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::GreaterThan) if bits <= 64 => {
                Instruction::I64GtU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::LessThan) if bits <= 64 => {
                Instruction::I64LtU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::GreaterEqual) if bits <= 64 => {
                Instruction::I64GeU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::LessEqual) if bits <= 64 => {
                Instruction::I64LeU
            }
            (SymPrimitiveKind::Uint { bits }, SymBinaryOp::EqualEqual) if bits <= 64 => {
                Instruction::I64Eq
            }

            (SymPrimitiveKind::Usize, SymBinaryOp::Add) => Instruction::I32Add,
            (SymPrimitiveKind::Usize, SymBinaryOp::Sub) => Instruction::I32Sub,
            (SymPrimitiveKind::Usize, SymBinaryOp::Mul) => Instruction::I32Mul,
            (SymPrimitiveKind::Usize, SymBinaryOp::Div) => Instruction::I32DivU,
            (SymPrimitiveKind::Usize, SymBinaryOp::GreaterThan) => Instruction::I32GtU,
            (SymPrimitiveKind::Usize, SymBinaryOp::LessThan) => Instruction::I32LtU,
            (SymPrimitiveKind::Usize, SymBinaryOp::GreaterEqual) => Instruction::I32GeU,
            (SymPrimitiveKind::Usize, SymBinaryOp::LessEqual) => Instruction::I32LeU,
            (SymPrimitiveKind::Usize, SymBinaryOp::EqualEqual) => Instruction::I32Eq,

            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Add) if bits <= 32 => {
                Instruction::F32Add
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Sub) if bits <= 32 => {
                Instruction::F32Sub
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Mul) if bits <= 32 => {
                Instruction::F32Mul
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Div) if bits <= 32 => {
                Instruction::F32Div
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::GreaterThan) if bits <= 32 => {
                Instruction::F32Gt
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::LessThan) if bits <= 32 => {
                Instruction::F32Lt
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::GreaterEqual) if bits <= 32 => {
                Instruction::F32Ge
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::LessEqual) if bits <= 32 => {
                Instruction::F32Le
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::EqualEqual) if bits <= 32 => {
                Instruction::F32Eq
            }

            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Add) if bits <= 64 => {
                Instruction::F64Add
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Sub) if bits <= 64 => {
                Instruction::F64Sub
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Mul) if bits <= 64 => {
                Instruction::F64Mul
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::Div) if bits <= 64 => {
                Instruction::F64Div
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::GreaterThan) if bits <= 64 => {
                Instruction::F64Gt
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::LessThan) if bits <= 64 => {
                Instruction::F64Lt
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::GreaterEqual) if bits <= 64 => {
                Instruction::F64Ge
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::LessEqual) if bits <= 64 => {
                Instruction::F64Le
            }
            (SymPrimitiveKind::Float { bits }, SymBinaryOp::EqualEqual) if bits <= 64 => {
                Instruction::F64Eq
            }

            (SymPrimitiveKind::Int { bits: _ }, _)
            | (SymPrimitiveKind::Uint { bits: _ } | SymPrimitiveKind::Float { bits: _ }, _) => {
                panic!("invalid number of bits for scalar: {prim_kind:?}")
            }
        };

        self.instructions.push(instruction);
    }

    /// Return the primitive kind that represents `ty` or `Err` if `ty` is not a primitive.
    fn primitive_kind(&self, ty: SymTy<'db>) -> Result<SymPrimitiveKind, NotPrimitive> {
        let db = self.cx.db;
        match ty.kind(db) {
            SymTyKind::Named(ty_name, _ty_args) => match ty_name {
                SymTyName::Primitive(sym_primitive) => Ok(sym_primitive.kind(db)),
                SymTyName::Aggregate(_) | SymTyName::Future | SymTyName::Tuple { arity: _ } => {
                    Err(NotPrimitive::OtherType)
                }
            },
            SymTyKind::Var(sym_variable) => {
                self.primitive_kind(self.generics[sym_variable].assert_type(db))
            }
            SymTyKind::Never | SymTyKind::Error(_) => Err(NotPrimitive::DeadCode),
            SymTyKind::Infer(_) => panic!("unexpected inference variable"),
            #[expect(unused_variables)]
            SymTyKind::Perm(sym_perm, sym_ty) => todo!(),
        }
    }

    fn push_match_expr(&mut self, match_ty: SymTy<'db>, arms: &[SymMatchArm<'db>]) {
        let Some((if_arm, else_arms)) = arms.split_first() else {
            return;
        };

        if let Some(condition) = if_arm.condition {
            // Evaluate the condition.
            self.push_expr(condition);

            // The `If` block will execute the next set of instructions
            // if the condition was true. Otherwise it will skip to the `Else` or `End.`
            let block_type = self.block_type(match_ty);
            self.instructions.push(Instruction::If(block_type));

            // Code to execute if true.
            self.push_expr(if_arm.body);

            // If false push an `Else` and evaluate it recursively.
            self.instructions.push(Instruction::Else);
            self.push_match_expr(match_ty, else_arms);

            // End the if.
            self.instructions.push(Instruction::End);
        } else {
            // Execute body unconditionally.
            self.push_expr(if_arm.body);

            // Any remaining arms are ignored.
            let _ = else_arms;
        }
    }

    /// [Block control-flow instructions][cfi] like `if` and friends
    /// come equipped with an associated "block type". This is a function
    /// type indicating the *inputs* they consume from the stack (in our case,
    /// always none) and the *outputs* they produce. As a shorthand, if they produce
    /// nothing or a single value, there is a shorthand form. This function converts
    /// an object-type into this form.
    ///
    /// [cfi]: https://webassembly.github.io/spec/core/syntax/instructions.html#control-instructions
    fn block_type(&mut self, match_ty: SymTy<'db>) -> wasm_encoder::BlockType {
        let val_types = self.wasm_repr_of_type(match_ty).flatten();
        match val_types.len() {
            0 => wasm_encoder::BlockType::Empty,
            1 => wasm_encoder::BlockType::Result(val_types[0]),
            _ => wasm_encoder::BlockType::FunctionType(u32::from(
                self.cx.declare_fn_type(vec![], val_types),
            )),
        }
    }

    fn push_literal(&mut self, ty: SymTy<'db>, literal: SymLiteral) {
        let db = self.cx.db;
        let kind = match ty.kind(db) {
            SymTyKind::Named(sym_ty_name, _) => match sym_ty_name {
                SymTyName::Primitive(sym_primitive) => sym_primitive.kind(db),
                SymTyName::Aggregate(_) | SymTyName::Future | SymTyName::Tuple { arity: _ } => {
                    panic!("unexpected type for literal {literal:?}: {ty:?}")
                }
            },
            SymTyKind::Var(sym_variable) => {
                return self.push_literal(self.generics[sym_variable].assert_type(db), literal);
            }
            SymTyKind::Infer(_) | SymTyKind::Never => {
                panic!("unexpected type for literal {literal:?}: {ty:?}")
            }
            SymTyKind::Error(reported) => {
                return self.push_error(*reported);
            }
            #[expect(unused_variables)]
            SymTyKind::Perm(sym_perm, sym_ty) => todo!(),
        };
        match kind {
            SymPrimitiveKind::Bool
            | SymPrimitiveKind::Isize
            | SymPrimitiveKind::Usize
            | SymPrimitiveKind::Char => {
                let SymLiteral::Integral { bits } = literal else {
                    panic!("expected integral {literal:?}");
                };
                self.instructions.push(Instruction::I32Const(bits as i32));
            }
            SymPrimitiveKind::Int { bits } | SymPrimitiveKind::Uint { bits } if bits <= 32 => {
                let SymLiteral::Integral { bits } = literal else {
                    panic!("expected integral {literal:?}");
                };
                self.instructions.push(Instruction::I32Const(bits as i32));
            }
            SymPrimitiveKind::Int { bits } | SymPrimitiveKind::Uint { bits } if bits <= 64 => {
                let SymLiteral::Integral { bits } = literal else {
                    panic!("expected integral {literal:?}");
                };
                self.instructions.push(Instruction::I64Const(bits as i64));
            }
            SymPrimitiveKind::Float { bits } if bits <= 32 => {
                let SymLiteral::Float { bits } = literal else {
                    panic!("expected float {literal:?}");
                };
                self.instructions.push(Instruction::F32Const(bits.0 as f32));
            }
            SymPrimitiveKind::Float { bits } if bits <= 32 => {
                let SymLiteral::Float { bits } = literal else {
                    panic!("expected float {literal:?}");
                };
                self.instructions.push(Instruction::F64Const(bits.0));
            }
            SymPrimitiveKind::Int { .. }
            | SymPrimitiveKind::Uint { .. }
            | SymPrimitiveKind::Float { .. } => {
                panic!("unexpected kind: {kind:?}");
            }
        }
    }

    fn push_error(&mut self, _reported: Reported) {
        self.instructions.push(Instruction::Unreachable);
    }
}

/// Error `enum` for [`ExprCodegen::primitive_kind`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum NotPrimitive {
    DeadCode,
    OtherType,
}
