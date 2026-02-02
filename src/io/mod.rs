pub mod excel;
pub mod markdown;
pub mod plan;

use crate::cli::BuildOpts;
use crate::config::{default_config_path, load_symposium_config, resolve_cwd_path};
use anyhow::Result;
use std::path::Path;

pub fn run_build(opts: BuildOpts) -> Result<()> {
    let config_path = default_config_path()?;
    let cfg = load_symposium_config(&config_path)?;
    let abstracts = opts
        .abstracts
        .or_else(|| cfg.as_ref().and_then(|c| c.abstracts.clone()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing abstracts path. Pass --abstracts or set [symposium].abstracts in {}",
                config_path.to_string_lossy()
            )
        })?;
    let ordering = opts
        .ordering
        .or_else(|| cfg.as_ref().and_then(|c| c.ordering.clone()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing ordering path. Pass --ordering or set [symposium].ordering in {}",
                config_path.to_string_lossy()
            )
        })?;
    let output = opts
        .output
        .or_else(|| cfg.as_ref().and_then(|c| c.output.clone()))
        .unwrap_or_else(|| "output".to_string());
    let template = opts
        .template
        .or_else(|| cfg.as_ref().and_then(|c| c.template.clone()));
    let typst_bin = opts
        .typst_bin
        .or_else(|| cfg.as_ref().and_then(|c| c.typst_bin.clone()));

    let abstracts_path = resolve_cwd_path(&abstracts)?;
    let ordering_path = resolve_cwd_path(&ordering)?;
    let output_path = resolve_cwd_path(&output)?;

    // if user passed an option to emit parse JSON, handle it here
    if opts.dry_run {
        tracing::info!(
            "Dry run: validating abstracts={} ordering={}",
            abstracts_path,
            ordering_path
        );
    } else {
        tracing::info!(
            "Building with abstracts={} ordering={} output={}",
            abstracts_path,
            ordering_path,
            output_path
        );
    }

    // validate input (parse + reference checks)
    crate::validation::validate_inputs(Some(abstracts), Some(ordering))?;

    // parse excel (again to obtain values for the build path)
    let (abstracts, sessions) = excel::parse_two_workbooks(&abstracts_path, &ordering_path)?;

    // In dry-run mode, collect a plan of actions instead of writing files
    let mut plan = plan::Plan::default();

    // If requested, emit a parse JSON and exit
    if opts.emit_parse_json {
        let outdir = std::path::Path::new(&output_path).join("tools_output");
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
        let outdir = Path::new(&output_path);
        plan.push(plan::PlanAction::DeleteDir {
            path: outdir.to_path_buf(),
        });
        // ask markdown writer to produce plan entries
        markdown::write_markdown_plan(&abstracts, &sessions, &output_path, &mut plan)?;
        crate::typst::emit_typst_plan(&output_path, &opts.locales, &template, &mut plan)?;

        // print pretty plan and json to stdout
        println!("DRY-RUN PLAN:\n{}", plan.pretty_print());
        let plan_json = serde_json::to_string_pretty(&plan)?;
        println!("PLAN JSON:\n{}", plan_json);
        return Ok(());
    }

    let outdir = Path::new(&output_path);
    if outdir.as_os_str().is_empty() {
        return Err(anyhow::anyhow!("Refusing to wipe empty output directory"));
    }
    if outdir == Path::new(".") {
        return Err(anyhow::anyhow!(
            "Refusing to wipe output directory set to current working directory"
        ));
    }
    if outdir.has_root() && outdir.components().count() <= 1 {
        return Err(anyhow::anyhow!(
            "Refusing to wipe output directory set to filesystem root"
        ));
    }
    if outdir.exists() {
        std::fs::remove_dir_all(outdir)?;
    }

    // write md
    markdown::write_markdown(&abstracts, &sessions, &output_path)?;

    // emit typst
    crate::typst::emit_typst(&output_path, &opts.locales, &template)?;

    // attempt to run typst if available
    crate::typst::maybe_run_typst(&output_path, &opts.locales, typst_bin.as_deref())?;

    Ok(())
}
