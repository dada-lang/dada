use std::ffi::OsString;

use structopt::StructOpt;

mod check;
mod ide;
mod run;
mod test_harness;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

impl Options {
    /// Returns the options to run the default test harness.
    pub fn test_harness() -> Self {
        Options {
            cmd: Command::Test(test_harness::Options::from_iter(None::<OsString>)),
        }
    }

    pub async fn main(&self) -> eyre::Result<()> {
        match &self.cmd {
            Command::Ide(command_options) => {
                ide::main(self, command_options)?;
            }
            Command::Check(command_options) => command_options.main(self)?,
            Command::Test(command_options) => command_options.main(self).await?,
            Command::Run(command_options) => command_options.main(self).await?,
        }
        Ok(())
    }
}

#[derive(StructOpt)]
pub enum Command {
    /// Pound acorns into flour for cookie dough.
    Ide(ide::Options),
    Check(check::Options),
    Test(test_harness::Options),
    Run(run::Options),
}
