use std::fs;

use symposium_booklet::typst::emit_typst;

#[test]
fn emit_typst_keeps_entry_when_bibliography_url_contains_triple_dash() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let outdir = tmp.path().join("out");
    let session_slug = "teknologi-postere";
    let session_dir = outdir.join(session_slug);
    fs::create_dir_all(&session_dir).expect("create session dir");

    let manifest = r#"{
  "event": "symposium-2026",
  "sessions": [
    {
      "count": 1,
      "id": "Teknologi - Postere",
      "order": 1,
      "slug": "teknologi-postere",
      "tema": "Teknologi",
      "title": "Teknologi - Postere",
      "type": "Postere"
    }
  ]
}
"#;
    fs::write(outdir.join("manifest.json"), manifest).expect("write manifest");

    let md = r#"---
id: "8e9881a9"
title: "Personlig hygiejne - på den rigtige måde"
tema: "Teknologi"
type: "Postere"
order: 7
locale: "da"
bibliography:
  - "https://www.sst.dk/da/udgivelser/2021/Hygiejne-i-aeldreplejen---Kommunale-erfaringer"
---

Baggrund og indhold.
"#;
    fs::write(
        session_dir.join("0007-personlig-hygiejne-pa-den-rigtige-made.md"),
        md,
    )
    .expect("write markdown");

    emit_typst(outdir.to_string_lossy().as_ref(), "da", &None).expect("emit typst");

    let typst = fs::read_to_string(outdir.join("typst").join("book_da.typ")).expect("read typst");
    assert!(
        typst.contains("Personlig hygiejne - på den rigtige måde"),
        "expected generated typst to include abstract title"
    );
    assert!(
        typst.contains("<abs-8e9881a9>"),
        "expected generated typst to include abstract label"
    );
}
