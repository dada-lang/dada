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
            ControlFlow::Done(pc, v) => {
                stepper.print_if_not_unit(pc, v).await?;
                return Ok(());
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip(db, kernel, machine))]
pub async fn interpret_until_for_repl(
    db: &dyn crate::Db,
    kernel: &mut dyn Kernel,
    machine: &mut Machine,
    stop_fn: &str,
) -> eyre::Result<()> {
    let mut stepper = Stepper::new(db, machine, kernel);

    loop {
        tracing::trace!("machine = {:#?}", stepper);
        match stepper.step()? {
            ControlFlow::Next => {
                let pc = stepper.machine.pc();
                let function = pc.bir.origin(db);
                let function_name = function.name(db).as_str(db);
                if function_name == stop_fn {
                    let top_frame = stepper.machine.top_frame().expect("frame");
                    let first_argument = *top_frame.locals.iter().next().expect("value");
                    stepper.print_if_not_unit(pc, first_argument).await?;
                    return Ok(());
                }
            }
            ControlFlow::Await(t) => t.invoke(&mut stepper).await?,
            ControlFlow::Done(_, _) => {
                return Ok(());
            }
        }
    }
}
