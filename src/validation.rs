use crate::model::{Abstract, Session};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub fn validate_input(_input: &str) -> Result<()> {
    // placeholder: would open and validate workbook
    anyhow::bail!("Not implemented: validate_input");
}

pub fn validate_refs(abstracts: &HashMap<String, Abstract>, sessions: &Vec<Session>) -> Result<()> {
    // ensure every referenced id exists
    for s in sessions.iter() {
        for item in s.items.iter() {
            if !abstracts.contains_key(&item.id) {
                return Err(anyhow!("Session {} references missing abstract id {}", s.title, item.id));
            }
        }
    }
    Ok(())
}
