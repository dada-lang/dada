//! This code asserts various invariants that ought to hold across the machine state.
//! A failed assertion here represents a bug in dada's type system or operational semantics
//! and thus ought never to occur.
//!
//! Invariants:
//!
//! * I0: Every owned or exclusive permission should be associated with exactly one object across the entire machine.

use crate::machine::op::MachineOpExt;

use super::Stepper;

impl Stepper<'_> {
    pub(crate) fn assert_invariants(&self) -> eyre::Result<()> {
        // Convert an assertion failure into a panic intentionally;
        // it's not the same as other sorts of failures.
        self.machine.assert_invariants(self.db).unwrap();
        Ok(())
    }
}
