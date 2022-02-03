use dada_brew::prelude::*;
use dada_ir::function::Function;

use crate::{
    kernel::Kernel,
    machine::{op::MachineOp, Machine, Value},
    step::{ControlFlow, Stepper},
};

/// Interprets a given function with the given kernel. Assumes this is the top stack frame.
/// Prints the result if it is not `()` to stdout.
#[tracing::instrument(level = "debug", skip(function, db, kernel, arguments))]
pub async fn interpret(
    function: Function,
    db: &dyn crate::Db,
    kernel: &mut dyn Kernel,
    arguments: Vec<Value>,
) -> eyre::Result<()> {
    tracing::debug!(
        "function={} arguments={:#?}",
        function.name(db).as_str(db),
        arguments
    );
    let bir = function.brew(db);
    let machine: &mut Machine = &mut Machine::default();
    machine.push_frame(db, bir, arguments);
    let mut stepper = Stepper::new(db, machine, kernel);

    loop {
        tracing::trace!("machine = {:#?}", stepper);
        match stepper.step()? {
            ControlFlow::Next => (),
            ControlFlow::Await(t) => t.invoke(&mut stepper).await?,
            ControlFlow::Done(v) => {
                stepper.print_if_not_unit(v).await?;
                return Ok(());
            }
        }
    }
}
