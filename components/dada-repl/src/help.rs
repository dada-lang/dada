pub type CommandName = &'static str;
pub type CommandDesc = &'static str;

pub struct HelpInfo {
    pub commands: Vec<(CommandName, CommandDesc)>,
}

impl HelpInfo {
    pub fn new() -> HelpInfo {
        HelpInfo {
            commands: vec![
                (
                    ":help",
                    "Display this message",
                ),
                (
                    ":exit",
                    "Exit the repl (also Ctrl-D)",
                ),
                (
                    ":reset",
                    "Clear the repl state",
                ),
                (
                    ":load",
                    "Load a .dada file into the repl",
                ),
                (
                    ":dump-source",
                    "Print the synthetic source file representing tihs repl session",
                ),
            ],
        }
    }
}
