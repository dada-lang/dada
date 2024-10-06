use dada_util::Fallible;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Fallible<()> {
    unsafe {
        backtrace_on_stack_overflow::enable();
    }
    dada_lang::Options::from_args().main()
}
