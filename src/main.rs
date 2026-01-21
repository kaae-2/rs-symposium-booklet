mod cli;
mod io;
mod log;
mod model;
mod typst;
mod validation;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    log::init()?;
    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Build(opts) => crate::io::run_build(opts),
        cli::Commands::EmitTypst { output, template, locales, typst_bin } => {
            crate::typst::emit_typst(&output, &locales, &template)?;
            crate::typst::maybe_run_typst(&output, &locales, typst_bin.as_deref())?;
            Ok(())
        }
        cli::Commands::Validate { input } => crate::validation::validate_input(&input),
    }
}
