use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn emit_typst(outdir: &str, locales_csv: &str, template: &Option<String>) -> Result<()> {
    let typst_dir = Path::new(outdir).join("typst");
    create_dir_all(&typst_dir)?;

    for locale in locales_csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let filename = format!("book_{}.typ", locale);
        let path = typst_dir.join(&filename);
        let mut f = File::create(&path)?;
        writeln!(f, "# Generated typst for locale {}", locale)?;
        writeln!(f, "# TODO: include manifest parsing and layout")?;
    }
    Ok(())
}

pub fn maybe_run_typst(outdir: &str, locales_csv: &str, typst_bin: Option<&str>) -> Result<()> {
    let bin = if let Some(p) = typst_bin { p.to_string() } else { "typst".to_string() };
    // check if command exists by trying --version
    let check = Command::new(&bin).arg("--version").output();
    match check {
        Ok(o) if o.status.success() => {
            for locale in locales_csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                let typst_file = Path::new(outdir).join("typst").join(format!("book_{}.typ", locale));
                let out_pdf = Path::new(outdir).join(format!("symposium-2026_{}.pdf", locale));
                tracing::info!("Running typst: {} -> {}", typst_file.display(), out_pdf.display());
                let status = Command::new(&bin)
                    .arg("compile")
                    .arg(typst_file)
                    .arg("-o")
                    .arg(out_pdf)
                    .status()?;
                if !status.success() {
                    return Err(anyhow!("typst failed for locale {}", locale));
                }
            }
            Ok(())
        }
        _ => {
            tracing::warn!("Typst binary '{}' not found or not runnable; typst files emitted in {}/typst. Run 'typst compile <file> -o <out.pdf>'", bin, outdir);
            Ok(())
        }
    }
}
