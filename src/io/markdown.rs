use crate::model::{Abstract, ItemRef, Manifest, Session};
use anyhow::{anyhow, Result};
use slug::slugify;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

pub fn write_markdown(abstracts: &HashMap<String, Abstract>, sessions: &Vec<Session>, outdir: &str) -> Result<()> {
    // ensure output exists
    create_dir_all(outdir)?;

    let mut manifest_sessions = Vec::new();

    for session in sessions.iter() {
        let slug = slugify(&session.title);
        let session_dir = Path::new(outdir).join(&slug);
        create_dir_all(&session_dir)?;

        // sort items by order
        let mut items = session.items.clone();
        items.sort_by_key(|i| i.order);

        for (idx, item) in items.iter().enumerate() {
            let abs = abstracts.get(&item.id).ok_or_else(|| anyhow!("Referenced abstract {} not found", item.id))?;
            let title_slug = slugify(&abs.title);
            let filename = format!("{:04}-{}.md", idx + 1, title_slug);
            let path = session_dir.join(&filename);

            let mut f = File::create(&path)?;
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
