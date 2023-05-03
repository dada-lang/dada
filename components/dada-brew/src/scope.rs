use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir, syntax,
        validated::{self, ExprOrigin},
    },
    storage::Atomic,
};

use crate::brewery::Brewery;

/// Tracks the current basic block that we are appending statements to.
pub(crate) struct Scope<'s> {
    /// The block that we started from; may or may not be "complete"
    /// (i.e., may not yet have a terminator assigned to it).
    start_block: bir::BasicBlock,

    /// The basic block we are currently appending to; could be the
    /// same as `start_block`.
    ///
    /// If `None`, we are in a section of dead code.
    end_block: Option<bir::BasicBlock>,

    /// Reason for introducing this scope
    cause: ScopeCause,

    /// Previous scope in the chain
    previous: Option<&'s Scope<'s>>,

    /// Variables that have been introduced in this scope,
    /// whether temporaries or user declared. These variables
    /// need to be popped before the scope is complete.
    ///
    /// See `push_variable_marker` and `pop_variables`.
    variables: Vec<bir::LocalVariable>,
}

/// Reason for introducing a new scope
pub(crate) enum ScopeCause {
    /// Root scope
    Root,

    /// An internal branch (e.g., if-then-else, new block)
    Branch,

    /// Loop introduced that might be target of a break
    Loop(LoopContext),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct LoopContext {
    pub expr: validated::Expr,
    pub continue_block: bir::BasicBlock,
    pub break_block: bir::BasicBlock,
    pub loop_value: bir::TargetPlace,
}

/// Created when we start brewing an expression or other thing that
/// may create temporary values. When the scope is popped, the temporaries
/// are cleared out.
///
/// See the `temporaries` field of [`Brewery`] for more information.
pub(crate) struct VariableMarker {
    mark: usize,
}

impl Scope<'_> {
    /// Creates a new cursor with a dummy starting block.
    pub(crate) fn root(brewery: &mut Brewery<'_>, origin: ExprOrigin) -> Self {
        let block = brewery.dummy_block(origin);
        Scope {
            start_block: block,
            end_block: Some(block),
            cause: ScopeCause::Root,
            previous: None,
            variables: vec![],
        }
    }

    /// Invoked at the end of the method, returns the start block.
    pub(crate) fn complete(self) -> bir::BasicBlock {
        assert!(self.in_dead_code());
        assert!(matches!(self.cause, ScopeCause::Root));
        self.start_block
    }

    /// Creates a subscope with the given `cause` that shares the same start block but is now appending
    /// to `end_block`. It is your reponsibility to connect `end_block` (or some successor of it) back to
    /// `self.end_block` in this subscope.
    pub(crate) fn subscope<'s>(
        &'s self,
        end_block: Option<bir::BasicBlock>,
        cause: ScopeCause,
    ) -> Scope<'s> {
        Scope {
            start_block: self.start_block,
            end_block: end_block,
            cause,
            previous: Some(self),
            variables: vec![],
        }
    }

    /// Test if this cursor is contained in dead code.
    pub(crate) fn in_dead_code(&self) -> bool {
        self.end_block.is_none()
    }

    /// Iterate up the causal chain from all parent scopes
    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        let mut s = Some(self);
        std::iter::from_fn(move || match s {
            Some(s1) => {
                s = s1.previous;
                Some(s1)
            }
            None => None,
        })
    }

    /// Find and return loop context for a given loop expression,
    /// along with a list of variables whose values must be cleared
    /// before breaking or continuing from that loop.
    ///
    /// Panics if that loop context has not been pushed.
    #[track_caller]
    pub fn loop_context(
        &self,
        loop_expr: validated::Expr,
    ) -> (LoopContext, Vec<bir::LocalVariable>) {
        let mut variables = vec![];

        for s in self.scopes() {
            variables.extend(s.variables.iter());
            match &s.cause {
                ScopeCause::Loop(c) if c.expr == loop_expr => {
                    return (*c, variables);
                }
                _ => {}
            }
        }

        panic!("malformed IR: loop expr {loop_expr:?} not in scope")
    }

    /// Create a temporary variable and push it into this scope; it will be cleared
    /// when the surrounding `clear_variables_since_marker` is invoked.
    pub fn add_temporary(&mut self, brewery: &mut Brewery, origin: ExprOrigin) -> bir::TargetPlace {
        let temporary = brewery.add(
            bir::LocalVariableData {
                name: None,
                atomic: Atomic::No,
            },
            validated::LocalVariableOrigin::Temporary(origin.into()),
        );
        tracing::debug!("created temporary: temp{{{:?}}}", u32::from(temporary));
        self.variables.push(temporary);
        brewery.add(bir::TargetPlaceData::LocalVariable(temporary), origin)
    }

    /// Pushes user-declared variables into scope. Returns a new [`VariableMarker`][]
    /// that should be freed by a call to [`Self::pop_variables_since_marker`][]
    /// to clear the declared variables.
    pub(crate) fn push_declared_variables(
        &mut self,
        vars: &[validated::LocalVariable],
        brewery: &mut Brewery<'_>,
    ) -> VariableMarker {
        let marker = self.mark_variables();
        for &v in vars {
            self.variables.push(brewery.variable(v));
        }
        marker
    }

    /// Record the set of declared variables; must be paired
    /// with a call to `pop_variables_since_marker`
    /// that will clear all variables (temporary or declared)
    /// pushed since the marker was created.
    pub(crate) fn mark_variables(&self) -> VariableMarker {
        VariableMarker {
            mark: self.variables.len(),
        }
    }

    /// Clears all variables (temporary or declared) pushed
    /// since the marker was created. Clears of temporaries
    /// will be given an origin based on the expression that they
    /// were synthesized from; clears of local variables use `origin`.
    pub(crate) fn clear_variables_since_marker(
        &mut self,
        marker: VariableMarker,
        brewery: &mut Brewery<'_>,
        origin: ExprOrigin,
    ) {
        assert!(marker.mark <= self.variables.len());
        while self.variables.len() > marker.mark {
            let var = self.variables.pop().unwrap();
            let clear_origin = match brewery.bir_origin(var) {
                validated::LocalVariableOrigin::Temporary(expr) => ExprOrigin::synthesized(expr),
                validated::LocalVariableOrigin::LocalVariable(_)
                | validated::LocalVariableOrigin::Parameter(_) => origin,
            };
            self.push_clear_variable(brewery, var, clear_origin);
        }
    }

    /// Push clear statements for `variables` onto the current block.
    pub(crate) fn push_clear_variables(
        &mut self,
        brewery: &mut Brewery<'_>,
        variables: &[bir::LocalVariable],
        origin: ExprOrigin,
    ) {
        for &variable in variables {
            self.push_clear_variable(brewery, variable, origin);
        }
    }

    /// Push a clear statement for `variable` onto the current block.
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
