use crate::model::{Abstract, Session};
use anyhow::{anyhow, Result};
use slug::slugify;
use std::collections::HashMap;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::Path;

const MAX_SESSION_SLUG_LEN: usize = 60;
const MAX_TITLE_SLUG_LEN: usize = 80;

fn truncate_slug(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }
    let trimmed = input[..max_len].trim_end_matches('-');
    trimmed.to_string()
}

pub fn write_markdown(
    abstracts: &HashMap<String, Abstract>,
    sessions: &[Session],
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
            truncate_slug(&slug_safe, MAX_SESSION_SLUG_LEN)
        };

        let session_dir = Path::new(outdir).join(&session_slug);
        if session_dir.exists() {
            remove_dir_all(&session_dir)?;
        }
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
            let title_slug = truncate_slug(&title_safe, MAX_TITLE_SLUG_LEN);
            let filename_base = if title_slug.is_empty() {
                format!("{:04}", idx + 1)
            } else {
                format!("{:04}-{}", idx + 1, title_slug)
            };

            // ensure filename uniqueness within session and vs existing files by
            // appending `-1`, `-2`, ... when collisions are detected
            let mut candidate = filename_base.clone();
            let mut suffix: u32 = 0;
            loop {
                if !used_names.contains(&candidate) {
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
            if let Some(take_home) = &abs.take_home {
                writeln!(f, "take_home: \"{}\"", take_home)?;
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

// produce a Plan of filesystem actions without performing writes
pub fn write_markdown_plan(
    abstracts: &HashMap<String, Abstract>,
    sessions: &[Session],
    outdir: &str,
    plan: &mut crate::io::plan::Plan,
) -> Result<()> {
    use crate::io::plan::PlanAction;
    use std::path::PathBuf;

    // ensure output dir would exist
    plan.push(PlanAction::CreateDir {
        path: PathBuf::from(outdir),
    });

    for session in sessions.iter() {
        let mut slug = slugify(&session.title);
        if slug.trim().is_empty() {
            slug = format!("session-{}", session.order);
        }
        let slug_safe: String = slug
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let session_slug = if slug_safe.is_empty() {
            format!("session-{}", session.order)
        } else {
            truncate_slug(&slug_safe, MAX_SESSION_SLUG_LEN)
        };

        let session_dir = Path::new(outdir).join(&session_slug);
        plan.push(PlanAction::DeleteDir {
            path: session_dir.clone(),
        });
        plan.push(PlanAction::CreateDir {
            path: session_dir.clone(),
        });

        let mut items = session.items.clone();
        items.sort_by_key(|i| i.order);

        let mut used_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (idx, item) in items.iter().enumerate() {
            let abs = abstracts
                .get(&item.id)
                .ok_or_else(|| anyhow!(format!("Referenced abstract {} not found", item.id)))?;
            let mut title_slug = slugify(&abs.title);
            if title_slug.trim().is_empty() {
                title_slug = abs.id.clone();
            }
            let title_safe: String = title_slug
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            let title_slug = truncate_slug(&title_safe, MAX_TITLE_SLUG_LEN);
            let filename_base = if title_slug.is_empty() {
                format!("{:04}", idx + 1)
            } else {
                format!("{:04}-{}", idx + 1, title_slug)
            };

            let mut candidate = filename_base.clone();
            let mut suffix: u32 = 0;
            loop {
                if !used_names.contains(&candidate) {
                    break;
                }
                suffix += 1;
                candidate = format!("{}-{}", filename_base, suffix);
            }
            used_names.insert(candidate.clone());

            let path = session_dir.join(format!("{}.md", candidate));
            // produce a short summary for plan
            let summary = format!("{} — locale:{}", abs.title, abs.locale);
            plan.push(PlanAction::WriteFile { path, summary });
        }

        // manifest session entry
        plan.push(PlanAction::UpdateManifest {
            path: PathBuf::from(outdir).join("manifest.json"),
            manifest_summary: format!("session {} => {} items", session.title, session.items.len()),
        });
    }

    Ok(())
}
