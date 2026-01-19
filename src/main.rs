mod cli;
mod io;
mod model;
mod validation;
mod typst;
mod log;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    log::init()?;
    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Build(opts) => {
            crate::io::run_build(opts)
        }
        cli::Commands::Validate { input } => {
            crate::validation::validate_input(&input)
        }
    }
}
