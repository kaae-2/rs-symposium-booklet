use crate::model::{Abstract, ItemRef, Manifest, Session};
use anyhow::{anyhow, Result};
use slug::slugify;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

pub fn write_markdown(
    abstracts: &HashMap<String, Abstract>,
    sessions: &Vec<Session>,
    outdir: &str,
) -> Result<()> {
    // ensure output exists
    create_dir_all(outdir)?;

    let mut manifest_sessions = Vec::new();

    for session in sessions.iter() {
        // build a safe slug for the session directory
        let mut slug = slugify(&session.title);
        if slug.trim().is_empty() {
            slug = format!("session-{}", session.order);
        }
        // guard characters (extra safety)
        let slug_safe: String = slug
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let session_slug = if slug_safe.is_empty() {
            format!("session-{}", session.order)
        } else {
            slug_safe
        };

        let session_dir = Path::new(outdir).join(&session_slug);
        create_dir_all(&session_dir)?;

        // sort items by order
        let mut items = session.items.clone();
        items.sort_by_key(|i| i.order);

        let mut used_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (idx, item) in items.iter().enumerate() {
            let abs = abstracts
                .get(&item.id)
                .ok_or_else(|| anyhow!("Referenced abstract {} not found", item.id))?;
            let mut title_slug = slugify(&abs.title);
            if title_slug.trim().is_empty() {
                title_slug = abs.id.clone();
            }
            let title_safe: String = title_slug
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            let filename_base = if title_safe.is_empty() {
                format!("{:04}", idx + 1)
            } else {
                format!("{:04}-{}", idx + 1, title_safe)
            };

            // ensure filename uniqueness within session and vs existing files by
            // appending `-1`, `-2`, ... when collisions are detected
            let mut candidate = filename_base.clone();
            let mut suffix: u32 = 0;
            loop {
                let candidate_path = session_dir.join(format!("{}.md", candidate));
                if !used_names.contains(&candidate) && !candidate_path.exists() {
                    break;
                }
                suffix += 1;
                candidate = format!("{}-{}", filename_base, suffix);
            }
            used_names.insert(candidate.clone());

            let path = session_dir.join(format!("{}.md", candidate));

            let mut f = File::create(&path)
                .map_err(|e| anyhow!("Failed to create file {}: {}", path.display(), e))?;
            // write yaml frontmatter
            writeln!(f, "---")?;
            writeln!(f, "id: \"{}\"", abs.id)?;
            writeln!(f, "title: \"{}\"", abs.title)?;
            writeln!(f, "authors:")?;
            for a in abs.authors.iter() {
                writeln!(f, "  - \"{}\"", a)?;
            }
            if let Some(aff) = &abs.affiliation {
                writeln!(f, "affiliation: \"{}\"", aff)?;
            }
            writeln!(f, "session: \"{}\"", session.title)?;
            writeln!(f, "order: {}", item.order)?;
            writeln!(f, "locale: \"{}\"", abs.locale)?;
            if !abs.keywords.is_empty() {
                writeln!(f, "keywords:")?;
                for k in abs.keywords.iter() {
                    writeln!(f, "  - \"{}\"", k)?;
                }
            }
            writeln!(f, "---\n")?;

            // write body
            writeln!(f, "{}", abs.abstract_text)?;
        }

        manifest_sessions.push(serde_json::json!({
            "id": session.id,
            "title": session.title,
            "slug": slug,
            "order": session.order,
            "count": session.items.len()
        }));
    }

    // write manifest file
    let manifest = serde_json::json!({
        "event": "symposium-2026",
        "sessions": manifest_sessions
    });
    let mf = Path::new(outdir).join("manifest.json");
    let mut f = File::create(mf)?;
    write!(f, "{}", serde_json::to_string_pretty(&manifest)?)?;

    Ok(())
}
