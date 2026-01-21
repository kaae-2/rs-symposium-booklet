use std::path::Path;
use std::error::Error;

pub fn make_fixture(path: &str) -> Result<(), umya_spreadsheet::writer::xlsx::XlsxError> {
    let mut book = umya_spreadsheet::new_file();
    let sheet_name = "abstracts";
    book.new_sheet(sheet_name);
    let sheet = book.get_sheet_by_name_mut(sheet_name).unwrap();
    sheet.get_cell_by_column_and_row_mut(1, 1).set_value("id");
    sheet.get_cell_by_column_and_row_mut(2, 1).set_value("title");
    sheet.get_cell_by_column_and_row_mut(3, 1).set_value("locale");
    sheet.get_cell_by_column_and_row_mut(4, 1).set_value("abstract");
    sheet.get_cell_by_column_and_row_mut(1, 2).set_value("f1");
    sheet.get_cell_by_column_and_row_mut(2, 2).set_value("Fixture One");
    sheet.get_cell_by_column_and_row_mut(3, 2).set_value("en");
    sheet.get_cell_by_column_and_row_mut(4, 2).set_value("Text one");

    book.new_sheet("sessions");
    let s = book.get_sheet_by_name_mut("sessions").unwrap();
    s.get_cell_by_column_and_row_mut(1, 1).set_value("Session 1");
    s.get_cell_by_column_and_row_mut(1, 2).set_value("f1");

    umya_spreadsheet::writer::xlsx::write(&book, Path::new(path))?;
    Ok(())
}
