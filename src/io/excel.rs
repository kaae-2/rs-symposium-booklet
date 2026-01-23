use crate::model::{Abstract, ItemRef, Session};
use anyhow::{anyhow, Result};
use calamine::{open_workbook_auto, DataType, Reader};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn as_str(cell: Option<&DataType>) -> String {
    match cell {
        None => "".to_string(),
        Some(c) => match c {
            DataType::Empty => "".to_string(),
            DataType::String(s) => s.trim().to_string(),
            DataType::Float(f) => f.to_string(),
            DataType::Int(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
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
    let mut normalized = input.to_string();
    for needle in [" og ", " Og ", " OG "] {
        normalized = normalized.replace(needle, ";");
    }
    normalized
}

fn parse_authors_and_affiliation(input: &str) -> (Vec<String>, Option<String>) {
    let normalized = normalize_author_separators(input);
    let mut authors: Vec<String> = Vec::new();
    let mut affiliations: Vec<String> = Vec::new();

    for raw in normalized.split(';') {
        let chunk = raw.trim();
        if chunk.is_empty() {
            continue;
        }
        let parts: Vec<String> = chunk
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|p| p.to_string())
            .collect();
        if parts.is_empty() {
            continue;
        }
        authors.push(parts[0].clone());
        if parts.len() > 1 {
            let affiliation = parts[parts.len() - 1].clone();
            if !affiliation.is_empty() && !affiliations.contains(&affiliation) {
                affiliations.push(affiliation);
            }
        }
    }

    let affiliation = if affiliations.is_empty() {
        None
    } else {
        Some(affiliations.join("; "))
    };

    (authors, affiliation)
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

fn clean_abstract_text(input: &str) -> String {
    let labels = [
        "Baggrund",
        "Formål",
        "Metode og materiale",
        "Resultater",
        "Diskussion",
        "Konklusion",
        "Background",
        "Objective",
        "Aim",
        "Purpose",
        "Methods and materials",
        "Materials and methods",
        "Results",
        "Discussion",
        "Conclusion",
    ];
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut out = String::with_capacity(input.len());
    let mut idx = 0;

    while idx < chars.len() {
        let mut matched = None;
        let mut prev_idx = idx;
        let mut prev_non_ws = None;
        while prev_idx > 0 {
            prev_idx -= 1;
            let ch = chars[prev_idx].1;
            if !ch.is_whitespace() {
                prev_non_ws = Some(ch);
                break;
            }
        }
        let prev_is_boundary = prev_non_ws
            .map(|ch| ch == '/' || ch == ':' || ch == '.' || ch == ',' || ch == ';')
            .unwrap_or(true);
        for label in labels.iter() {
            if let Some(after_label) = match_label_at(&chars, idx, label) {
                let after_space = skip_whitespace(&chars, after_label);
                let has_space = after_space > after_label;
                if after_space < chars.len() {
                    let delim = chars[after_space].1;
                    if delim == '/' || delim == ':' || delim == '.' || delim == ',' || delim == ';'
                    {
                        let after_delim = skip_whitespace(&chars, after_space + 1);
                        matched = Some(after_delim);
                        break;
                    }
                    if has_space && (delim.is_uppercase() || delim.is_ascii_digit()) {
                        matched = Some(after_space);
                        break;
                    }
                    if has_space && prev_is_boundary {
                        matched = Some(after_space);
                        break;
                    }
                }
            }
        }
        if let Some(next_idx) = matched {
            idx = next_idx;
            continue;
        }
        out.push(chars[idx].1);
        idx += 1;
    }
    out
}

// Extract parsing of abstracts from a rows buffer into a helper so tests can exercise
// duplicate-id handling and header-detection without needing actual workbook files.
#[allow(dead_code)]
pub fn parse_abstracts_from_rows(
    rows_a: &[Vec<String>],
    header_idx: usize,
) -> Result<HashMap<String, Abstract>> {
    let header_row = &rows_a[header_idx];
    let lower_row: Vec<String> = header_row.iter().map(|s| s.to_lowercase()).collect();
    let find_col = |subs: &[&str]| -> Option<usize> {
        for (j, cell) in lower_row.iter().enumerate() {
            for &s in subs {
                if cell.contains(&s.to_lowercase()) {
                    return Some(j);
                }
            }
        }
        None
    };

    let col_id = find_col(&["id"]).ok_or_else(|| anyhow!("id column not found in abstracts"))?;
    let col_title = find_col(&["title", "titel"]).unwrap_or(col_id + 1);
    let col_authors = find_col(&["authors", "author", "forfatter"]).unwrap_or(col_title + 1);
    let col_abstract = find_col(&["abstract", "resum", "resumé"]).unwrap_or(col_title + 2);
    let col_keywords = find_col(&["keyword", "keywords", "nøgle", "emne ord", "emneord"])
        .unwrap_or(col_abstract + 1);
    let col_takehome = find_col(&["take home", "take-home", "takehome", "take home messages"])
        .unwrap_or(col_keywords + 1);
    let col_reference = find_col(&["reference", "published", "doi"]).unwrap_or(col_takehome + 1);
    let col_literature = find_col(&["litterature", "literature", "references", "literatur"])
        .unwrap_or(col_reference + 1);
    let col_center = find_col(&["center", "centre", "center/centre"]).unwrap_or(col_authors + 1);
    let col_contact = find_col(&["email", "kontakt", "contact"]).unwrap_or(col_authors + 2);

    let mut abstracts: Vec<Abstract> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (ridx, row) in rows_a.iter().enumerate().skip(header_idx + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let aid = row
            .get(col_id)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let title = row
            .get(col_title)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let authors_raw = row
            .get(col_authors)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = row
            .get(col_abstract)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = clean_abstract_text(&abstract_text);
        let keywords = row
            .get(col_keywords)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let take_home = row
            .get(col_takehome)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let reference = row
            .get(col_reference)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let literature = row
            .get(col_literature)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let center = row
            .get(col_center)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let contact = row
            .get(col_contact)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if aid.is_empty() && title.is_empty() && abstract_text.is_empty() {
            continue;
        }

        if !aid.is_empty() {
            if seen.contains_key(&aid) {
                return Err(anyhow!(
                    "Duplicate abstract id found: {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            seen.insert(aid.clone(), ridx + 1);
        }

        let (authors_vec, affiliation) = parse_authors_and_affiliation(&authors_raw);
        let keywords_vec = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let locale_val = detect_locale(header_row, row);

        abstracts.push(Abstract {
            id: aid.clone(),
            title: title.clone(),
            authors: authors_vec,
            affiliation,
            center,
            contact_email: contact,
            abstract_text: abstract_text.clone(),
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
        let mut xls = Vec::new();
        for entry in fs::read_dir(path)? {
            let e = entry?;
            let p = e.path();
            if let Some(ext) = p.extension() {
                if ext == "xlsx" {
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                        if fname.starts_with("~$") {
                            continue;
                        }
                    }
                    xls.push(p.to_string_lossy().to_string());
                }
            }
        }
        if xls.is_empty() {
            return Err(anyhow!("No .xlsx files found in directory {}", path));
        }
        // prefer with_ids.xlsx as abstracts file
        let mut file_a = None::<String>;
        let mut file_b = None::<String>;
        for f in &xls {
            if f.to_lowercase().contains("with_ids") || f.to_lowercase().contains("afsluttede") {
                file_a = Some(f.clone());
            }
            if f.to_lowercase().contains("kopi")
                || f.to_lowercase().contains("grupper")
                || f.to_lowercase().contains("final")
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
        .ok_or_else(|| anyhow!("Failed to get range for sheet {}", abstracts_sheet))??;
    let mut rows_a: Vec<Vec<String>> = Vec::new();
    for r in range.rows() {
        rows_a.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    // detect header row
    let header_idx = find_header_row(&rows_a, &[])
        .ok_or_else(|| anyhow!("Could not detect header row in abstracts sheet"))?;
    let header_row = &rows_a[header_idx];
    let lower_row: Vec<String> = header_row.iter().map(|s| s.to_lowercase()).collect();
    let find_col = |subs: &[&str]| -> Option<usize> {
        for (j, cell) in lower_row.iter().enumerate() {
            for &s in subs {
                if cell.contains(&s.to_lowercase()) {
                    return Some(j);
                }
            }
        }
        None
    };

    let col_id = find_col(&["id"]).ok_or_else(|| anyhow!("id column not found in abstracts"))?;
    let col_title = find_col(&["title", "titel"]).unwrap_or(col_id + 1);
    let col_authors = find_col(&["authors", "author", "forfatter"]).unwrap_or(col_title + 1);
    let col_abstract = find_col(&["abstract", "resum", "resumé"]).unwrap_or(col_title + 2);
    let col_keywords = find_col(&["keyword", "keywords", "nøgle", "emne ord", "emneord"])
        .unwrap_or(col_abstract + 1);
    let col_takehome = find_col(&["take home", "take-home", "takehome", "take home messages"])
        .unwrap_or(col_keywords + 1);
    let col_reference = find_col(&["reference", "published", "doi"]).unwrap_or(col_takehome + 1);
    let col_literature = find_col(&["litterature", "literature", "references", "literatur"])
        .unwrap_or(col_reference + 1);
    let col_center = find_col(&["center", "centre", "center/centre"]).unwrap_or(col_authors + 1);
    let col_contact = find_col(&["email", "kontakt", "contact"]).unwrap_or(col_authors + 2);

    let mut abstracts: Vec<Abstract> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (ridx, row) in rows_a.iter().enumerate().skip(header_idx + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let aid = row
            .get(col_id)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let title = row
            .get(col_title)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let authors_raw = row
            .get(col_authors)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = row
            .get(col_abstract)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = clean_abstract_text(&abstract_text);
        let keywords = row
            .get(col_keywords)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let take_home = row
            .get(col_takehome)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let reference = row
            .get(col_reference)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let literature = row
            .get(col_literature)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let center = row
            .get(col_center)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let contact = row
            .get(col_contact)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if aid.is_empty() && title.is_empty() && abstract_text.is_empty() {
            continue;
        }

        if !aid.is_empty() {
            if seen.contains_key(&aid) {
                return Err(anyhow!(
                    "Duplicate abstract id found: {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            seen.insert(aid.clone(), ridx + 1);
        }

        let (authors_vec, affiliation) = parse_authors_and_affiliation(&authors_raw);
        let keywords_vec = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let locale_val = detect_locale(header_row, row);

        abstracts.push(Abstract {
            id: aid.clone(),
            title: title.clone(),
            authors: authors_vec,
            affiliation,
            center,
            contact_email: contact,
            abstract_text: abstract_text.clone(),
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

    // parse sessions sheet using flexible heuristics (header rows vs item rows)
    tracing::info!("Parsing sessions sheet: {}", sessions_sheet);
    let range_b = wb
        .worksheet_range(&sessions_sheet)
        .ok_or_else(|| anyhow!("Failed to get range for sheet {}", sessions_sheet))??;
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
        .ok_or_else(|| anyhow!("Failed to read sheet {} from {}", sheet_a, file_a))??;
    let mut rows_a: Vec<Vec<String>> = Vec::new();
    for r in range_a.rows() {
        rows_a.push(r.iter().map(|c| as_str(Some(c))).collect());
    }

    let header_idx = find_header_row(&rows_a, &[])
        .ok_or_else(|| anyhow!("Could not detect header row in abstracts sheet"))?;
    let header_row = &rows_a[header_idx];
    let lower_row: Vec<String> = header_row.iter().map(|s| s.to_lowercase()).collect();
    let find_col = |subs: &[&str]| -> Option<usize> {
        for (j, cell) in lower_row.iter().enumerate() {
            for &s in subs {
                if cell.contains(&s.to_lowercase()) {
                    return Some(j);
                }
            }
        }
        None
    };

    let col_id = find_col(&["id"]).ok_or_else(|| anyhow!("id column not found in abstracts"))?;
    let col_title = find_col(&["title", "titel"]).unwrap_or(col_id + 1);
    let col_authors = find_col(&["authors", "author", "forfatter"]).unwrap_or(col_title + 1);
    let col_abstract = find_col(&["abstract", "resum", "resumé"]).unwrap_or(col_title + 2);
    let col_keywords = find_col(&["keyword", "keywords", "nøgle", "emne ord", "emneord"])
        .unwrap_or(col_abstract + 1);
    let col_takehome = find_col(&["take home", "take-home", "takehome", "take home messages"])
        .unwrap_or(col_keywords + 1);
    let col_reference = find_col(&["reference", "published", "doi"]).unwrap_or(col_takehome + 1);
    let col_literature = find_col(&["litterature", "literature", "references", "literatur"])
        .unwrap_or(col_reference + 1);
    let col_center = find_col(&["center", "centre", "center/centre"]).unwrap_or(col_authors + 1);
    let col_contact = find_col(&["email", "kontakt", "contact"]).unwrap_or(col_authors + 2);

    let mut abstracts: Vec<Abstract> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (ridx, row) in rows_a.iter().enumerate().skip(header_idx + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let aid = row
            .get(col_id)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let title = row
            .get(col_title)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let authors_raw = row
            .get(col_authors)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = row
            .get(col_abstract)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let abstract_text = clean_abstract_text(&abstract_text);
        let keywords = row
            .get(col_keywords)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let take_home = row
            .get(col_takehome)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let reference = row
            .get(col_reference)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let literature = row
            .get(col_literature)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let center = row
            .get(col_center)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let contact = row
            .get(col_contact)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if aid.is_empty() && title.is_empty() && abstract_text.is_empty() {
            continue;
        }

        if !aid.is_empty() {
            if seen.contains_key(&aid) {
                return Err(anyhow!(
                    "Duplicate abstract id found: {} at row {}",
                    aid,
                    ridx + 1
                ));
            }
            seen.insert(aid.clone(), ridx + 1);
        }

        let (authors_vec, affiliation) = parse_authors_and_affiliation(&authors_raw);
        let keywords_vec = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let locale_val = detect_locale(header_row, row);

        abstracts.push(Abstract {
            id: aid.clone(),
            title: title.clone(),
            authors: authors_vec,
            affiliation,
            center,
            contact_email: contact,
            abstract_text: abstract_text.clone(),
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
        .ok_or_else(|| anyhow!("Failed to read sheet {} from {}", sheet_b, file_b))??;
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
