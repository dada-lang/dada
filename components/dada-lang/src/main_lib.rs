use dada_util::Fallible;

use crate::{Command, GlobalOptions};

mod compile;
mod run;
mod test;

pub struct Main {
    #[allow(dead_code)]
    global_options: GlobalOptions,
}

impl Main {
    pub fn new(global_options: GlobalOptions) -> Self {
        Self { global_options }
    }

    pub fn run(mut self, command: Command) -> Fallible<()> {
        match command {
            Command::Compile { compile_options } => self.compile(&compile_options, None)?,
            Command::Test { test_options } => self.test(test_options)?,
            Command::Run { run_options } => self.run_command(&run_options)?,
            Command::Debug { debug_options, compile_options } => {
                let mut debug_server = debug_options.to_server();
                let debug_tx = debug_server.launch();
                let result = self.compile(&compile_options, Some(debug_tx));
                debug_server.block_on()?;
                result?;
            }
        }
        Ok(())
    }
}
