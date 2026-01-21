pub mod excel;
pub mod markdown;
pub mod plan;

pub use excel::{parse_two_workbooks, parse_workbook};
pub use crate::io::plan::{Plan, PlanAction};

use crate::cli::BuildOpts;
use anyhow::Result;

pub fn run_build(opts: BuildOpts) -> Result<()> {
    // if user passed an option to emit parse JSON, handle it here
    if opts.dry_run {
        tracing::info!("Dry run: validating input {}", opts.input);
    } else {
        tracing::info!("Building with input={} output={}", opts.input, opts.output);
    }

    if opts.dry_run {
        tracing::info!("Dry run: validating input {}", opts.input);
    } else {
        tracing::info!("Building with input={} output={}", opts.input, opts.output);
    }

    // validate input (parse + reference checks)
    crate::validation::validate_input(&opts.input)?;

    // parse excel (again to obtain values for the build path)
    let (abstracts, sessions) = excel::parse_workbook(&opts.input)?;

    // In dry-run mode, collect a plan of actions instead of writing files
    let mut plan = Plan::default();

    // If requested, emit a parse JSON and exit
    if opts.emit_parse_json {
        let outdir = std::path::Path::new(&opts.output).join("tools_output");
        std::fs::create_dir_all(&outdir)?;
        let manifest_path = outdir.join("parse.json");
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "summary": {"num_abstracts_parsed": abstracts.len(), "num_sessions": sessions.len()},
            "abstracts": abstracts,
            "sessions": sessions
        }))?;
        std::fs::write(&manifest_path, json)?;
        tracing::info!("Wrote parse JSON to {}", manifest_path.display());
        return Ok(());
    }

    if opts.dry_run {
        // ask markdown writer to produce plan entries
        markdown::write_markdown_plan(&abstracts, &sessions, &opts.output, &mut plan)?;
        crate::typst::emit_typst_plan(&opts.output, &opts.locales, &opts.template, &mut plan)?;

        // print pretty plan and json to stdout
        println!("DRY-RUN PLAN:\n{}", plan.pretty_print());
        let plan_json = serde_json::to_string_pretty(&plan)?;
        println!("PLAN JSON:\n{}", plan_json);
        return Ok(());
    }

    // write md
    markdown::write_markdown(&abstracts, &sessions, &opts.output)?;

    // emit typst
    crate::typst::emit_typst(&opts.output, &opts.locales, &opts.template)?;

    // attempt to run typst if available
    crate::typst::maybe_run_typst(&opts.output, &opts.locales, opts.typst_bin.as_deref())?;

    Ok(())
}
