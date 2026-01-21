use serde_json::json;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

use symposium_booklet::parse_two_workbooks;

fn main() {
    // Require explicit file paths via cargo env vars; fail early if missing
    let abstracts_path = option_env!("SYMPOSIUM_ABSTRACTS")
        .expect("SYMPOSIUM_ABSTRACTS must be set in .cargo/config.toml");
    let grouping_path = option_env!("SYMPOSIUM_GROUPING")
        .expect("SYMPOSIUM_GROUPING must be set in .cargo/config.toml");

    // parse the two explicit workbooks
    let (abstracts_map, sessions) = match parse_two_workbooks(abstracts_path, grouping_path) {
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
}
