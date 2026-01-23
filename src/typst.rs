use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
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
    take_home: Option<String>,
}

// Emit typst files by reading `outdir/manifest.json` and per-abstract markdown frontmatter.
pub fn emit_typst(outdir: &str, locales_csv: &str, _template: &Option<String>) -> Result<()> {
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
    type LocaleSessions = Vec<(String, Vec<(FrontMatter, String)>)>;
    let mut locales: HashMap<String, LocaleSessions> = HashMap::new();

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
                let slot = locales.entry(locale).or_default();
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
        let take_home_label = labels
            .get("take_home_label")
            .cloned()
            .unwrap_or_else(|| "Take-home".to_string());
        let tags_label = labels
            .get("tags_label")
            .cloned()
            .unwrap_or_else(|| "Tags".to_string());
        let tag_index_label = labels
            .get("tag_index_label")
            .cloned()
            .unwrap_or_else(|| "Tag index".to_string());
        let authors_label = labels
            .get("authors_label")
            .cloned()
            .unwrap_or_else(|| "Authors".to_string());
        let affiliation_label = labels
            .get("affiliation_label")
            .cloned()
            .unwrap_or_else(|| "Affiliation".to_string());

        // build generated content into a string
        let mut gen = String::new();

        if let Some(sess_list) = locales.get(locale).or_else(|| locales.get("en")) {
            let mut first_session = true;
            for (sess_title, abstracts) in sess_list {
                if !first_session {
                    gen.push_str("#pagebreak()\n");
                }
                first_session = false;

                let sess_title_upper = escape_typst_text(&sess_title.to_uppercase());
                gen.push_str("#set page(fill: brand-blue)\n");
                gen.push_str(&format!("= {}\n\n", sess_title_upper));
                gen.push_str("#pagebreak()\n#set page(fill: none)\n");
                // sort by order if present
                let mut abs_sorted = abstracts.clone();
                abs_sorted.sort_by_key(|(fm, _)| fm.order.unwrap_or(0));
                let abs_len = abs_sorted.len();
                for (idx, (fm, body)) in abs_sorted.into_iter().enumerate() {
                    let abs_title = escape_typst_text(&fm.title);
                    let abs_label = label_for_abstract(&fm);
                    gen.push_str(&format!("== {} <{}>\n\n", abs_title, abs_label));
                    // add authors/affiliation
                    let mut meta_written = false;
                    if let Some(auths) = &fm.authors {
                        let joined = auths.join(", ");
                        gen.push_str(&format!(
                            "*{}*: {}\n",
                            escape_typst_text(&authors_label),
                            escape_typst_text(&joined)
                        ));
                        meta_written = true;
                    }
                    if let Some(aff) = &fm.affiliation {
                        if fm.authors.is_some() {
                            gen.push_str("#v(6pt)\n");
                        }
                        let affiliations = unique_list(aff);
                        let aff_text = if affiliations.is_empty() {
                            escape_typst_text(aff)
                        } else {
                            escape_typst_text(&affiliations.join("; "))
                        };
                        gen.push_str(&format!(
                            "*{}*: {}\n",
                            escape_typst_text(&affiliation_label),
                            aff_text
                        ));
                        meta_written = true;
                    }
                    let body_text = escape_typst_text(body.trim());
                    if meta_written {
                        gen.push_str("#v(8pt)\n");
                    }
                    gen.push('\n');
                    gen.push_str(&body_text);
                    gen.push_str("\n\n");
                    if let Some(take_home) = &fm.take_home {
                        gen.push_str("#v(8pt)\n");
                        gen.push_str(&format!(
                            "*{}*: #emph[{}]\n",
                            escape_typst_text(&take_home_label),
                            escape_typst_text(take_home)
                        ));
                    }
                    if let Some(tags) = &fm.keywords {
                        let formatted = format_tags(tags);
                        if !formatted.is_empty() {
                            gen.push_str("#v(8pt)\n");
                            gen.push_str(&format!(
                                "#set par(justify: false)\n#text(size: 8.5pt, fill: rgb(\"#646c6f\"))[*{}*: {}]\n#set par(justify: true)\n",
                                escape_typst_text(&tags_label),
                                escape_typst_text(&formatted.join(" "))
                            ));
                        }
                    }
                    if idx + 1 < abs_len {
                        gen.push_str("#pagebreak()\n\n");
                    }
                }
            }
        } else {
            gen.push_str(&format!(
                "No content for locale \"{}\".\n",
                escape_typst_text(locale)
            ));
        }

        // build an index from keywords
        let mut keyword_map: std::collections::BTreeMap<String, Vec<(String, String)>> =
            std::collections::BTreeMap::new();
        if let Some(sess_list) = locales.get(locale).or_else(|| locales.get("en")) {
            for (_sess_title, abstracts) in sess_list {
                for (fm, _body) in abstracts.iter() {
                    if let Some(ks) = &fm.keywords {
                        for k in ks.iter() {
                            let normalized = k.replace(" - ", ",").replace(". ", ",");
                            for part in normalized.split(',') {
                                let key = part.trim().to_lowercase();
                                if key.is_empty() {
                                    continue;
                                }
                                let label = label_for_abstract(fm);
                                keyword_map
                                    .entry(key)
                                    .or_default()
                                    .push((fm.title.clone(), label));
                            }
                        }
                    }
                }
            }
        }

        if !keyword_map.is_empty() {
            let mut tag_map: std::collections::BTreeMap<String, Vec<(String, String)>> =
                std::collections::BTreeMap::new();
            for (k, titles) in keyword_map.iter() {
                let formatted = format_tags(std::slice::from_ref(k));
                if formatted.is_empty() {
                    continue;
                }
                let tag = formatted[0].clone();
                tag_map.entry(tag).or_default().extend(titles.clone());
            }
            if !tag_map.is_empty() {
                gen.push_str("#pagebreak()\n");
                gen.push_str(
                    "#show heading.where(level: 1): it => block(above: 10pt, below: 10pt)[\n  #set text(size: 13pt, weight: \"bold\", font: \"Source Sans 3\")\n  #text(fill: brand-blue)[#it.body]\n]\n",
                );
                gen.push_str(&format!("= {}\n\n", escape_typst_text(&tag_index_label)));
                for (tag, titles) in tag_map.iter() {
                    let mut uniq = titles.clone();
                    uniq.sort_by(|a, b| a.0.cmp(&b.0));
                    uniq.dedup_by(|a, b| a.1 == b.1);
                    let links: Vec<String> = uniq
                        .iter()
                        .map(|(title, label)| {
                            let title_text = escape_typst_text(title);
                            format!(
                                "#link(<{}>)[{}] (#context counter(page).at(<{}>).at(0))",
                                label, title_text, label
                            )
                        })
                        .collect();
                    gen.push_str(&format!(
                        "- {}: {}\n",
                        escape_typst_text(tag),
                        links.join("; ")
                    ));
                }
                gen.push('\n');
            }
        }

        // Build a minimal validated Typst document to avoid template/comment
        // interpolation issues. This produces consistent output and is easy
        // to extend later with richer templates.
        let title_upper = escape_typst_text(&title_label.to_uppercase());
        let header = format!(
            r##"#set page(
  width: 148mm,
  height: 210mm,
  margin: (top: 20mm, bottom: 18mm, left: 18mm, right: 18mm),
)
#let brand-blue = rgb("#007dbb")
#let brand-navy = rgb("#002555")
#let brand-sky = rgb("#009ce8")
#let brand-muted = rgb("#e5f2f8")
#let page-footer = [
  #align(center)[
    #text(fill: rgb("#646c6f"), size: 8.5pt)[#context counter(page).display()]
  ]
]
#set text(font: "Libertinus Serif", size: 10.5pt, fill: rgb("#333333"))
#set par(justify: true)
#set heading(numbering: none)
#show heading.where(level: 1): it => block(above: 0pt, below: 0pt)[
  #align(center)[
    #v(90pt)
    #text(size: 36pt, weight: "bold", font: "Source Sans 3", fill: white)[#it.body]
  ]
]
#show heading.where(level: 2): it => block(above: 10pt, below: 10pt)[
  #set text(size: 13pt, weight: "bold", font: "Source Sans 3")
  #text(fill: brand-blue)[#it.body]
]
#show heading.where(level: 3): it => block(above: 8pt, below: 4pt)[
  #set text(size: 11.5pt, weight: "semibold", font: "Source Sans 3")
  #text(fill: brand-navy)[#it.body]
]
#show strong: set text(weight: "semibold", fill: brand-navy)

#set page(footer: none)
#set page(fill: gradient.linear(angle: 45deg, brand-blue, brand-sky))
#align(center)[
  #v(110pt)
  #text(size: 40pt, weight: "bold", font: "Source Sans 3", fill: white)[{title_upper}]
]
#pagebreak()
#set page(fill: none)
#pagebreak()
#set page(footer: page-footer)
"##,
            title_upper = title_upper
        );

        let toc_section = format!(
            "#v(-16pt)\n#text(size: 15pt, weight: \"bold\", font: \"Source Sans 3\", fill: brand-blue)[Indholdsfortegnelse]\n#v(6pt)\n#set par(justify: false, spacing: 2pt)\n#set text(size: 9.5pt)\n#show outline.entry.where(level: 1): set text(weight: \"bold\")\n#outline(title: [{}], depth: 2, indent: 1.1em)\n#pagebreak()\n#set text(size: 10.5pt)\n#set par(justify: true)\n",
            escape_typst_text(&toc_label)
        );

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
    m.insert("take_home_label".to_string(), "Take-home".to_string());
    m.insert("tags_label".to_string(), "Tags".to_string());
    m.insert("tag_index_label".to_string(), "Tag index".to_string());
    m
}

fn escape_typst_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn unique_list(input: &str) -> Vec<String> {
    let mut parts: Vec<String> = input
        .replace(" / ", ";")
        .split(';')
        .flat_map(|part| part.split('/'))
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect();

    if parts.len() <= 1 {
        parts = input
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect();
    }

    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();
    for part in parts {
        if seen.insert(part.to_lowercase()) {
            unique.push(part);
        }
    }
    unique
}

fn format_tags(tags: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for tag in tags.iter() {
        let normalized = tag.replace(" - ", ",").replace(". ", ",");
        for part in normalized.split(',') {
            let cleaned = part.trim();
            if cleaned.is_empty() {
                continue;
            }
            let normalized = cleaned
                .chars()
                .map(|ch| if ch.is_whitespace() { '_' } else { ch })
                .collect::<String>()
                .to_lowercase();
            let formatted = format!("#{}", normalized);
            if seen.insert(formatted.to_lowercase()) {
                out.push(formatted);
            }
        }
    }
    out
}

fn label_for_abstract(fm: &FrontMatter) -> String {
    let base = if !fm.id.is_empty() {
        fm.id.clone()
    } else {
        fm.title.clone()
    };
    let mut label: String = base
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    while label.starts_with('-') {
        label.remove(0);
    }
    while label.ends_with('-') {
        label.pop();
    }
    if label.is_empty() {
        label = "abstract".to_string();
    }
    format!("abs-{}", label)
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
    if let Some(t) = v.get("take_home_label").and_then(|s| s.as_str()) {
        m.insert("take_home_label".to_string(), t.to_string());
    }
    if let Some(t) = v.get("tags_label").and_then(|s| s.as_str()) {
        m.insert("tags_label".to_string(), t.to_string());
    }
    if let Some(t) = v.get("tag_index_label").and_then(|s| s.as_str()) {
        m.insert("tag_index_label".to_string(), t.to_string());
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
            let font_path = Path::new("templates")
                .join("starter")
                .join("fonts")
                .join("TTF");
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
                    .arg("--font-path")
                    .arg(&font_path)
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
        let font_path = Path::new("templates")
            .join("starter")
            .join("fonts")
            .join("TTF");
        let cmd = Some(format!(
            "typst compile --font-path {} {} {}",
            font_path.display(),
            path.display(),
            Path::new(outdir)
                .join(format!("symposium-2026_{}.pdf", locale))
                .display()
        ));
        plan.push(PlanAction::EmitTypst {
            path,
            template: template_name,
            command: cmd,
        });
    }
    Ok(())
}
