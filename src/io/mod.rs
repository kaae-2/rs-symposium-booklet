pub mod excel;
pub mod markdown;

use crate::cli::BuildOpts;
use anyhow::Result;

pub fn run_build(opts: BuildOpts) -> Result<()> {
    if opts.dry_run {
        tracing::info!("Dry run: validating input {}", opts.input);
    } else {
        tracing::info!("Building with input={} output={}", opts.input, opts.output);
    }

    // parse excel
    let (abstracts, sessions) = excel::parse_workbook(&opts.input)?;

    // validate
    crate::validation::validate_refs(&abstracts, &sessions)?;

    // write md
    markdown::write_markdown(&abstracts, &sessions, &opts.output)?;

    // emit typst
    crate::typst::emit_typst(&opts.output, &opts.locales, &opts.template)?;

    // attempt to run typst if available
    crate::typst::maybe_run_typst(&opts.output, &opts.locales, opts.typst_bin.as_deref())?;

    Ok(())
}
