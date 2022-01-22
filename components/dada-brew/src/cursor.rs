use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir,
        validated::{self, ExprOrigin},
    },
    word::SpannedOptionalWord,
};

use crate::brewery::Brewery;

pub(crate) struct Cursor {
    /// The block that we started from; may or may not be "complete".
    start_block: bir::BasicBlock,

    /// The basic block we are currently appending to.
    ///
    /// If `None`, we are in a section of dead code.
    end_block: Option<bir::BasicBlock>,
}

impl Cursor {
    pub(crate) fn new(brewery: &mut Brewery<'_>, origin: ExprOrigin) -> Self {
        let block = brewery.dummy_block(origin);
        Cursor {
            start_block: block,
            end_block: Some(block),
        }
    }

    pub(crate) fn complete(self) -> bir::BasicBlock {
        assert!(self.in_dead_code());
        self.start_block
    }

    pub(crate) fn with_end_block(&self, end_block: bir::BasicBlock) -> Cursor {
        Cursor {
            start_block: self.start_block,
            end_block: Some(end_block),
        }
    }

    pub(crate) fn in_dead_code(&self) -> bool {
        self.end_block.is_none()
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
        target: bir::Place,
        value: bir::ExprData,
        origin: ExprOrigin,
    ) {
        if self.end_block.is_some() {
            let value = brewery.add(value, origin);
            let statement = brewery.add(bir::StatementData::Assign(target, value), origin);
            self.push_statement(brewery, statement);
        }
    }

    /// Push a "cusp" statement onto the current basic block.
    /// These statements indicate the end of the given origin node
    /// in the BIR.
    pub(crate) fn push_cusp(&mut self, brewery: &mut Brewery<'_>, origin: ExprOrigin) {
        if self.end_block.is_some() {
            let statement = brewery.add(bir::StatementData::Cusp, origin);
            self.push_statement(brewery, statement);
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
    ) -> Option<(bir::Place, SpannedOptionalWord)> {
        let validated::NamedExprData { name, expr } = arg.data(brewery.validated_tables());
        let place = self.brew_expr_to_place(brewery, *expr)?;
        Some((place, *name))
    }
}
