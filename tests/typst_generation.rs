use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;

mod common;
use common::fixtures::make_fixture;

#[test]
fn typst_output_compiles_when_typst_available() {
    let check = StdCommand::new("typst").arg("--version").status();
    if check.is_err() || !check.unwrap().success() {
        return;
    }

    let fixture_dir = "target/test-fixtures";
    let _ = fs::create_dir_all(fixture_dir);
    let xlsx_path = format!("{}/fixture.xlsx", fixture_dir);
    make_fixture(&xlsx_path).expect("create fixture");

    let out = "target/test-typst";
    let _ = fs::remove_dir_all(out);

    let mut cmd = Command::cargo_bin("symposium-booklet").unwrap();
    cmd.args([
        "build",
        "--input",
        &xlsx_path,
        "--output",
        out,
        "--locales",
        "en",
    ]);
    cmd.assert().success();

    let typst_file = format!("{}/typst/book_en.typ", out);
    let pdf_path = format!("{}/book_en_test.pdf", out);
    let status = StdCommand::new("typst")
        .arg("compile")
        .arg(&typst_file)
        .arg(&pdf_path)
        .status()
        .expect("run typst compile");
    assert!(status.success(), "typst compile failed");

    let _ = fs::remove_dir_all(fixture_dir);
    let _ = fs::remove_dir_all(out);
}
