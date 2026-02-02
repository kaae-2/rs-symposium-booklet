use symposium_booklet::{cli::BuildOpts, config, io, log};

fn main() -> anyhow::Result<()> {
    log::init()?;

    let mut args = std::env::args().skip(1);
    let abstracts_arg = args.next();
    let ordering_arg = args.next();
    let output_arg = args.next();
    let locales = args.next().unwrap_or_else(|| "da".to_string());

    let config_path = config::default_config_path()?;
    let cfg = config::load_symposium_config(&config_path)?;

    let abstracts = abstracts_arg
        .or_else(|| cfg.as_ref().and_then(|c| c.abstracts.clone()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing abstracts path. Pass arg or set [symposium].abstracts in {}",
                config_path.to_string_lossy()
            )
        })?;
    let ordering = ordering_arg
        .or_else(|| cfg.as_ref().and_then(|c| c.ordering.clone()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing ordering path. Pass arg or set [symposium].ordering in {}",
                config_path.to_string_lossy()
            )
        })?;
    let output = output_arg
        .or_else(|| cfg.as_ref().and_then(|c| c.output.clone()))
        .unwrap_or_else(|| "out/example-render".to_string());

    let opts = BuildOpts {
        abstracts: Some(abstracts),
        ordering: Some(ordering),
        output: Some(output.clone()),
        template: None,
        locales: locales.clone(),
        dry_run: false,
        emit_parse_json: false,
        verbose: false,
        typst_bin: None,
    };

    io::run_build(opts)?;

    println!(
        "Rendered PDF(s) are in {} (symposium-2026_<locale>.pdf)",
        output
    );
    Ok(())
}
