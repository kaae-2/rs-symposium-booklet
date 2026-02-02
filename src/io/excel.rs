use crate::model::{Abstract, AbstractSection, ItemRef, Session};
use anyhow::{Result, anyhow};
use calamine::{Data, Reader, open_workbook_auto};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

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

fn parse_env_file(path: &Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vars,
    };
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        let Some((key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = raw_value.trim();
        if value.len() >= 2 {
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = &value[1..value.len() - 1];
            }
        }
        if !value.is_empty() {
            vars.insert(key.to_string(), value.to_string());
        }
    }
    vars
}

fn resolve_env_path(dir: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        raw.to_string()
    } else if path.exists() {
        raw.to_string()
    } else {
        dir.join(path).to_string_lossy().to_string()
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
    let mut normalized = input.to_string();
    for needle in [" og ", " Og ", " OG "] {
        normalized = normalized.replace(needle, ";");
    }

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

fn push_session(
    sessions: &mut Vec<Session>,
    seen: &mut HashMap<String, u32>,
    title: String,
    items: &mut Vec<ItemRef>,
) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    let order = sessions.len() as u32 + 1;
    let base_id = title.clone();
    let count = seen.entry(base_id.clone()).or_insert(0);
    *count += 1;
    let id = if *count == 1 {
        base_id.clone()
    } else {
        format!("{}_{}", base_id, count)
    };
    let title = if *count == 1 {
        title
    } else {
        format!("{}_{}", title, count)
    };
    sessions.push(Session {
        id,
        title,
        order,
        items: std::mem::take(items),
    });
    Ok(())
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

    let col_id = find_col(&["id"]).ok_or_else(|| anyhow!("id column not found in abstracts"))?;
    let col_title = find_col(&["title", "titel"])
        .ok_or_else(|| anyhow!("title column not found in abstracts"))?;
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

        if !aid.is_empty() {
            if title.trim().is_empty() {
                return Err(anyhow!(
                    "Missing title for abstract id {} at row {}",
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

fn find_sheet_by_substr(path: &str, subs: &[&str]) -> Result<String> {
    let wb = open_workbook_auto(path).map_err(|e| anyhow!("open failed: {}", e))?;
    for name in wb.sheet_names() {
        let low = name.to_lowercase();
        for &s in subs {
            if low.contains(&s.to_lowercase()) {
                return Ok(name.clone());
            }
        }
    }
    // fallback to first sheet
    wb.sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("no sheets in workbook {}", path))
}

pub fn parse_workbook(path: &str) -> Result<(HashMap<String, Abstract>, Vec<Session>)> {
    // if `path` is a directory, find two xlsx files and parse accordingly
    if Path::new(path).is_dir() {
        let input_dir = Path::new(path);
        let mut xls = Vec::new();
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
        let mut env_overrides: HashMap<String, String> = HashMap::new();
        if let Ok(val) = env::var("SYMPOSIUM_ABSTRACTS") {
            env_overrides.insert("SYMPOSIUM_ABSTRACTS".to_string(), val);
        }
        if let Ok(val) = env::var("SYMPOSIUM_GROUPING") {
            env_overrides.insert("SYMPOSIUM_GROUPING".to_string(), val);
        }
        if env_overrides.get("SYMPOSIUM_ABSTRACTS").is_none()
            || env_overrides.get("SYMPOSIUM_GROUPING").is_none()
        {
            let env_path = input_dir.join(".env");
            if env_path.exists() {
                let file_vars = parse_env_file(&env_path);
                if env_overrides.get("SYMPOSIUM_ABSTRACTS").is_none() {
                    if let Some(val) = file_vars.get("SYMPOSIUM_ABSTRACTS") {
                        env_overrides.insert("SYMPOSIUM_ABSTRACTS".to_string(), val.clone());
                    }
                }
                if env_overrides.get("SYMPOSIUM_GROUPING").is_none() {
                    if let Some(val) = file_vars.get("SYMPOSIUM_GROUPING") {
                        env_overrides.insert("SYMPOSIUM_GROUPING".to_string(), val.clone());
                    }
                }
            }
        }

        // prefer with_ids.xlsx as abstracts file
        let mut file_a = None::<String>;
        let mut file_b = None::<String>;
        if let Some(val) = env_overrides.get("SYMPOSIUM_ABSTRACTS") {
            file_a = Some(resolve_env_path(input_dir, val));
        }
        if let Some(val) = env_overrides.get("SYMPOSIUM_GROUPING") {
            file_b = Some(resolve_env_path(input_dir, val));
        }
        for f in &xls {
            if file_a.is_none()
                && (f.to_lowercase().contains("with_ids")
                    || f.to_lowercase().contains("afsluttede"))
            {
                file_a = Some(f.clone())
            }
            if file_b.is_none()
                && (f.to_lowercase().contains("kopi")
                    || f.to_lowercase().contains("grupper")
                    || f.to_lowercase().contains("final"))
            {
                file_b = Some(f.clone());
            }
        }
        if file_a.is_none() {
            file_a = xls.first().cloned();
        }
        if file_b.is_none() {
            if xls.len() > 1 {
                file_b = xls.get(1).cloned();
            } else {
                file_b = file_a.clone();
            }
        }
        let file_a = file_a.ok_or_else(|| anyhow!("failed to choose abstracts file"))?;
        let file_b = file_b.ok_or_else(|| anyhow!("failed to choose grouping file"))?;

        // now parse abstracts from file_a and sessions from file_b
        return parse_two_workbooks(&file_a, &file_b);
    }

    // existing single-workbook logic (both sheets in one workbook)
    let mut wb = open_workbook_auto(path).map_err(|e| anyhow!("Failed to open workbook: {}", e))?;

    // identify candidate sheet names
    let names = wb.sheet_names().to_owned();
    if names.is_empty() {
        return Err(anyhow!("Workbook has no sheets"));
    }

    // heuristics for abstracts/session sheets (case-insensitive)
    let mut abstracts_sheet: Option<String> = None;
    let mut sessions_sheet: Option<String> = None;
    for n in &names {
        let low = n.to_lowercase();
        if abstracts_sheet.is_none()
            && (low.contains("afsluttede")
                || low.contains("abstract")
                || low.contains("afsluttet")
                || low.contains("resum"))
        {
            abstracts_sheet = Some(n.clone());
        }
        if sessions_sheet.is_none()
            && (low.contains("gruppering")
                || low.contains("grupper")
                || low.contains("poster")
                || low.contains("session")
                || low.contains("include"))
        {
            sessions_sheet = Some(n.clone());
        }
    }

    let abstracts_sheet = abstracts_sheet.ok_or_else(|| {
        anyhow!("No abstracts sheet found (tried matching 'afsluttede','abstract','resum')")
    })?;
    let sessions_sheet = sessions_sheet.ok_or_else(|| anyhow!("No sessions/include sheet found (tried matching 'gruppering','poster','session','include')"))?;

    tracing::info!("Parsing abstracts sheet: {}", abstracts_sheet);

    // load rows for abstracts sheet
    let range = wb
        .worksheet_range(&abstracts_sheet)
        .map_err(|e| anyhow!("Failed to get range for sheet {}: {}", abstracts_sheet, e))?;
    let mut rows_a: Vec<Vec<String>> = Vec::new();
    for r in range.rows() {
        rows_a.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    // detect header row
    let header_idx = find_header_row(&rows_a, &[])
        .ok_or_else(|| anyhow!("Could not detect header row in abstracts sheet"))?;
    let abstract_map = parse_abstracts_from_rows(&rows_a, header_idx)?;

    // parse sessions sheet using flexible heuristics (header rows vs item rows)
    tracing::info!("Parsing sessions sheet: {}", sessions_sheet);
    let range_b = wb
        .worksheet_range(&sessions_sheet)
        .map_err(|e| anyhow!("Failed to get range for sheet {}: {}", sessions_sheet, e))?;
    let mut rows_b: Vec<Vec<String>> = Vec::new();
    for r in range_b.rows() {
        rows_b.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    // try to detect a header row (first non-empty row with 'id' or 'abstract')
    let mut sessions: Vec<Session> = Vec::new();
    let mut seen_session_ids: HashMap<String, u32> = HashMap::new();
    let mut current_session_title = None::<String>;
    let mut current_items: Vec<ItemRef> = Vec::new();
    let mut item_counter = 1u32;

    // helper to flush current session
    let flush_session = |sessions: &mut Vec<Session>,
                         seen: &mut HashMap<String, u32>,
                         title: Option<String>,
                         items: &mut Vec<ItemRef>|
     -> Result<()> {
        let title = title.unwrap_or_else(|| "(unnamed)".to_string());
        push_session(sessions, seen, title, items)
    };

    for row in rows_b.iter() {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        // try to find any token that looks like an abstract id present in abstract_map
        let mut found_ids: Vec<String> = Vec::new();
        for c in row.iter() {
            if c.trim().is_empty() {
                continue;
            }
            let token = c.trim();
            if abstract_map.contains_key(token) {
                found_ids.push(token.to_string());
                continue;
            }
            for part in token.replace(';', ",").split(',').map(|s| s.trim()) {
                if abstract_map.contains_key(part) {
                    found_ids.push(part.to_string());
                }
            }
        }

        if !found_ids.is_empty() {
            // this row contains item(s)
            if current_session_title.is_none() {
                current_session_title = Some("(unnamed)".to_string());
            }
            for fid in found_ids.into_iter() {
                current_items.push(ItemRef {
                    id: fid,
                    order: item_counter,
                });
                item_counter += 1;
            }
        } else {
            // treat as session header
            // flush previous
            flush_session(
                &mut sessions,
                &mut seen_session_ids,
                current_session_title.take(),
                &mut current_items,
            )?;
            // set new title
            let textcells: Vec<String> = row
                .iter()
                .filter(|c| !c.trim().is_empty())
                .cloned()
                .collect();
            let title = textcells.join(" ").trim().to_string();
            current_session_title = Some(if title.is_empty() {
                "(unnamed)".to_string()
            } else {
                title
            });
            item_counter = 1;
        }
    }
    // flush last
    flush_session(
        &mut sessions,
        &mut seen_session_ids,
        current_session_title.take(),
        &mut current_items,
    )?;

    // determine referenced set
    let mut referenced: HashSet<String> = HashSet::new();
    for s in &sessions {
        for it in &s.items {
            referenced.insert(it.id.clone());
        }
    }

    // Unreferenced abstracts are not added to an automatic session.

    Ok((abstract_map, sessions))
}

pub fn parse_two_workbooks(
    file_a: &str,
    file_b: &str,
) -> Result<(HashMap<String, Abstract>, Vec<Session>)> {
    tracing::info!(
        "Parsing abstracts from {} and sessions from {}",
        file_a,
        file_b
    );
    // load rows A
    let sheet_a = find_sheet_by_substr(file_a, &["afsluttede", "abstract"])?;
    let range_a = open_workbook_auto(file_a)?
        .worksheet_range(&sheet_a)
        .map_err(|e| anyhow!("Failed to read sheet {} from {}: {}", sheet_a, file_a, e))?;
    let mut rows_a: Vec<Vec<String>> = Vec::new();
    for r in range_a.rows() {
        rows_a.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    let header_idx = find_header_row(&rows_a, &[])
        .ok_or_else(|| anyhow!("Could not detect header row in abstracts sheet"))?;
    let abstract_map = parse_abstracts_from_rows(&rows_a, header_idx)?;

    // load rows B
    let sheet_b = match find_sheet_by_substr(file_b, &["gruppering", "grupper", "poster"]) {
        Ok(s) => s,
        Err(_) => match open_workbook_auto(file_b) {
            Ok(wb) => wb
                .sheet_names()
                .first()
                .cloned()
                .unwrap_or_else(|| "Sheet1".to_string()),
            Err(_) => "Sheet1".to_string(),
        },
    };
    let range_b = open_workbook_auto(file_b)?
        .worksheet_range(&sheet_b)
        .map_err(|e| anyhow!("Failed to read sheet {} from {}: {}", sheet_b, file_b, e))?;
    let mut rows_b: Vec<Vec<String>> = Vec::new();
    for r in range_b.rows() {
        rows_b.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    // parse sessions from rows_b (same heuristics as single workbook case)
    let mut sessions: Vec<Session> = Vec::new();
    let mut seen_session_ids: HashMap<String, u32> = HashMap::new();
    let mut current_session_title = None::<String>;
    let mut current_items: Vec<ItemRef> = Vec::new();
    let mut item_counter = 1u32;

    let flush_session = |sessions: &mut Vec<Session>,
                         seen: &mut HashMap<String, u32>,
                         title: Option<String>,
                         items: &mut Vec<ItemRef>|
     -> Result<()> {
        let title = title.unwrap_or_else(|| "(unnamed)".to_string());
        push_session(sessions, seen, title, items)
    };

    for row in rows_b.iter() {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let mut found_ids: Vec<String> = Vec::new();
        for c in row.iter() {
            if c.trim().is_empty() {
                continue;
            }
            let token = c.trim();
            if abstract_map.contains_key(token) {
                found_ids.push(token.to_string());
                continue;
            }
            for part in token.replace(';', ",").split(',').map(|s| s.trim()) {
                if abstract_map.contains_key(part) {
                    found_ids.push(part.to_string());
                }
            }
        }

        if !found_ids.is_empty() {
            if current_session_title.is_none() {
                current_session_title = Some("(unnamed)".to_string());
            }
            for fid in found_ids.into_iter() {
                current_items.push(ItemRef {
                    id: fid,
                    order: item_counter,
                });
                item_counter += 1;
            }
        } else {
            flush_session(
                &mut sessions,
                &mut seen_session_ids,
                current_session_title.take(),
                &mut current_items,
            )?;
            let textcells: Vec<String> = row
                .iter()
                .filter(|c| !c.trim().is_empty())
                .cloned()
                .collect();
            let title = textcells.join(" ").trim().to_string();
            current_session_title = Some(if title.is_empty() {
                "(unnamed)".to_string()
            } else {
                title
            });
            item_counter = 1;
        }
    }
    flush_session(
        &mut sessions,
        &mut seen_session_ids,
        current_session_title.take(),
        &mut current_items,
    )?;

    // determine referenced set and add Unassigned for unreferenced
    let mut referenced: HashSet<String> = HashSet::new();
    for s in &sessions {
        for it in &s.items {
            referenced.insert(it.id.clone());
        }
    }
    // Unreferenced abstracts are not added to an automatic session.

    Ok((abstract_map, sessions))
}
