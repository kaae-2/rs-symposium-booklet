use assert_cmd::cargo::cargo_bin_cmd;
use std::collections::HashMap;
use std::fs;
use symposium_booklet::io::excel::parse_abstracts_from_rows;
use symposium_booklet::io::markdown::write_markdown_plan;
use symposium_booklet::io::plan::PlanAction;
use symposium_booklet::model::{Abstract, ItemRef, Session};
mod common;
use common::fixtures::make_fixture;

#[test]
fn parse_rows_detects_locale_column() {
    let rows = vec![
        vec![
            "id".to_string(),
            "title".to_string(),
            "locale".to_string(),
            "abstract".to_string(),
        ],
        vec![
            "a1".to_string(),
            "Title 1".to_string(),
            "en".to_string(),
            "Text 1".to_string(),
        ],
    ];
    let header_idx = 0usize;
    let map = parse_abstracts_from_rows(&rows, header_idx).expect("parse should succeed");
    let a = map.get("a1").expect("abstract a1 present");
    assert_eq!(a.locale, "en");
}

#[test]
fn write_markdown_plan_includes_locale_and_paths() {
    let mut abstracts: HashMap<String, Abstract> = HashMap::new();
    abstracts.insert(
        "a1".to_string(),
        Abstract {
            id: "a1".to_string(),
            title: "My Title".to_string(),
            authors: vec!["A".to_string()],
            affiliation: None,
            center: None,
            contact_email: None,
            abstract_text: "T".to_string(),
            keywords: vec![],
            take_home: None,
            reference: None,
            literature: None,
            locale: "en".to_string(),
        },
    );
    let session = Session {
        id: "s1".to_string(),
        title: "Session 1".to_string(),
        order: 1,
        items: vec![ItemRef {
            id: "a1".to_string(),
            order: 1,
        }],
    };
    let mut plan = symposium_booklet::io::plan::Plan::default();
    write_markdown_plan(&abstracts, &vec![session], "outdir", &mut plan).unwrap();

    // find a WriteFile action with summary containing locale
    let mut found = false;
    for a in plan.actions.iter() {
        if let PlanAction::WriteFile { path: _p, summary } = a {
            if summary.contains("locale:en") {
                found = true;
            }
        }
    }
    assert!(
        found,
        "expected a WriteFile plan entry mentioning locale:en"
    );
}

#[test]
fn integration_fixture_runs_binary_dry_run() {
    // generate a small fixture workbook
    let fixture_dir = "target/test-fixtures";
    let _ = fs::create_dir_all(fixture_dir);
    let xlsx_path = format!("{}/fixture.xlsx", fixture_dir);
    make_fixture(&xlsx_path).expect("create fixture");

    let out = "target/test-dry-run-int";
    let _ = fs::remove_dir_all(out);

    let mut cmd = cargo_bin_cmd!("symposium-booklet");
    cmd.args(["build", "--input", &xlsx_path, "--output", out, "--dry-run"]);
    let assert = cmd.assert().success();
    let outstr = String::from_utf8(assert.get_output().stdout.clone()).unwrap_or_default();
    assert!(outstr.contains("DRY-RUN PLAN"));
    assert!(outstr.contains("PLAN JSON"));

    let _ = fs::remove_dir_all(fixture_dir);
    let _ = fs::remove_dir_all(out);
}
