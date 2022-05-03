use dada_ir::code::bir::Bir;
use salsa::DebugWithDb;

use crate::{
    kernel::Kernel,
    machine::{op::MachineOp, Machine, Value},
    step::{ControlFlow, Stepper},
};

/// Interprets a given function with the given kernel. Assumes this is the top stack frame.
/// Prints the result if it is not `()` to stdout.
#[tracing::instrument(level = "debug", skip(bir, db, kernel, arguments))]
pub async fn interpret(
    bir: Bir,
    db: &dyn crate::Db,
    kernel: &mut dyn Kernel,
    arguments: Vec<Value>,
) -> eyre::Result<()> {
    tracing::debug!(
        "function={:?} arguments={:#?}",
        bir.origin(db).name(db).debug(db),
        arguments
    );
    let machine: &mut Machine = &mut Machine::default();
    machine.push_frame(db, bir, arguments);
    let mut stepper = Stepper::new(db, machine, kernel);

    loop {
        tracing::trace!("machine = {:#?}", stepper);
        match stepper.step()? {
            ControlFlow::Next => (),
            ControlFlow::Await(t) => t.invoke(&mut stepper).await?,
            ControlFlow::Done(pc, v) => {
                stepper.print_if_not_unit(pc, v).await?;
                return Ok(());
            }
        }
    }
}
