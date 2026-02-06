use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs::{File, create_dir_all, read_dir, read_to_string};
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
struct FrontMatter {
    id: String,
    title: String,
    tema: Option<String>,
    #[serde(rename = "type")]
    presentation_type: Option<String>,
    presenters: Option<Vec<String>>,
    affiliation: Option<String>,
    order: Option<u32>,
    locale: Option<String>,
    keywords: Option<Vec<String>>,
    take_home: Option<String>,
    reference: Option<String>,
    bibliography: Option<Vec<String>>,
    sections: Option<Vec<AbstractSection>>,
}

#[derive(Debug, Deserialize, Clone)]
struct AbstractSection {
    label: String,
    text: String,
}

#[derive(Debug, Clone)]
struct SessionEntry {
    title: String,
    slug: String,
    tema: String,
    presentation_type: String,
    abstracts: Vec<(FrontMatter, String)>,
}

const TEMA_ORDER: [&str; 3] = ["Miljø", "Teknologi", "Organisation"];
const TEMA_COLORS: [&str; 3] = ["#0070c0", "#ff0000", "#00b065"];

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

    // build session entries from manifest
    let mut sessions: Vec<SessionEntry> = Vec::new();
    if let Some(sess_list) = mf.get("sessions").and_then(|s| s.as_array()) {
        for sess in sess_list.iter() {
            let sess_title = sess
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let sess_slug = sess.get("slug").and_then(|s| s.as_str()).unwrap_or("");
            let tema = sess
                .get("tema")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let presentation_type = sess
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if sess_slug.is_empty() || tema.is_empty() || presentation_type.is_empty() {
                continue;
            }
            let session_dir = Path::new(outdir).join(sess_slug);
            if !session_dir.exists() {
                continue;
            }

            let mut abstracts: Vec<(FrontMatter, String)> = Vec::new();
            let mut entries: Vec<_> = read_dir(&session_dir)?.filter_map(|r| r.ok()).collect();
            entries.sort_by_key(|e| e.path());
            for ent in entries {
                let p = ent.path();
                if p.extension().and_then(|e| e.to_str()).unwrap_or("") != "md" {
                    continue;
                }
                let txt = read_to_string(&p)?;
                if let Some(start) = txt.find("---")
                    && let Some(rest) = txt[start + 3..].find("---")
                {
                    let fm_text = &txt[start + 3..start + 3 + rest];
                    let body = txt[start + 3 + rest + 3..].trim().to_string();
                    match serde_yaml::from_str::<FrontMatter>(fm_text) {
                        Ok(fm) => abstracts.push((fm, body)),
                        Err(_) => continue,
                    }
                }
            }

            sessions.push(SessionEntry {
                title: sess_title,
                slug: sess_slug.to_string(),
                tema,
                presentation_type,
                abstracts,
            });
        }
    }

    let mut locales: HashMap<String, Vec<SessionEntry>> = HashMap::new();
    for locale in locales_csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let mut locale_sessions: Vec<SessionEntry> = Vec::new();
        for session in sessions.iter() {
            let abstracts: Vec<(FrontMatter, String)> = session
                .abstracts
                .iter()
                .filter(|(fm, _)| {
                    fm.locale
                        .as_deref()
                        .unwrap_or("en")
                        .eq_ignore_ascii_case(locale)
                })
                .cloned()
                .collect();
            if abstracts.is_empty() {
                continue;
            }
            locale_sessions.push(SessionEntry {
                title: session.title.clone(),
                slug: session.slug.clone(),
                tema: session.tema.clone(),
                presentation_type: session.presentation_type.clone(),
                abstracts,
            });
        }
        locales.insert(locale.to_string(), locale_sessions);
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
        let link_index_label = labels
            .get("link_index_label")
            .cloned()
            .unwrap_or_else(|| "Links".to_string());
        // let presenters_label = labels
        //     .get("presenters_label")
        //     .cloned()
        //     .unwrap_or_else(|| "Presenters".to_string());
        let affiliation_label = labels
            .get("affiliation_label")
            .cloned()
            .unwrap_or_else(|| "Affiliation".to_string());
        let more_info_prefix = labels
            .get("more_info_prefix")
            .cloned()
            .unwrap_or_else(|| "For more info, click".to_string());
        let more_info_link_text = labels
            .get("more_info_link_text")
            .cloned()
            .unwrap_or_else(|| "this link".to_string());
        let cover_header_label = labels
            .get("cover_header")
            .cloned()
            .unwrap_or_else(|| "Interprofessional Education Symposium 2026".to_string());
        let cover_title_label = labels
            .get("cover_title")
            .cloned()
            .unwrap_or_else(|| "PROGRAM".to_string());
        let cover_symposium_label = labels
            .get("cover_symposium")
            .cloned()
            .unwrap_or_else(|| "Interprofessional Education Symposium".to_string());
        let cover_date_label = labels
            .get("cover_date")
            .cloned()
            .unwrap_or_else(|| "13 March 2026".to_string());
        let cover_subtitle_label = labels
            .get("cover_subtitle")
            .cloned()
            .unwrap_or_else(|| "The impact of intelligence on learning and guidance".to_string());

        // build generated content into a string
        let mut r#gen = String::new();
        let mut keyword_map: std::collections::BTreeMap<String, Vec<(String, String)>> =
            std::collections::BTreeMap::new();
        let mut link_map: std::collections::BTreeMap<String, Vec<(String, String)>> =
            std::collections::BTreeMap::new();
        let mut label_state = LabelState::default();

        if let Some(sess_list) = locales.get(locale).or_else(|| locales.get("en")) {
            let mut first_session = true;
            let mut current_tema: Option<String> = None;
            for session in sess_list.iter() {
                if !first_session {
                    r#gen.push_str("#pagebreak()\n");
                }
                first_session = false;

                let tema_num = tema_number(&session.tema);
                let tema_heading = match tema_num {
                    Some(num) => format!("TEMA {}: {}", num, session.tema.to_uppercase()),
                    None => session.tema.to_uppercase(),
                };
                let session_title_upper = session.title.to_uppercase();
                let session_title_parts: Vec<String> = session_title_upper
                    .split(" - ")
                    .map(|part| escape_typst_text(part))
                    .collect();
                let session_title_markup = session_title_parts.join(" #linebreak()\n- ");
                let sess_title_markup = match tema_num {
                    Some(num) => {
                        let left = escape_typst_text(&format!("TEMA {}:", num));
                        format!("{} {}", left, session_title_markup)
                    }
                    None => session_title_markup,
                };
                let sess_color = tema_color(&session.tema);
                r#gen.push_str(&format!(
                    "#set page(footer: none, header: none)\n#set page(fill: rgb(\"{}\"))\n",
                    sess_color
                ));
                r#gen.push_str("#show heading.where(level: 1): it => none\n#show heading.where(level: 2): it => none\n");
                if current_tema.as_deref() != Some(&session.tema) {
                    current_tema = Some(session.tema.clone());
                    r#gen.push_str(&format!(
                        "#heading(level: 1)[{}]\n",
                        escape_typst_text(&tema_heading)
                    ));
                }
                r#gen.push_str(&format!(
                    "#heading(level: 2)[{}]\n",
                    escape_typst_text(&session.presentation_type)
                ));
                r#gen.push_str(
                    "#show heading.where(level: 1): it => block(above: 0pt, below: 0pt)[\n  #align(center)[\n    #v(70pt)\n    #text(size: 28pt, weight: \"bold\", font: \"Mari\", fill: white)[#it.body]\n  ]\n]\n",
                );
                r#gen.push_str(
                    "#show heading.where(level: 2): it => block(above: 10pt, below: 10pt)[\n  #set text(size: 13pt, weight: \"bold\", font: \"Mari\")\n  #text(fill: brand-navy)[#it.body]\n]\n",
                );
                r#gen.push_str(&format!(
                    "#heading(level: 1, outlined: false)[{}]\n\n",
                    sess_title_markup
                ));
                r#gen.push_str(&format!(
                    "#pagebreak()\n#set page(fill: none, footer: page-footer, header: [#grid(columns: (auto, 1fr), align: (left, right), text(size: 8.5pt, fill: brand-navy)[{}], image(\"/templates/starter/images/Logo_dark.jpg\", height: 6mm))])\n",
                    escape_typst_text(&cover_header_label)
                ));
                r#gen.push_str(
                    "#show heading.where(level: 3): it => block(above: 8pt, below: 10pt)[\n  #set text(size: 11.5pt, weight: \"semibold\", font: \"Mari\")\n  #text(fill: brand-navy)[#it.body]\n]\n",
                );
                let mut abs_sorted = session.abstracts.clone();
                abs_sorted.sort_by_key(|(fm, _)| fm.order.unwrap_or(0));
                let abs_len = abs_sorted.len();
                for (idx, (fm, body)) in abs_sorted.into_iter().enumerate() {
                    let abs_title = escape_typst_text(&fm.title);
                    let abs_label = label_state.next(&fm);
                    r#gen.push_str(&format!("=== {} <{}>\n\n", abs_title, abs_label));
                    // add presenters/affiliation
                    let mut meta_written = false;
                    r#gen.push_str("#set text(size: 8.5pt)\n");
                    if let Some(presenters) = &fm.presenters {
                        for (p_idx, presenter) in presenters.iter().enumerate() {
                            r#gen.push_str(&format!("#emph[{}]", escape_typst_text(presenter)));
                            if p_idx + 1 < presenters.len() {
                                r#gen.push_str(" #linebreak()\n");
                            } else {
                                r#gen.push_str("\n");
                            }
                        }
                        meta_written = true;
                    }
                    if let Some(aff) = &fm.affiliation {
                        if fm.presenters.is_some() {
                            r#gen.push_str("#v(6pt)\n");
                        }
                        let affiliations = unique_list(aff);
                        let aff_text = if affiliations.is_empty() {
                            escape_typst_text(aff)
                        } else {
                            escape_typst_text(&affiliations.join("; "))
                        };
                        r#gen.push_str(&format!(
                            "*{}*: {}\n",
                            escape_typst_text(&affiliation_label),
                            aff_text
                        ));
                        meta_written = true;
                    }
                    let mut body_text = String::new();
                    if let Some(sections) = fm.sections.as_ref().filter(|s| !s.is_empty()) {
                        let mut parts: Vec<(String, String)> = Vec::new();
                        for section in sections.iter() {
                            let label = section.label.trim().to_string();
                            let text = section.text.trim().to_string();
                            if text.is_empty() {
                                continue;
                            }
                            parts.push((label, text));
                        }
                        if parts.is_empty() {
                            body_text = escape_typst_text(body.trim());
                        } else {
                            for (idx, (label, text)) in parts.iter().enumerate() {
                                let label_text = escape_typst_text(label);
                                let text_body = escape_typst_text(text);
                                if label_text.is_empty() {
                                    body_text.push_str(&text_body);
                                } else {
                                    body_text.push_str(&format!("*{}*: {}", label_text, text_body));
                                }
                                if idx + 1 < parts.len() {
                                    body_text.push_str("\n#v(6pt)\n");
                                }
                            }
                        }
                    } else {
                        body_text = escape_typst_text(body.trim());
                    }
                    if meta_written {
                        r#gen.push_str("#v(8pt)\n");
                    }
                    r#gen.push('\n');
                    r#gen.push_str("#set text(size: 8.5pt)\n");
                    r#gen.push_str(&body_text);
                    r#gen.push_str("\n#set text(size: 10.5pt)\n\n");
                    if let Some(take_home) = &fm.take_home {
                        r#gen.push_str("#v(8pt)\n");
                        r#gen.push_str(&format!(
                            "#set text(size: 8.5pt)\n*{}*: {}\n",
                            escape_typst_text(&take_home_label),
                            escape_typst_text(take_home)
                        ));
                    }
                    if let Some(reference) = fm.reference.as_ref().map(|s| s.trim()) {
                        if !reference.is_empty() {
                            link_map
                                .entry(reference.to_string())
                                .or_default()
                                .push((fm.title.clone(), abs_label.clone()));
                            r#gen.push_str("#v(8pt)\n");
                            r#gen.push_str(&format!(
                                "#set text(size: 8.5pt)\n{} #underline[#text(fill: brand-navy)[#link(\"{}\")[{}]]]\n",
                                escape_typst_text(&more_info_prefix),
                                escape_typst_string(reference),
                                escape_typst_text(&more_info_link_text)
                            ));
                        }
                    }
                    if let Some(bibliography) = fm.bibliography.as_ref().filter(|b| !b.is_empty()) {
                        r#gen.push_str("#v(8pt)\n");
                        r#gen.push_str("#set text(size: 6.5pt)\n");
                        r#gen.push_str("*References:*\n");
                        for entry in bibliography.iter() {
                            r#gen.push_str(&format!("+ {}\n", escape_typst_text(entry)));
                        }
                        r#gen.push_str("#set text(size: 10.5pt)\n");
                    }
                    if let Some(tags) = &fm.keywords {
                        let formatted = format_tags(tags);
                        if !formatted.is_empty() {
                            r#gen.push_str("#v(8pt)\n");
                            r#gen.push_str(&format!(
                                "#set par(justify: false)\n#text(size: 6.5pt, fill: rgb(\"#646c6f\"))[*{}*: {}]\n#set par(justify: true)\n",
                                escape_typst_text(&tags_label),
                                escape_typst_text(&formatted.join(" "))
                            ));
                        }
                    }
                    if let Some(ks) = &fm.keywords {
                        for k in ks.iter() {
                            let normalized = k.replace(" - ", ",").replace(". ", ",");
                            for part in normalized.split(',') {
                                let key = part.trim().to_lowercase();
                                if key.is_empty() {
                                    continue;
                                }
                                keyword_map
                                    .entry(key)
                                    .or_default()
                                    .push((fm.title.clone(), abs_label.clone()));
                            }
                        }
                    }
                    if idx + 1 < abs_len {
                        r#gen.push_str("#pagebreak()\n\n");
                    }
                }
            }
        } else {
            r#gen.push_str(&format!(
                "No content for locale \"{}\".\n",
                escape_typst_text(locale)
            ));
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
                r#gen.push_str("#pagebreak()\n#set page(header: none)\n");
                r#gen.push_str(
                    "#show heading.where(level: 1): it => block(above: 10pt, below: 10pt)[\n  #set text(size: 13pt, weight: \"bold\", font: \"Mari\")\n  #text(fill: brand-navy)[#it.body]\n]\n",
                );
                r#gen.push_str(&format!("= {}\n\n", escape_typst_text(&tag_index_label)));
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
                    r#gen.push_str(&format!(
                        "- {}: {}\n",
                        escape_typst_text(tag),
                        links.join("; ")
                    ));
                }
                r#gen.push('\n');
            }
        }

        if !link_map.is_empty() {
            r#gen.push_str("#pagebreak()\n#set page(header: none)\n");
            r#gen.push_str(
                "#show heading.where(level: 1): it => block(above: 10pt, below: 10pt)[\n  #set text(size: 13pt, weight: \"bold\", font: \"Mari\")\n  #text(fill: brand-blue)[#it.body]\n]\n",
            );
            r#gen.push_str(&format!("= {}\n\n", escape_typst_text(&link_index_label)));
            for (url, titles) in link_map.iter() {
                let mut uniq = titles.clone();
                uniq.sort_by(|a, b| a.0.cmp(&b.0));
                uniq.dedup_by(|a, b| a.1 == b.1);
                let internal_links: Vec<String> = uniq
                    .iter()
                    .map(|(title, label)| {
                        let title_text = escape_typst_text(title);
                        let anchor = format!(
                            "#link(<{}>)[{}] (#context counter(page).at(<{}>).at(0))",
                            label, title_text, label
                        );
                        format!("#underline[#text(fill: brand-navy)[{}]]", anchor)
                    })
                    .collect();
                let url_text = escape_typst_text(url);
                let url_link = format!(
                    "#underline[#text(fill: brand-navy)[#link(\"{}\")[{}]]]",
                    escape_typst_string(url),
                    url_text
                );
                r#gen.push_str(&format!("- {}: {}\n", url_link, internal_links.join("; ")));
            }
            r#gen.push('\n');
        }

        // Build a minimal validated Typst document to avoid template/comment
        // interpolation issues. This produces consistent output and is easy
        // to extend later with richer templates.
        let cover_header = escape_typst_text(&cover_header_label);
        let cover_title = escape_typst_text(&cover_title_label);
        let cover_symposium = escape_typst_text(&cover_symposium_label);
        let cover_date = escape_typst_text(&cover_date_label);
        let cover_subtitle = escape_typst_text(&cover_subtitle_label);
        let template_path = _template
            .clone()
            .unwrap_or_else(|| "templates/starter/book.typ".to_string());
        let template_text = read_to_string(&template_path)?;
        let mut out_text = template_text;
        out_text = out_text.replace("{{TITLE}}", &escape_typst_text(&title_label));
        out_text = out_text.replace("{{LOCALE}}", &escape_typst_text(locale));
        out_text = out_text.replace("{{TOC_LABEL}}", &escape_typst_text(&toc_label));
        out_text = out_text.replace("{{COVER_HEADER}}", &cover_header);
        out_text = out_text.replace("{{COVER_TITLE}}", &cover_title);
        out_text = out_text.replace("{{COVER_SYMPOSIUM}}", &cover_symposium);
        out_text = out_text.replace("{{COVER_DATE}}", &cover_date);
        out_text = out_text.replace("{{COVER_SUBTITLE}}", &cover_subtitle);
        out_text = out_text.replace("{{CONTENT}}", &format!("{}\n", r#gen));

        let mut f = File::create(&path)?;
        write!(f, "{}", out_text)?;
    }

    Ok(())
}

fn default_labels() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("title".to_string(), "Symposium 2026".to_string());
    m.insert("presenters_label".to_string(), "Presenters".to_string());
    m.insert("affiliation_label".to_string(), "Affiliation".to_string());
    m.insert("toc_label".to_string(), "Table of contents".to_string());
    m.insert("index_label".to_string(), "Index".to_string());
    m.insert("take_home_label".to_string(), "Take-home".to_string());
    m.insert("tags_label".to_string(), "Tags".to_string());
    m.insert("tag_index_label".to_string(), "Tag index".to_string());
    m.insert("link_index_label".to_string(), "Links".to_string());
    m.insert(
        "cover_header".to_string(),
        "Interprofessional Education Symposium 2026".to_string(),
    );
    m.insert("cover_title".to_string(), "PROGRAM".to_string());
    m.insert(
        "cover_symposium".to_string(),
        "Interprofessional Education Symposium".to_string(),
    );
    m.insert("cover_date".to_string(), "13 March 2026".to_string());
    m.insert(
        "cover_subtitle".to_string(),
        "The impact of intelligence on learning and guidance".to_string(),
    );
    m.insert(
        "more_info_prefix".to_string(),
        "For more info, click".to_string(),
    );
    m.insert("more_info_link_text".to_string(), "this link".to_string());
    m
}

fn tema_color(tema: &str) -> &str {
    for (idx, name) in TEMA_ORDER.iter().enumerate() {
        if tema == *name {
            return TEMA_COLORS.get(idx).copied().unwrap_or("#0070c0");
        }
    }
    "#0070c0"
}

fn tema_number(tema: &str) -> Option<usize> {
    for (idx, name) in TEMA_ORDER.iter().enumerate() {
        if tema == *name {
            return Some(idx + 1);
        }
    }
    None
}

fn escape_typst_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('*', "\\*")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn escape_typst_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
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

#[derive(Default)]
struct LabelState {
    used: std::collections::HashSet<String>,
    counter: u32,
}

impl LabelState {
    fn next(&mut self, fm: &FrontMatter) -> String {
        let base = label_for_abstract(fm);
        if self.used.insert(base.clone()) {
            return base;
        }
        loop {
            self.counter += 1;
            let candidate = format!("{}-{}", base, self.counter);
            if self.used.insert(candidate.clone()) {
                return candidate;
            }
        }
    }
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
    if let Some(a) = v.get("presenters_label").and_then(|s| s.as_str()) {
        m.insert("presenters_label".to_string(), a.to_string());
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
    if let Some(t) = v.get("link_index_label").and_then(|s| s.as_str()) {
        m.insert("link_index_label".to_string(), t.to_string());
    }
    if let Some(t) = v.get("cover_header").and_then(|s| s.as_str()) {
        m.insert("cover_header".to_string(), t.to_string());
    }
    if let Some(t) = v.get("cover_title").and_then(|s| s.as_str()) {
        m.insert("cover_title".to_string(), t.to_string());
    }
    if let Some(t) = v.get("cover_symposium").and_then(|s| s.as_str()) {
        m.insert("cover_symposium".to_string(), t.to_string());
    }
    if let Some(t) = v.get("cover_date").and_then(|s| s.as_str()) {
        m.insert("cover_date".to_string(), t.to_string());
    }
    if let Some(t) = v.get("cover_subtitle").and_then(|s| s.as_str()) {
        m.insert("cover_subtitle".to_string(), t.to_string());
    }
    if let Some(t) = v.get("more_info_prefix").and_then(|s| s.as_str()) {
        m.insert("more_info_prefix".to_string(), t.to_string());
    }
    if let Some(t) = v.get("more_info_link_text").and_then(|s| s.as_str()) {
        m.insert("more_info_link_text".to_string(), t.to_string());
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
                    .arg("--root")
                    .arg(".")
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
            "typst compile --root . --font-path {} {} {}",
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
