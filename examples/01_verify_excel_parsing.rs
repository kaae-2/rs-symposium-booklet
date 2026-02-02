use serde_json::json;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

use symposium_booklet::{config, parse_two_workbooks};

fn main() -> anyhow::Result<()> {
    let config_path = config::default_config_path()?;
    let cfg = config::load_symposium_config(&config_path)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Missing [symposium] config in {}",
            config_path.to_string_lossy()
        )
    })?;
    let abstracts_raw = cfg.abstracts.ok_or_else(|| {
        anyhow::anyhow!(
            "Missing [symposium].abstracts in {}",
            config_path.to_string_lossy()
        )
    })?;
    let ordering_raw = cfg.ordering.ok_or_else(|| {
        anyhow::anyhow!(
            "Missing [symposium].ordering in {}",
            config_path.to_string_lossy()
        )
    })?;
    let abstracts_path = config::resolve_cwd_path(&abstracts_raw)?;
    let grouping_path = config::resolve_cwd_path(&ordering_raw)?;

    // parse the two explicit workbooks
    let (abstracts_map, sessions) = match parse_two_workbooks(&abstracts_path, &grouping_path) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error parsing workbooks: {}", e);
            std::process::exit(1);
        }
    };

    // convert abstracts map to vec
    let mut abstracts: Vec<_> = abstracts_map.into_iter().map(|(_k, v)| v).collect();
    // sort by id for deterministic output
    abstracts.sort_by(|a, b| a.id.cmp(&b.id));

    // build JSON object
    let manifest = json!({
        "sheet_a": {"path": abstracts_path},
        "sheet_b": {"path": grouping_path},
        "summary": {"num_abstracts_parsed": abstracts.len(), "num_sessions": sessions.len()},
        "abstracts": abstracts,
        "sessions": sessions,
    });

    // write to data/tools_output/parse_example_output.json
    let out_dir = Path::new("data").join("tools_output");
    if let Err(e) = create_dir_all(&out_dir) {
        eprintln!("Failed to create output dir {}: {}", out_dir.display(), e);
        std::process::exit(1);
    }
    let out_file = out_dir.join("parse_example_output.json");
    match File::create(&out_file) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes())
            {
                eprintln!("Failed to write {}: {}", out_file.display(), e);
                std::process::exit(1);
            }
            println!("Wrote JSON to {}", out_file.display());
        }
        Err(e) => {
            eprintln!("Failed to create {}: {}", out_file.display(), e);
            std::process::exit(1);
        }
    }
    Ok(())
}
