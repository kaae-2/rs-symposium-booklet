use crate::model::{Abstract, ItemRef, Manifest, Session};
use anyhow::{anyhow, Result};
use calamine::{open_workbook_auto, DataType, RangeDeserializerBuilder, Reader};
use std::collections::HashMap;

pub fn parse_workbook(path: &str) -> Result<(HashMap<String, Abstract>, Vec<Session>)> {
    let mut wb = open_workbook_auto(path).map_err(|e| anyhow!("Failed to open workbook: {}", e))?;

    // heuristics: look for a sheet named "abstracts" and one named "sessions" (case-insensitive)
    let mut abstracts_map = HashMap::new();
    let mut sessions: Vec<Session> = Vec::new();

    // parse abstracts sheet
    let sheet_names = wb.sheet_names().to_owned();
    let mut abstracts_sheet = None;
    let mut sessions_sheet = None;
    for name in sheet_names {
        let low = name.to_lowercase();
        if low.contains("abstract") {
            abstracts_sheet = Some(name.clone());
        }
        if low.contains("session") || low.contains("include") || low.contains("inclusion") {
            sessions_sheet = Some(name.clone());
        }
    }

    let abstracts_sheet = abstracts_sheet.ok_or_else(|| anyhow!("No abstracts sheet found"))?;
    let sessions_sheet = sessions_sheet.ok_or_else(|| anyhow!("No sessions/include sheet found"))?;

    tracing::info!("Parsing abstracts sheet: {}", abstracts_sheet);
    if let Some(Ok(range)) = wb.worksheet_range(&abstracts_sheet) {
        let mut rows = range.rows();
        let headers: Vec<String> = rows
            .next()
            .ok_or_else(|| anyhow!("Abstracts sheet is empty"))?
            .iter()
            .map(|c| c.to_string())
            .collect();

        // expected headers: id,title,authors,affiliation,abstract,keywords,locale
        for (ridx, row) in rows.enumerate() {
            // map columns by header index
            let mut map = HashMap::new();
            for (cidx, cell) in row.iter().enumerate() {
                map.insert(headers[cidx].to_lowercase(), cell.clone());
            }

            let id = cell_to_string(map.get("id")).ok_or_else(|| anyhow!("Missing id at row {}", ridx + 2))?;
            let title = cell_to_string(map.get("title")).ok_or_else(|| anyhow!("Missing title at row {}", ridx + 2))?;
            let authors = cell_to_string(map.get("authors")).unwrap_or_default();
            let affiliation = cell_to_string(map.get("affiliation"));
            let abstract_text = cell_to_string(map.get("abstract")).ok_or_else(|| anyhow!("Missing abstract at row {}", ridx + 2))?;
            let keywords = cell_to_string(map.get("keywords")).unwrap_or_default();
            let locale = cell_to_string(map.get("locale")).unwrap_or_else(|| "da".to_string());

            let abstract = Abstract {
                id: id.clone(),
                title: title.clone(),
                authors: authors.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                affiliation: affiliation,
                abstract_text,
                keywords: keywords.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                locale,
            };
            if abstracts_map.insert(id.clone(), abstract).is_some() {
                return Err(anyhow!("Duplicate abstract id {}", id));
            }
        }
    }

    tracing::info!("Parsing sessions sheet: {}", sessions_sheet);
    if let Some(Ok(range)) = wb.worksheet_range(&sessions_sheet) {
        let mut rows = range.rows();
        let headers: Vec<String> = rows
            .next()
            .ok_or_else(|| anyhow!("Sessions sheet is empty"))?
            .iter()
            .map(|c| c.to_string())
            .collect();

        // detect shape: if headers contain abstract_id -> shape 2, else shape 1
        let headers_l = headers.iter().map(|h| h.to_lowercase()).collect::<Vec<_>>();
        let shape2 = headers_l.contains(&"abstract_id".to_string()) || headers_l.contains(&"abstract".to_string());

        if shape2 {
            for (ridx, row) in rows.enumerate() {
                let mut map = HashMap::new();
                for (cidx, cell) in row.iter().enumerate() {
                    map.insert(headers[cidx].to_lowercase(), cell.clone());
                }
                let aid = cell_to_string(map.get("abstract_id")).or_else(|| cell_to_string(map.get("abstract"))).ok_or_else(|| anyhow!("Missing abstract_id at row {}", ridx + 2))?;
                let sid = cell_to_string(map.get("session_id")).unwrap_or_else(|| "default".to_string());
                let stitle = cell_to_string(map.get("session_title")).unwrap_or_else(|| "Session".to_string());
                let order = cell_to_u32(map.get("session_order")).unwrap_or(0);
                let item_order = cell_to_u32(map.get("item_order")).unwrap_or(0);

                // find or create session
                let mut session = sessions.iter_mut().find(|s| s.id == sid).cloned();
                if session.is_none() {
                    sessions.push(Session { id: sid.clone(), title: stitle.clone(), order, items: Vec::new() });
                }
                // push itemref
                if let Some(s) = sessions.iter_mut().find(|s| s.id == sid) {
                    s.items.push(ItemRef { id: aid.clone(), order: item_order });
                }
            }
        } else {
            // shape1: one row per session with abstract_ids list
            for (ridx, row) in rows.enumerate() {
                let mut map = HashMap::new();
                for (cidx, cell) in row.iter().enumerate() {
                    map.insert(headers[cidx].to_lowercase(), cell.clone());
                }
                let sid = cell_to_string(map.get("session_id")).or_else(|| cell_to_string(map.get("id"))).unwrap_or_else(|| format!("s_{}", ridx + 1));
                let stitle = cell_to_string(map.get("session_title")).or_else(|| cell_to_string(map.get("title"))).unwrap_or_else(|| format!("Session {}", ridx + 1));
                let order = cell_to_u32(map.get("session_order")).unwrap_or(ridx as u32 + 1);
                let aids = cell_to_string(map.get("abstract_ids")).unwrap_or_default();
                let mut items = Vec::new();
                for (i, aid) in aids.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).enumerate() {
                    items.push(ItemRef { id: aid.to_string(), order: i as u32 + 1 });
                }
                sessions.push(Session { id: sid, title: stitle, order, items });
            }
        }
    }

    Ok((abstracts_map, sessions))
}

fn cell_to_string(c: Option<&DataType>) -> Option<String> {
    c.map(|cell| match cell {
        DataType::Empty => "".to_string(),
        DataType::String(s) => s.clone(),
        DataType::Float(f) => f.to_string(),
        DataType::Int(i) => i.to_string(),
        DataType::Bool(b) => b.to_string(),
        _ => format!("{}", cell),
    })
}

fn cell_to_u32(c: Option<&DataType>) -> Option<u32> {
    c.and_then(|cell| match cell {
        DataType::Int(i) => Some(*i as u32),
        DataType::Float(f) => Some(*f as u32),
        DataType::String(s) => s.parse::<u32>().ok(),
        _ => None,
    })
}
