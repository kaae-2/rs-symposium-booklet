use crate::model::{Abstract, Session};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub fn validate_input(input: &str) -> Result<()> {
    // parse workbook (this now performs strict header checks and duplicate-id errors)
    let (abstracts, sessions) = crate::io::parse_workbook(input)?;
    // ensure every referenced id exists
    validate_refs(&abstracts, &sessions)?;
    Ok(())
}

pub fn validate_refs(abstracts: &HashMap<String, Abstract>, sessions: &[Session]) -> Result<()> {
    // ensure every referenced id exists
    for s in sessions.iter() {
        for item in s.items.iter() {
            if !abstracts.contains_key(&item.id) {
                return Err(anyhow!(
                    "Session {} references missing abstract id {}",
                    s.title,
                    item.id
                ));
            }
        }
    }
    Ok(())
}
