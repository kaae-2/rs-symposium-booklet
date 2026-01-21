use symposium_booklet::io::excel::{find_header_row, parse_abstracts_from_rows};

#[test]
fn header_detection_finds_header() {
    let rows = vec![
        vec!["Some meta".to_string(), "".to_string()],
        vec!["ID".to_string(), "Title".to_string(), "Abstract".to_string()],
        vec!["a1".to_string(), "My title".to_string(), "text".to_string()],
    ];
    let idx = find_header_row(&rows, &[]).expect("header should be found");
    assert_eq!(idx, 1);
}

#[test]
fn duplicate_id_causes_error() {
    let rows = vec![
        vec!["ID".to_string(), "Title".to_string(), "Abstract".to_string()],
        vec!["a1".to_string(), "Title 1".to_string(), "Text 1".to_string()],
        vec!["a1".to_string(), "Title 2".to_string(), "Text 2".to_string()],
    ];
    let header_idx = find_header_row(&rows, &[]).unwrap();
    let res = parse_abstracts_from_rows(&rows, header_idx);
    assert!(res.is_err(), "expected duplicate id to error");
}
