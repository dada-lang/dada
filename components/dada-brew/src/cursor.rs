use dada_id::prelude::*;
use dada_ir::code::{bir, syntax, validated};

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
    pub(crate) fn new(brewery: &mut Brewery<'_>, origin: syntax::Expr) -> Self {
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
        origin: syntax::Expr,
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
        origin: syntax::Expr,
    ) {
        self.terminate(brewery, terminator_data, origin, None);
    }

    pub(crate) fn terminate_and_continue(
        &mut self,
        brewery: &mut Brewery<'_>,
        terminator_data: impl FnOnce(bir::BasicBlock) -> bir::TerminatorData,
        origin: syntax::Expr,
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
        origin: syntax::Expr,
    ) {
        if let Some(_) = self.end_block {
            let value = brewery.add(value, origin);
            let statement = brewery.add(bir::StatementData::Assign(target, value), origin);
            self.push_statement(brewery, statement);
        }
    }

    pub(crate) fn terminate_and_goto(
        &mut self,
        brewery: &mut Brewery<'_>,
        target: bir::BasicBlock,
        origin: syntax::Expr,
    ) {
        self.terminate_and_diverge(brewery, bir::TerminatorData::Goto(target), origin);
    }

    pub(crate) fn brew_named_expr(
        &mut self,
        brewery: &mut Brewery,
        arg: validated::NamedExpr,
    ) -> Option<bir::NamedPlace> {
        let origin = brewery.origin(arg);
        let validated::NamedExprData { name, expr } = arg.data(brewery.validated_tables());
        let place = self.brew_expr_to_place(brewery, *expr)?;
        Some(brewery.add(bir::NamedPlaceData { name: *name, place }, origin))
    }
}
