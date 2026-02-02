use crate::config::{default_config_path, load_symposium_config, resolve_cwd_path};
use crate::model::{Abstract, Session};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub fn validate_inputs(abstracts: Option<String>) -> Result<()> {
    let config_path = default_config_path()?;
    let cfg = load_symposium_config(&config_path)?;

    let abstracts = abstracts
        .or_else(|| cfg.as_ref().and_then(|c| c.abstracts.clone()))
        .ok_or_else(|| {
            anyhow!(
                "Missing abstracts path. Pass --abstracts or set [symposium].abstracts in {}",
                config_path.to_string_lossy()
            )
        })?;

    let abstracts_path = resolve_cwd_path(&abstracts)?;

    let (abstracts, sessions) = crate::io::excel::parse_workbook(&abstracts_path)?;
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
