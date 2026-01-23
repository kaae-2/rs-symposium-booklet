use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
struct FrontMatter {
    id: String,
    title: String,
    authors: Option<Vec<String>>,
    affiliation: Option<String>,
    session: Option<String>,
    order: Option<u32>,
    locale: Option<String>,
    keywords: Option<Vec<String>>,
}

// Emit typst files by reading `outdir/manifest.json` and per-abstract markdown frontmatter.
pub fn emit_typst(outdir: &str, locales_csv: &str, template: &Option<String>) -> Result<()> {
    let typst_dir = Path::new(outdir).join("typst");
    create_dir_all(&typst_dir)?;

    // read manifest
    let mf_path = Path::new(outdir).join("manifest.json");
    if !mf_path.exists() {
        return Err(anyhow!(
            "manifest.json not found in output directory: {}",
            mf_path.display()
        ));
    }
    let mf_text = read_to_string(&mf_path)?;
    let mf: JsonValue = serde_json::from_str(&mf_text)?;

    // build a map locale -> Vec<(session_title, Vec<(frontmatter, body)>)>
    let mut locales: HashMap<String, Vec<(String, Vec<(FrontMatter, String)>)>> = HashMap::new();

    if let Some(sessions) = mf.get("sessions").and_then(|s| s.as_array()) {
        for sess in sessions.iter() {
            let sess_title = sess
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let sess_slug = sess.get("slug").and_then(|s| s.as_str()).unwrap_or("");
            if sess_slug.is_empty() {
                continue;
            }
            let session_dir = Path::new(outdir).join(sess_slug);
            if !session_dir.exists() {
                continue;
            }
            // collect markdown files in session dir
            let mut abstracts: Vec<(FrontMatter, String)> = Vec::new();
            let mut entries: Vec<_> = read_dir(&session_dir)?.filter_map(|r| r.ok()).collect();
            entries.sort_by_key(|e| e.path());
            for ent in entries {
                let p = ent.path();
                if p.extension().and_then(|e| e.to_str()).unwrap_or("") != "md" {
                    continue;
                }
                let txt = read_to_string(&p)?;
                // parse frontmatter between first two '---' lines
                if let Some(start) = txt.find("---") {
                    if let Some(rest) = txt[start + 3..].find("---") {
                        let fm_text = &txt[start + 3..start + 3 + rest];
                        let body = txt[start + 3 + rest + 3..].trim().to_string();
                        match serde_yaml::from_str::<FrontMatter>(fm_text) {
                            Ok(fm) => abstracts.push((fm, body)),
                            Err(_) => continue,
                        }
                    }
                }
            }

            // group abstracts by locale
            for (fm, body) in abstracts.into_iter() {
                let locale = fm.locale.clone().unwrap_or_else(|| "en".to_string());
                let slot = locales.entry(locale).or_insert_with(Vec::new);
                // find or push session entry
                if let Some((_, v)) = slot.iter_mut().find(|(t, _)| t == &sess_title) {
                    v.push((fm, body));
                } else {
                    slot.push((sess_title.clone(), vec![(fm, body)]));
                }
            }
        }
    }

    for locale in locales_csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let filename = format!("book_{}.typ", locale);
        let path = typst_dir.join(&filename);

        // load localized labels from templates/starter/locales/<locale>.toml if present
        let labels = load_locale_labels(locale).unwrap_or_else(|_| default_labels());
        let title_label = labels
            .get("title")
            .cloned()
            .unwrap_or_else(|| "Symposium 2026".to_string());
        let toc_label = labels
            .get("toc_label")
            .cloned()
            .unwrap_or_else(|| "Table of contents".to_string());
        let index_label = labels
            .get("index_label")
            .cloned()
            .unwrap_or_else(|| "Index".to_string());
        let authors_label = labels
            .get("authors_label")
            .cloned()
            .unwrap_or_else(|| "Authors".to_string());
        let affiliation_label = labels
            .get("affiliation_label")
            .cloned()
            .unwrap_or_else(|| "Affiliation".to_string());

        // build generated content into a string, and build a TOC list
        let mut gen = String::new();
        let mut toc_items: Vec<String> = Vec::new();

        if let Some(sess_list) = locales.get(locale).or_else(|| locales.get("en")) {
            for (sess_title, abstracts) in sess_list {
                let sess_title_text = escape_typst_text(sess_title);
                toc_items.push(format!("- {}", sess_title_text));
                gen.push_str(&format!("== {}\n\n", sess_title_text));
                // sort by order if present
                let mut abs_sorted = abstracts.clone();
                abs_sorted.sort_by_key(|(fm, _)| fm.order.unwrap_or(0));
                for (fm, body) in abs_sorted {
                    let abs_title = escape_typst_text(&fm.title);
                    toc_items.push(format!("  - {}", abs_title));
                    gen.push_str(&format!("=== {}\n\n", abs_title));
                    // add authors/affiliation
                    if let Some(auths) = &fm.authors {
                        let joined = auths.join(", ");
                        gen.push_str(&format!(
                            "*{}*: {}\n",
                            escape_typst_text(&authors_label),
                            escape_typst_text(&joined)
                        ));
                    }
                    if let Some(aff) = &fm.affiliation {
                        gen.push_str(&format!(
                            "*{}*: {}\n",
                            escape_typst_text(&affiliation_label),
                            escape_typst_text(aff)
                        ));
                    }
                    let body_text = escape_typst_text(body.trim());
                    gen.push_str("\n");
                    gen.push_str(&body_text);
                    gen.push_str("\n\n");
                }
            }
        } else {
            gen.push_str(&format!(
                "No content for locale \"{}\".\n",
                escape_typst_text(locale)
            ));
        }

        // build an index from keywords
        let mut keyword_map: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        if let Some(sess_list) = locales.get(locale).or_else(|| locales.get("en")) {
            for (_sess_title, abstracts) in sess_list {
                for (fm, _body) in abstracts.iter() {
                    if let Some(ks) = &fm.keywords {
                        for k in ks.iter() {
                            let key = k.trim().to_string();
                            if key.is_empty() {
                                continue;
                            }
                            keyword_map.entry(key).or_default().push(fm.title.clone());
                        }
                    }
                }
            }
        }

        if !keyword_map.is_empty() {
            gen.push_str(&format!("== {}\n\n", escape_typst_text(&index_label)));
            for (k, titles) in keyword_map.iter() {
                let uniq: Vec<String> = {
                    let mut s = titles.clone();
                    s.sort();
                    s.dedup();
                    s
                };
                let links: Vec<String> = uniq.iter().cloned().collect();
                gen.push_str(&format!(
                    "- {}: {}\n",
                    escape_typst_text(k),
                    escape_typst_text(&links.join("; "))
                ));
            }
            gen.push_str("\n");
        }

        // Build a minimal validated Typst document to avoid template/comment
        // interpolation issues. This produces consistent output and is easy
        // to extend later with richer templates.
        let header = format!(
            "#set page(width: 148mm, height: 210mm, margin: 18mm)\n#set text(font: \"Libertinus Serif\")\n#set heading(numbering: \"1.\")\n\n= {}\n\n",
            escape_typst_text(&title_label)
        );

        let toc_section = if !toc_items.is_empty() {
            format!(
                "== {}\n\n{}\n\n",
                escape_typst_text(&toc_label),
                toc_items.join("\n")
            )
        } else {
            "".to_string()
        };

        let out_text = format!("{}{}{}{}", header, toc_section, gen, "\n");

        let mut f = File::create(&path)?;
        write!(f, "{}", out_text)?;
    }

    Ok(())
}

fn default_labels() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("title".to_string(), "Symposium 2026".to_string());
    m.insert("authors_label".to_string(), "Authors".to_string());
    m.insert("affiliation_label".to_string(), "Affiliation".to_string());
    m.insert("toc_label".to_string(), "Table of contents".to_string());
    m.insert("index_label".to_string(), "Index".to_string());
    m
}

fn escape_typst_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn load_locale_labels(locale: &str) -> Result<HashMap<String, String>> {
    let path = Path::new("templates")
        .join("starter")
        .join("locales")
        .join(format!("{}.toml", locale));
    if !path.exists() {
        return Err(anyhow!("locale file not found"));
    }
    let txt = read_to_string(path)?;
    let v: toml::Value = toml::from_str(&txt)?;
    let mut m = HashMap::new();
    if let Some(t) = v.get("title").and_then(|s| s.as_str()) {
        m.insert("title".to_string(), t.to_string());
    }
    if let Some(a) = v.get("authors_label").and_then(|s| s.as_str()) {
        m.insert("authors_label".to_string(), a.to_string());
    }
    if let Some(a) = v.get("affiliation_label").and_then(|s| s.as_str()) {
        m.insert("affiliation_label".to_string(), a.to_string());
    }
    if let Some(t) = v.get("toc_label").and_then(|s| s.as_str()) {
        m.insert("toc_label".to_string(), t.to_string());
    }
    if let Some(t) = v.get("index_label").and_then(|s| s.as_str()) {
        m.insert("index_label".to_string(), t.to_string());
    }
    Ok(m)
}

pub fn maybe_run_typst(outdir: &str, locales_csv: &str, typst_bin: Option<&str>) -> Result<()> {
    let bin = if let Some(p) = typst_bin {
        p.to_string()
    } else {
        "typst".to_string()
    };
    let check = Command::new(&bin).arg("--version").output();
    match check {
        Ok(o) if o.status.success() => {
            for locale in locales_csv
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                let typst_file = Path::new(outdir)
                    .join("typst")
                    .join(format!("book_{}.typ", locale));
                let out_pdf = Path::new(outdir).join(format!("symposium-2026_{}.pdf", locale));
                tracing::info!(
                    "Running typst: {} -> {}",
                    typst_file.display(),
                    out_pdf.display()
                );
                // typst CLI accepts OUTPUT as a positional argument rather than `-o` in some versions
                let status = Command::new(&bin)
                    .arg("compile")
                    .arg(typst_file)
                    .arg(out_pdf)
                    .status()?;
                if !status.success() {
                    return Err(anyhow!("typst failed for locale {}", locale));
                }
            }
            Ok(())
        }
        _ => {
            tracing::warn!(
                "Typst binary '{}' not found or not runnable; typst files emitted in {}/typst.",
                bin,
                outdir
            );
            tracing::warn!("To render PDFs run: typst compile <typst-file> -o <out.pdf>");
            Ok(())
        }
    }
}

// Emit a plan of typst files that would be generated.
pub fn emit_typst_plan(
    outdir: &str,
    locales_csv: &str,
    template: &Option<String>,
    plan: &mut crate::io::plan::Plan,
) -> Result<()> {
    use crate::io::plan::PlanAction;
    use std::path::PathBuf;

    let typst_dir = Path::new(outdir).join("typst");
    plan.push(PlanAction::CreateDir {
        path: PathBuf::from(&typst_dir),
    });

    for locale in locales_csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let filename = format!("book_{}.typ", locale);
        let path = typst_dir.join(&filename);
        let template_name = template
            .clone()
            .unwrap_or_else(|| "templates/starter/book.typ".to_string());
        let cmd = Some(format!(
            "typst compile {} -o {}",
            path.display(),
            Path::new(outdir)
                .join(format!("symposium-2026_{}.pdf", locale))
                .display()
        ));
        plan.push(PlanAction::EmitTypst {
            path: PathBuf::from(path),
            template: template_name,
            command: cmd,
        });
    }
    Ok(())
}
