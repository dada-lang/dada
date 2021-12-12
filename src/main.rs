use structopt::StructOpt;

mod ide;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    /// Pound acorns into flour for cookie dough.
    Ide(ide::Options),
}

fn main() -> eyre::Result<()> {
    let options = Options::from_args();
    match &options.cmd {
        Command::Ide(command_options) => {
            ide::main(&options, command_options)?;
        }
    }
    Ok(())
}
