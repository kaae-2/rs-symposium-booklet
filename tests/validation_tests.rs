use symposium_booklet::io::excel::{find_header_row, parse_abstracts_from_rows};

#[test]
fn header_detection_finds_header() {
    let rows = vec![
        vec!["Some meta".to_string(), "".to_string()],
        vec![
            "ID".to_string(),
            "Title".to_string(),
            "Abstract".to_string(),
        ],
        vec!["a1".to_string(), "My title".to_string(), "text".to_string()],
    ];
    let idx = find_header_row(&rows, &[]).expect("header should be found");
    assert_eq!(idx, 1);
}

#[test]
fn duplicate_id_causes_error() {
    let rows = vec![
        vec![
            "ID".to_string(),
            "Title".to_string(),
            "Abstract".to_string(),
        ],
        vec![
            "a1".to_string(),
            "Title 1".to_string(),
            "Text 1".to_string(),
        ],
        vec![
            "a1".to_string(),
            "Title 2".to_string(),
            "Text 2".to_string(),
        ],
    ];
    let header_idx = find_header_row(&rows, &[]).unwrap();
    let res = parse_abstracts_from_rows(&rows, header_idx);
    assert!(res.is_err(), "expected duplicate id to error");
}

#[test]
fn slug_collision_appends_suffix() {
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::Path;
    // create a temporary directory for session output
    let out = "target/test-output";
    let session_dir = Path::new(out).join("session-1");
    let _ = remove_dir_all(out);
    create_dir_all(&session_dir).expect("create session dir");

    // create a file that would collide with generated name
    std::fs::write(session_dir.join("0001-duplicate.md"), "existing").unwrap();

    // prepare abstracts and session
    let mut abstracts = std::collections::HashMap::new();
    abstracts.insert(
        "a1".to_string(),
        symposium_booklet::model::Abstract {
            id: "a1".to_string(),
            title: "Duplicate".to_string(),
            authors: vec!["A".to_string()],
            affiliation: None,
            center: None,
            contact_email: None,
            abstract_text: "T".to_string(),
            abstract_sections: Vec::new(),
            keywords: vec![],
            take_home: None,
            reference: None,
            literature: None,
            locale: "en".to_string(),
        },
    );
    let session = symposium_booklet::model::Session {
        id: "s1".to_string(),
        title: "Session 1".to_string(),
        order: 1,
        items: vec![symposium_booklet::model::ItemRef {
            id: "a1".to_string(),
            order: 1,
        }],
    };

    symposium_booklet::io::markdown::write_markdown(&abstracts, &vec![session], out).unwrap();

    // expect original and a suffixed file exist
    assert!(session_dir.join("0001-duplicate.md").exists());
    assert!(session_dir.join("0001-duplicate-1.md").exists());

    let _ = remove_dir_all(out);
}
