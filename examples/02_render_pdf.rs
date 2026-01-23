use symposium_booklet::{cli::BuildOpts, io, log};

fn main() -> anyhow::Result<()> {
    log::init()?;

    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap_or_else(|| "data".to_string());
    let output = args
        .next()
        .unwrap_or_else(|| "output/example-render".to_string());
    let locales = args.next().unwrap_or_else(|| "da".to_string());

    let opts = BuildOpts {
        input: input.clone(),
        output: output.clone(),
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
