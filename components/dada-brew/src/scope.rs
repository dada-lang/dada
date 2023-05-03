use dada_id::prelude::*;
use dada_ir::code::{
    bir, syntax,
    validated::{self, ExprOrigin},
};

use crate::brewery::Brewery;

/// Tracks the current basic block that we are appending statements to.
pub(crate) struct Scope {
    /// The block that we started from; may or may not be "complete"
    /// (i.e., may not yet have a terminator assigned to it).
    start_block: bir::BasicBlock,

    /// The basic block we are currently appending to; could be the
    /// same as `start_block`.
    ///
    /// If `None`, we are in a section of dead code.
    end_block: Option<bir::BasicBlock>,
}

/// Created when we start brewing an expression or other thing that
/// may create temporary values. When the scope is popped, the temporaries
/// are cleared out.
///
/// See the `temporaries` field of [`Brewery`] for more information.
pub(crate) struct TemporaryScope {
    mark: usize,
}

impl Scope {
    /// Creates a new cursor with a dummy starting block.
    pub(crate) fn new(brewery: &mut Brewery<'_>, origin: ExprOrigin) -> Self {
        let block = brewery.dummy_block(origin);
        Scope {
            start_block: block,
            end_block: Some(block),
        }
    }

    /// Invoked at the end of the method, returns the start block.
    pub(crate) fn complete(self) -> bir::BasicBlock {
        assert!(self.in_dead_code());
        self.start_block
    }

    /// Creates a new cursor that shares the same start block but is now appending
    /// to `end_block`.
    pub(crate) fn with_end_block(&self, end_block: bir::BasicBlock) -> Scope {
        Scope {
            start_block: self.start_block,
            end_block: Some(end_block),
        }
    }

    /// Test if this cursor is contained in dead code.
    pub(crate) fn in_dead_code(&self) -> bool {
        self.end_block.is_none()
    }

    /// Creates a temporary scope marker that tracks the current number of temporaries;
    /// the return value should later be given to `pop_temporary_scope`.
    pub(crate) fn push_temporary_scope(&self, brewery: &mut Brewery<'_>) -> TemporaryScope {
        TemporaryScope {
            mark: brewery.temporary_stack_len(),
        }
    }

    /// Pops all temporaries pushed since `scope` was created from the stack and inserts
    /// "clear variable" instructions.
    pub(crate) fn pop_temporary_scope(&mut self, brewery: &mut Brewery<'_>, scope: TemporaryScope) {
        while brewery.temporary_stack_len() > scope.mark {
            let temporary = brewery.pop_temporary();
            let origin = match brewery.bir_origin(temporary) {
                validated::LocalVariableOrigin::Temporary(expr) => ExprOrigin::synthesized(expr),
                validated::LocalVariableOrigin::LocalVariable(_)
                | validated::LocalVariableOrigin::Parameter(_) => {
                    panic!("BIR temporaries should not originate from locals or parameters")
                }
            };
            self.push_clear_variable(brewery, temporary, origin);
        }
    }

    /// Pushes clear instructions for each of the given variables.
    pub(crate) fn pop_declared_variables(
        &mut self,
        brewery: &mut Brewery<'_>,
        vars: &[validated::LocalVariable],
        origin: ExprOrigin,
    ) {
        for var in vars {
            let bir_var = brewery.variable(*var);
            self.push_clear_variable(brewery, bir_var, origin);
        }
    }

    fn push_clear_variable(
        &mut self,
        brewery: &mut Brewery<'_>,
        variable: bir::LocalVariable,
        origin: ExprOrigin,
    ) {
        let statement = brewery.add(bir::StatementData::Clear(variable), origin);
        self.push_statement(brewery, statement)
    }

    pub(crate) fn push_statement(&mut self, brewery: &mut Brewery<'_>, statement: bir::Statement) {
        if let Some(end_block) = self.end_block {
            brewery[end_block].statements.push(statement);
        }
    }

    fn terminate(
        &mut self,
        brewery: &mut Brewery<'_>,
        terminator_data: bir::TerminatorData,
        origin: ExprOrigin,
        next_block: Option<bir::BasicBlock>,
    ) {
        if let Some(end_block) = self.end_block {
            let terminator = brewery.add(terminator_data, origin);
            brewery[end_block].terminator = terminator;
            brewery[end_block].statements.shrink_to_fit();
            self.end_block = next_block;
        }
    }

    pub(crate) fn terminate_and_diverge(
        &mut self,
        brewery: &mut Brewery<'_>,
        terminator_data: bir::TerminatorData,
        origin: ExprOrigin,
    ) {
        self.terminate(brewery, terminator_data, origin, None);
    }

    pub(crate) fn terminate_and_continue(
        &mut self,
        brewery: &mut Brewery<'_>,
        terminator_data: impl FnOnce(bir::BasicBlock) -> bir::TerminatorData,
        origin: ExprOrigin,
    ) -> bir::BasicBlock {
        let next_block = brewery.dummy_block(origin);
        let terminator_data = terminator_data(next_block);
        self.terminate(brewery, terminator_data, origin, Some(next_block));
        next_block
    }

    pub(crate) fn push_assignment(
        &mut self,
        brewery: &mut Brewery<'_>,
        target: bir::TargetPlace,
        value: bir::ExprData,
        origin: ExprOrigin,
    ) {
        if self.end_block.is_some() {
            let value = brewery.add(value, origin);
            let statement = brewery.add(bir::StatementData::AssignExpr(target, value), origin);
            self.push_statement(brewery, statement);
        }
    }

    /// If any of the origins in `origins`, or `origin`, is a breakpoint expression,
    /// push a "breakpoint-start" statement onto the current basic block.
    pub(crate) fn push_breakpoint_starts(
        &mut self,
        brewery: &mut Brewery<'_>,
        origins: impl IntoIterator<Item = ExprOrigin>,
        origin: ExprOrigin,
    ) {
        for o in origins.into_iter().chain(Some(origin)) {
            self.push_breakpoint_start(brewery, o);
        }
    }

    /// If `origin` is a breakpoint expression, push a "breakpoint-start"
    /// statement onto the current basic block.
    pub(crate) fn push_breakpoint_start(&mut self, brewery: &mut Brewery<'_>, origin: ExprOrigin) {
        tracing::debug!(
            "push_breakpoint_start: origin={:?} breakpoints={:?}",
            origin,
            brewery.breakpoints
        );
        if !origin.synthesized && self.end_block.is_some() {
            if let Some(breakpoint_index) = brewery.expr_is_breakpoint(origin.syntax_expr) {
                let input_file = brewery.input_file();
                let statement = brewery.add(
                    bir::StatementData::BreakpointStart(input_file, breakpoint_index),
                    origin,
                );
                self.push_statement(brewery, statement);
            }
        }
    }

    /// Push breakpoint-end expressions for each origin in `origins`.
    /// Used when evaluating something like `a.b.c.give`:
    /// the cusps for `a`, `a.b`, and `a.b.c` are all emitted
    /// simultaneously with the final value.
    pub(crate) fn push_breakpoint_ends(
        &mut self,
        brewery: &mut Brewery<'_>,
        place: Option<impl AnyPlace>,
        origins: impl IntoIterator<Item = ExprOrigin>,
        origin: ExprOrigin,
    ) {
        let place = place.map(|p| p.into_place(brewery));
        for o in origins.into_iter().chain(Some(origin)) {
            self.push_breakpoint_end_with_distinct_origin(brewery, origin.syntax_expr, place, o);
        }
    }
    /// Push a "breakpoint-end" statement onto the current basic block.
    /// These statements indicate the end of the given origin node
    /// in the BIR.
    pub(crate) fn push_breakpoint_end(
        &mut self,
        brewery: &mut Brewery<'_>,
        place: Option<impl AnyPlace>,
        origin: ExprOrigin,
    ) {
        let place = place.map(|p| p.into_place(brewery));
        self.push_breakpoint_end_with_distinct_origin(brewery, origin.syntax_expr, place, origin);
    }

    /// Helper: push the breakpoint-end but the origin/breakpoint-expr might be distinct.
    fn push_breakpoint_end_with_distinct_origin(
        &mut self,
        brewery: &mut Brewery<'_>,
        expr: syntax::Expr,
        place: Option<bir::Place>,
        origin: ExprOrigin,
    ) {
        if !origin.synthesized && self.end_block.is_some() {
            if let Some(breakpoint_index) = brewery.expr_is_breakpoint(origin.syntax_expr) {
                let input_file = brewery.input_file();
                let statement = brewery.add(
                    bir::StatementData::BreakpointEnd(input_file, breakpoint_index, expr, place),
                    origin,
                );
                self.push_statement(brewery, statement);
            }
        }
    }

    pub(crate) fn terminate_and_goto(
        &mut self,
        brewery: &mut Brewery<'_>,
        target: bir::BasicBlock,
        origin: ExprOrigin,
    ) {
        self.terminate_and_diverge(brewery, bir::TerminatorData::Goto(target), origin);
    }

    pub(crate) fn brew_named_expr(
        &mut self,
        brewery: &mut Brewery,
        arg: validated::NamedExpr,
    ) -> Option<(bir::Place, Option<bir::Name>)> {
        let validated::NamedExprData { name, expr } = arg.data(brewery.validated_tables());
        let name = name.map(|n| self.brew_name(brewery, n));
        let place = self.brew_expr_to_temporary(brewery, *expr)?;
        Some((place, name))
    }

    fn brew_name(&mut self, brewery: &mut Brewery, name: validated::Name) -> bir::Name {
        let validated::NameData { word } = name.data(brewery.validated_tables());
        let origin = brewery.origin(name);
        brewery.add(bir::NameData { word: *word }, origin)
    }
}

pub(crate) trait AnyPlace {
    fn into_place(self, brewery: &mut Brewery<'_>) -> bir::Place;
}

impl AnyPlace for bir::Place {
    fn into_place(self, _brewery: &mut Brewery<'_>) -> bir::Place {
        self
    }
}

impl AnyPlace for bir::TargetPlace {
    fn into_place(self, brewery: &mut Brewery<'_>) -> bir::Place {
        brewery.place_from_target_place(self)
    }
}
