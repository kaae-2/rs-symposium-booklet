use crate::model::{Abstract, AbstractSection, ItemRef, Session};
use anyhow::{Result, anyhow};
use calamine::{Data, Reader, open_workbook_auto};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const TEMA_ORDER: [&str; 3] = ["Miljø", "Teknologi", "Organisation"];
const TYPE_ORDER: [&str; 2] = ["Poster", "Mundtlig"];

fn as_str(cell: Option<&Data>) -> String {
    match cell {
        None => "".to_string(),
        Some(c) => match c {
            Data::Empty => "".to_string(),
            Data::String(s) => s.trim().to_string(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            _ => format!("{}", c),
        },
    }
}

fn detect_locale(header_row: &[String], row: &[String]) -> String {
    let mut col_locale: Option<usize> = None;
    for (j, cell) in header_row.iter().enumerate() {
        let low = cell.to_lowercase();
        if low.contains("locale") || low.contains("sprog") {
            col_locale = Some(j);
            break;
        }
    }
    col_locale
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "da".to_string())
}

fn normalize_author_separators(input: &str) -> String {
    let normalized = input.to_string();
    // for needle in [" og ", " Og ", " OG "] {
    //     normalized = normalized.replace(needle, ";");
    // }

    let mut out = String::new();
    let mut ws_count = 0usize;
    for ch in normalized.chars() {
        if ch.is_whitespace() {
            ws_count += 1;
            continue;
        }
        if ws_count > 0 {
            if ws_count >= 2 {
                out.push(';');
            } else {
                out.push(' ');
            }
            ws_count = 0;
        }
        out.push(ch);
    }
    if ws_count > 0 {
        if ws_count >= 2 {
            out.push(';');
        } else {
            out.push(' ');
        }
    }
    out
}

fn parse_presenters_and_affiliation(input: &str) -> (Vec<String>, Option<String>) {
    let normalized = normalize_author_separators(input);
    let mut presenters: Vec<String> = Vec::new();

    for raw in normalized.split(';') {
        let chunk = raw.trim();
        if chunk.is_empty() {
            continue;
        }
        let cleaned = chunk.split_whitespace().collect::<Vec<_>>().join(" ");
        if !cleaned.is_empty() {
            presenters.push(cleaned);
        }
    }

    (presenters, None)
}

fn parse_tema(raw: &str, row: usize) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Missing tema at row {}", row));
    }
    for allowed in TEMA_ORDER.iter() {
        if trimmed == *allowed {
            return Ok(allowed.to_string());
        }
    }
    Err(anyhow!(
        "Invalid tema '{}' at row {} (expected: Miljø, Teknologi, Organisation)",
        trimmed,
        row
    ))
}

fn parse_presentation_type(raw: &str, row: usize) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Missing type at row {}", row));
    }
    for allowed in TYPE_ORDER.iter() {
        if trimmed == *allowed {
            return Ok(allowed.to_string());
        }
    }
    Err(anyhow!(
        "Invalid type '{}' at row {} (expected: Poster, Mundtlig)",
        trimmed,
        row
    ))
}

fn parse_order(raw: &str, row: usize) -> Result<u32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Missing order at row {}", row));
    }
    if let Ok(val) = trimmed.parse::<u32>() {
        return Ok(val);
    }
    if let Ok(val) = trimmed.parse::<f64>() {
        if val.is_finite() && val.fract() == 0.0 && val >= 0.0 {
            return Ok(val as u32);
        }
    }
    Err(anyhow!(
        "Invalid order '{}' at row {} (expected integer)",
        trimmed,
        row
    ))
}


pub fn find_header_row(rows: &[Vec<String>], _candidates: &[&str]) -> Option<usize> {
    for (i, row) in rows.iter().take(12).enumerate() {
        let lowered: Vec<String> = row.iter().map(|c| c.to_lowercase()).collect();
        let mut has_id = false;
        let mut has_title = false;
        for cell in lowered.iter() {
            if cell.contains("id") {
                has_id = true;
            }
            if cell.contains("title")
                || cell.contains("titel")
                || cell.contains("abstract")
                || cell.contains("resum")
            {
                has_title = true;
            }
        }
        if has_id && has_title {
            return Some(i);
        }
    }
    None
}

fn chars_eq_case_insensitive(a: char, b: char) -> bool {
    let mut a_lower = a.to_lowercase();
    let mut b_lower = b.to_lowercase();
    a_lower.next() == b_lower.next() && a_lower.next().is_none() && b_lower.next().is_none()
}

fn match_label_at(chars: &[(usize, char)], start: usize, label: &str) -> Option<usize> {
    let mut idx = start;
    for label_ch in label.chars() {
        if idx >= chars.len() {
            return None;
        }
        let ch = chars[idx].1;
        if !chars_eq_case_insensitive(ch, label_ch) {
            return None;
        }
        idx += 1;
    }
    Some(idx)
}

fn skip_whitespace(chars: &[(usize, char)], mut idx: usize) -> usize {
    while idx < chars.len() && chars[idx].1.is_whitespace() {
        idx += 1;
    }
    idx
}

fn capitalize_label(input: &str) -> String {
    let mut out = String::new();
    let mut start_word = true;
    for ch in input.chars() {
        if ch.is_whitespace() {
            start_word = true;
            out.push(ch);
            continue;
        }
        if start_word {
            for upper in ch.to_uppercase() {
                out.push(upper);
            }
            start_word = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn normalize_section_label(input: &str) -> String {
    let trimmed = input.trim();
    let trimmed = trimmed.trim_end_matches(|c: char| c == ',' || c == '.' || c == ':');
    let trimmed = trimmed.trim();
    capitalize_label(trimmed)
}

fn default_section_label(locale: &str) -> String {
    if locale.to_lowercase().starts_with("da") {
        "Resumé".to_string()
    } else {
        "Abstract".to_string()
    }
}

fn slice_text(input: &str, chars: &[(usize, char)], start_idx: usize, end_idx: usize) -> String {
    if input.is_empty() || start_idx >= chars.len() {
        return String::new();
    }
    let start_byte = chars[start_idx].0;
    let end_byte = if end_idx >= chars.len() {
        input.len()
    } else {
        chars[end_idx].0
    };
    input[start_byte..end_byte].to_string()
}

fn split_abstract_sections(input: &str, locale: &str) -> Vec<AbstractSection> {
    let labels = [
        "Baggrund",
        "Formål",
        "Metode og materiale",
        "Metode og materialer",
        "Metode",
        "Resultater",
        "Diskussion",
        "Konklusion og perspektiver",
        "Konklusion og perspektivering",
        "Konklusion",
        "Perspektivering",
        "Background",
        "Objective",
        "Aim",
        "Purpose",
        "Methods and materials",
        "Materials and methods",
        "Methods",
        "Results",
        "Discussion",
        "Conclusion",
    ];
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut sections: Vec<AbstractSection> = Vec::new();
    let mut idx = 0;
    let mut current_start = 0;
    let mut current_label: Option<String> = None;
    let default_label = default_section_label(locale);

    while idx < chars.len() {
        let mut matched: Option<(String, usize, usize)> = None;
        let mut prev_idx = idx;
        let mut prev_non_ws = None;
        let mut saw_line_break = false;
        while prev_idx > 0 {
            prev_idx -= 1;
            let ch = chars[prev_idx].1;
            if ch == '\n' || ch == '\r' {
                saw_line_break = true;
            }
            if !ch.is_whitespace() {
                prev_non_ws = Some(ch);
                break;
            }
        }
        let prev_is_boundary = saw_line_break
            || prev_non_ws
                .map(|ch| ch == '/' || ch == ':' || ch == '.' || ch == ',' || ch == ';')
                .unwrap_or(true);
        let at_boundary = idx == 0 || prev_is_boundary;

        if !at_boundary {
            idx += 1;
            continue;
        }
        for label in labels.iter() {
            if let Some(after_label) = match_label_at(&chars, idx, label) {
                let after_space = skip_whitespace(&chars, after_label);
                let has_space = after_space > after_label;
                let label_raw = slice_text(input, &chars, idx, after_label);
                let mut label_norm = normalize_section_label(&label_raw);
                if locale.to_lowercase().starts_with("da") {
                    label_norm = label_norm.replace(" Og ", " og ");
                }
                if after_space < chars.len() {
                    let delim = chars[after_space].1;
                    if delim == '/' || delim == ':' || delim == '.' || delim == ',' || delim == ';'
                    {
                        let after_delim = skip_whitespace(&chars, after_space + 1);
                        matched = Some((label_norm, idx, after_delim));
                        break;
                    }
                    if has_space && (delim.is_uppercase() || delim.is_ascii_digit()) {
                        matched = Some((label_norm, idx, after_space));
                        break;
                    }
                    if has_space && prev_is_boundary {
                        matched = Some((label_norm, idx, after_space));
                        break;
                    }
                } else {
                    matched = Some((label_norm, idx, after_space));
                    break;
                }
            }
        }

        if let Some((label, label_start, content_start)) = matched {
            let pre_text = slice_text(input, &chars, current_start, label_start);
            let pre_text = pre_text.trim().to_string();
            if let Some(prev_label) = current_label.take() {
                if !pre_text.is_empty() {
                    sections.push(AbstractSection {
                        label: prev_label,
                        text: pre_text,
                    });
                }
            } else if !pre_text.is_empty() {
                sections.push(AbstractSection {
                    label: default_label.clone(),
                    text: pre_text,
                });
            }
            current_label = Some(label);
            current_start = content_start;
            idx = content_start;
            continue;
        }

        idx += 1;
    }

    let tail = if current_start < chars.len() {
        slice_text(input, &chars, current_start, chars.len())
    } else {
        String::new()
    };
    let tail = tail.trim().to_string();
    if let Some(label) = current_label.take() {
        if !tail.is_empty() {
            sections.push(AbstractSection { label, text: tail });
        }
    } else if !tail.is_empty() {
        sections.push(AbstractSection {
            label: default_label,
            text: tail,
        });
    }

    sections
}

fn sanitize_abstract_text(input: &str) -> String {
    input.trim().to_string()
}

fn trim_url_punctuation(input: &str) -> String {
    let mut out = input
        .trim()
        .trim_matches(|c: char| matches!(c, '<' | '>' | '(' | ')' | '[' | ']' | '"' | '\''))
        .to_string();
    loop {
        let trimmed = out.trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ':'));
        if trimmed.len() == out.len() {
            break;
        }
        out = trimmed.to_string();
    }
    out
}

fn extract_url_from(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    if let Some(pos) = lower.find("https://") {
        let token = input[pos..].split_whitespace().next().unwrap_or("");
        let url = trim_url_punctuation(token);
        if !url.is_empty() {
            return Some(url);
        }
    }
    if let Some(pos) = lower.find("http://") {
        let token = input[pos..].split_whitespace().next().unwrap_or("");
        let url = trim_url_punctuation(token);
        if !url.is_empty() {
            return Some(url);
        }
    }
    if let Some(pos) = lower.find("www.") {
        let token = input[pos..].split_whitespace().next().unwrap_or("");
        let url = trim_url_punctuation(token);
        if !url.is_empty() {
            return Some(format!("https://{}", url));
        }
    }
    None
}

fn extract_doi_token(input: &str) -> Option<String> {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_whitespace() {
            break;
        }
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '/' | '_' | ';' | '(' | ')') {
            out.push(ch);
            continue;
        }
        if ch == ':' && out.is_empty() {
            continue;
        }
        break;
    }
    while out.ends_with(|c: char| matches!(c, '.' | ',' | ';' | ':' | ')')) {
        out.pop();
    }
    if out.is_empty() { None } else { Some(out) }
}

fn extract_doi(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    if let Some(pos) = lower.find("doi.org/") {
        let after = &input[pos + "doi.org/".len()..];
        return extract_doi_token(after);
    }
    if let Some(pos) = lower.find("doi:") {
        let after = &input[pos + "doi:".len()..];
        if let Some(token) = extract_doi_token(after) {
            return Some(token);
        }
    }
    if let Some(pos) = lower.find("10.") {
        let after = &input[pos..];
        return extract_doi_token(after);
    }
    None
}

fn extract_reference_link(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(url) = extract_url_from(trimmed) {
        return Some(url);
    }
    if let Some(doi) = extract_doi(trimmed) {
        return Some(format!("https://doi.org/{}", doi));
    }
    None
}

// Extract parsing of abstracts from a rows buffer into a helper so tests can exercise
// duplicate-id handling and header-detection without needing actual workbook files.
pub fn parse_abstracts_from_rows(
    rows_a: &[Vec<String>],
    header_idx: usize,
) -> Result<HashMap<String, Abstract>> {
    let normalize_ws =
        |input: &str| -> String { input.split_whitespace().collect::<Vec<_>>().join(" ") };
    let header_row = &rows_a[header_idx];
    let lower_row: Vec<String> = header_row
        .iter()
        .map(|s| normalize_ws(s).to_lowercase())
        .collect();
    let find_col = |subs: &[&str]| -> Option<usize> {
        for (j, cell) in lower_row.iter().enumerate() {
            for &s in subs {
                if cell.contains(&normalize_ws(s).to_lowercase()) {
                    return Some(j);
                }
            }
        }
        None
    };
    let find_col_exact = |name: &str| -> Option<usize> {
        for (j, cell) in header_row.iter().enumerate() {
            if cell.trim().eq_ignore_ascii_case(name) {
                return Some(j);
            }
        }
        None
    };

    let col_id = find_col(&["id"]).ok_or_else(|| anyhow!("id column not found in abstracts"))?;
    let col_title = find_col(&["title", "titel"])
        .ok_or_else(|| anyhow!("title column not found in abstracts"))?;
    let col_tema = find_col_exact("tema").ok_or_else(|| anyhow!("tema column not found in abstracts"))?;
    let col_type = find_col_exact("type").ok_or_else(|| anyhow!("type column not found in abstracts"))?;
    let col_order = find_col_exact("order").ok_or_else(|| anyhow!("order column not found in abstracts"))?;
    let col_presenter = find_col(&[
        "hvem præsenterer projektet? navn, titel, tilhørsforhold (afdeling, hospital eller andet fx institut, universitet).",
        "hvem præsenterer projektet? navn, titel, tilhørsforhold (afdeling, hospital eller andet fx institut, universitet)",
        "hvem præsenterer projektet? navn, titel, tilhørsforhold",
        "hvem præsenterer projektet",
        "præsenterer projektet",
    ]);
    let col_presenters = find_col(&["presenter", "presenters", "authors", "author", "forfatter"]);
    if col_presenter.is_none() && col_presenters.is_none() {
        return Err(anyhow!("presenters column not found in abstracts"));
    }
    let col_abstract = find_col(&["abstract", "resum", "resumé"])
        .ok_or_else(|| anyhow!("abstract column not found in abstracts"))?;
    let col_keywords = find_col(&["keyword", "keywords", "nøgle", "emne ord", "emneord"]);
    let col_takehome = find_col(&["take home", "take-home", "takehome", "take home messages"]);
    let col_reference = find_col(&[
        "reference",
        "published",
        "doi",
        "reference hvis studiet er publiceret",
        "link eller doi",
    ]);
    let col_literature = find_col(&["litterature", "literature", "references", "literatur"]);
    let col_center = find_col(&["center", "centre", "center/centre"]);
    let col_contact = find_col(&["email", "kontakt", "contact"]);

    let mut abstracts: Vec<Abstract> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (ridx, row) in rows_a.iter().enumerate().skip(header_idx + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let aid = row.get(col_id).map(|s| normalize_ws(s)).unwrap_or_default();
        let title = row
            .get(col_title)
            .map(|s| normalize_ws(s))
            .unwrap_or_default();
        let tema_raw = row.get(col_tema).map(|s| s.trim().to_string()).unwrap_or_default();
        let type_raw = row.get(col_type).map(|s| s.trim().to_string()).unwrap_or_default();
        let order_raw = row.get(col_order).map(|s| s.trim().to_string()).unwrap_or_default();
        let presenters_raw = col_presenters
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let presenter_raw = col_presenter
            .and_then(|idx| row.get(idx))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text_raw = row
            .get(col_abstract)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text_sanitized = sanitize_abstract_text(&abstract_text_raw);
        let locale_val = detect_locale(header_row, row);
        let abstract_sections = split_abstract_sections(&abstract_text_sanitized, &locale_val);
        let abstract_text = abstract_text_raw.trim().to_string();
        let keywords = col_keywords
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .unwrap_or_default();
        let take_home = col_takehome
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .filter(|s| !s.is_empty());
        let reference_raw = col_reference
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .unwrap_or_default();
        let reference = extract_reference_link(&reference_raw);
        let literature = col_literature
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .filter(|s| !s.is_empty());
        let center = col_center
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .filter(|s| !s.is_empty());
        let contact = col_contact
            .and_then(|idx| row.get(idx))
            .map(|s| normalize_ws(s))
            .filter(|s| !s.is_empty());

        if aid.is_empty() && title.is_empty() && abstract_text.is_empty() {
            continue;
        }

        let has_any_ordering = !(tema_raw.trim().is_empty()
            && type_raw.trim().is_empty()
            && order_raw.trim().is_empty());
        let has_all_ordering = !(tema_raw.trim().is_empty()
            || type_raw.trim().is_empty()
            || order_raw.trim().is_empty());
        if !has_any_ordering {
            continue;
        }
        if !has_all_ordering {
            let warn_id = if aid.is_empty() { "<missing id>" } else { &aid };
            tracing::warn!(
                "Row {} (id {}) has partial tema/type/order values",
                ridx + 1,
                warn_id
            );
        }

        if !aid.is_empty() {
            if title.trim().is_empty() {
                return Err(anyhow!(
                    "Missing title for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if tema_raw.trim().is_empty() {
                return Err(anyhow!(
                    "Missing tema for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if type_raw.trim().is_empty() {
                return Err(anyhow!(
                    "Missing type for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if order_raw.trim().is_empty() {
                return Err(anyhow!(
                    "Missing order for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if presenter_raw.trim().is_empty() && presenters_raw.trim().is_empty() {
                return Err(anyhow!(
                    "Missing presenters for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if abstract_text_raw.trim().is_empty() {
                return Err(anyhow!(
                    "Missing abstract text for abstract id {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            if seen.contains_key(&aid) {
                return Err(anyhow!(
                    "Duplicate abstract id found: {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            seen.insert(aid.clone(), ridx + 1);
        }

        let tema = parse_tema(&tema_raw, ridx + 1)?;
        let presentation_type = parse_presentation_type(&type_raw, ridx + 1)?;
        let order = parse_order(&order_raw, ridx + 1)?;
        let (presenters_vec, affiliation) = if !presenter_raw.is_empty() {
            parse_presenters_and_affiliation(&presenter_raw)
        } else {
            parse_presenters_and_affiliation(&presenters_raw)
        };
        let keywords_vec = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        abstracts.push(Abstract {
            id: aid.clone(),
            title: title.clone(),
            tema,
            presentation_type,
            order,
            presenters: presenters_vec,
            affiliation,
            center,
            contact_email: contact,
            abstract_text: abstract_text.clone(),
            abstract_sections,
            keywords: keywords_vec,
            take_home,
            reference,
            literature,
            locale: locale_val,
        });
    }

    // build id map
    let mut abstract_map: HashMap<String, Abstract> = HashMap::new();
    for a in abstracts.into_iter() {
        if !a.id.is_empty() {
            abstract_map.insert(a.id.clone(), a);
        }
    }

    Ok(abstract_map)
}

fn build_sessions_from_abstracts(abstracts: &HashMap<String, Abstract>) -> Vec<Session> {
    let mut sessions: Vec<Session> = Vec::new();
    let mut order_counter: u32 = 1;

    for tema in TEMA_ORDER.iter() {
        for presentation_type in TYPE_ORDER.iter() {
            let mut grouped: Vec<&Abstract> = abstracts
                .values()
                .filter(|a| a.tema == *tema && a.presentation_type == *presentation_type)
                .collect();
            if grouped.is_empty() {
                continue;
            }
            grouped.sort_by(|a, b| {
                a.order
                    .cmp(&b.order)
                    .then_with(|| a.id.cmp(&b.id))
                    .then_with(|| a.title.cmp(&b.title))
            });

            let items: Vec<ItemRef> = grouped
                .iter()
                .map(|a| ItemRef {
                    id: a.id.clone(),
                    order: a.order,
                })
                .collect();

            let title = format!("{} - {}", tema, presentation_type);
            sessions.push(Session {
                id: title.clone(),
                title,
                tema: tema.to_string(),
                presentation_type: presentation_type.to_string(),
                order: order_counter,
                items,
            });
            order_counter += 1;
        }
    }

    sessions
}

pub fn parse_workbook(path: &str) -> Result<(HashMap<String, Abstract>, Vec<Session>)> {
    let source_path = if Path::new(path).is_dir() {
        let mut xls: Vec<String> = Vec::new();
        for entry in fs::read_dir(path)? {
            let e = entry?;
            let p = e.path();
            if let Some(ext) = p.extension()
                && ext == "xlsx"
            {
                if let Some(fname) = p.file_name().and_then(|s| s.to_str())
                    && fname.starts_with("~$")
                {
                    continue;
                }
                xls.push(p.to_string_lossy().to_string());
            }
        }
        if xls.is_empty() {
            return Err(anyhow!("No .xlsx files found in directory {}", path));
        }
        xls.sort();
        if let Some(found) = xls
            .iter()
            .find(|f| f.to_lowercase().contains("with_ids") || f.to_lowercase().contains("abstract"))
        {
            found.clone()
        } else {
            xls.first().cloned().ok_or_else(|| anyhow!("failed to choose abstracts file"))?
        }
    } else {
        path.to_string()
    };

    let mut wb =
        open_workbook_auto(&source_path).map_err(|e| anyhow!("Failed to open workbook: {}", e))?;
    let names = wb.sheet_names().to_owned();
    if names.is_empty() {
        return Err(anyhow!("Workbook has no sheets"));
    }

    let mut abstracts_sheet: Option<String> = None;
    for n in &names {
        let low = n.to_lowercase();
        if low.contains("afsluttede")
            || low.contains("abstract")
            || low.contains("afsluttet")
            || low.contains("resum")
        {
            abstracts_sheet = Some(n.clone());
            break;
        }
    }
    let abstracts_sheet = abstracts_sheet.ok_or_else(|| {
        anyhow!("No abstracts sheet found (tried matching 'afsluttede','abstract','resum')")
    })?;

    tracing::info!("Parsing abstracts sheet: {}", abstracts_sheet);
    let range = wb
        .worksheet_range(&abstracts_sheet)
        .map_err(|e| anyhow!("Failed to get range for sheet {}: {}", abstracts_sheet, e))?;
    let mut rows_a: Vec<Vec<String>> = Vec::new();
    for r in range.rows() {
        rows_a.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    let header_idx = find_header_row(&rows_a, &[])
        .ok_or_else(|| anyhow!("Could not detect header row in abstracts sheet"))?;
    let abstract_map = parse_abstracts_from_rows(&rows_a, header_idx)?;
    let sessions = build_sessions_from_abstracts(&abstract_map);

    Ok((abstract_map, sessions))
}
