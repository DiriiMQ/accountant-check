//! Read the invoice list sheet into a flat `Vec<InvoiceRow>`.

use std::path::Path;

use calamine::{Data, DataType, Reader, Xlsx, open_workbook};

use crate::config::HEADER_ROWS;
use crate::row::{CellValue, InvoiceRow};

/// `so_hoa_don` and `ma_so_thue` must stay text: a genuinely numeric cell (e.g. an
/// invoice number stored as `00002224`) would otherwise render as `2224`, silently
/// dropping leading zeros -- see `README.md`'s documented leading-zero caveat. Forcing
/// these two columns through `as_display_string()` (rather than keeping `Number`)
/// mirrors Python's `dtype={"so_hoa_don": str, "ma_so_thue": str}` in the old loader.
fn force_text(cell: CellValue) -> CellValue {
    match cell {
        CellValue::Blank => CellValue::Blank,
        other => {
            let s = other.as_display_string();
            if s.trim().is_empty() {
                CellValue::Blank
            } else {
                CellValue::Text(s)
            }
        }
    }
}

fn parse_generic_cell(cell: &Data) -> CellValue {
    match cell {
        Data::Empty => CellValue::Blank,
        Data::String(s) if s.trim().is_empty() => CellValue::Blank,
        Data::String(s) => CellValue::Text(s.clone()),
        Data::Float(f) => CellValue::Number(*f),
        Data::Int(i) => CellValue::Number(*i as f64),
        Data::Bool(b) => CellValue::Text(b.to_string()),
        // A cell genuinely typed as a date/time -- only this variant counts as a
        // valid invoice date, matching openpyxl/pandas: date-*looking* text stays text.
        Data::DateTime(_) => match cell.as_date() {
            Some(d) => CellValue::Date(d),
            None => CellValue::Text(cell.to_string()),
        },
        _ => CellValue::Text(cell.to_string()),
    }
}

pub fn load_invoices(path: &Path) -> anyhow::Result<Vec<InvoiceRow>> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("workbook has no sheets"))?;
    let range = workbook.worksheet_range(&sheet_name)?;

    let mut rows = Vec::new();
    // `range.rows()` yields slices indexed *relative to the range's bounding box*, not
    // absolute sheet columns -- since this sheet's used area often starts at column B
    // (column A is never written), that would silently shift every column by one. Use
    // `get_value` with absolute (row, col) coordinates instead. HEADER_ROWS rows
    // (1-indexed 1..=14) are title/header; data starts at Excel row 15, i.e. 0-indexed
    // row HEADER_ROWS. Columns B..=I are 0-indexed 1..=8.
    let Some((end_row, _end_col)) = range.end() else {
        return Ok(rows);
    };
    for row_idx in (HEADER_ROWS as u32)..=end_row {
        let get = |col: u32| {
            range
                .get_value((row_idx, col))
                .map(parse_generic_cell)
                .unwrap_or(CellValue::Blank)
        };
        let row = InvoiceRow {
            excel_row: row_idx as usize + 1,
            row_number: get(1),
            invoice_number: force_text(get(2)),
            invoice_date: get(3),
            seller_name: get(4),
            tax_code: force_text(get(5)),
            amount_excl_vat: get(6),
            deductible_vat: get(7),
            note: get(8),
        };
        if !row.is_completely_blank() {
            rows.push(row);
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_xlsxwriter::{Format, Workbook};

    fn write_fixture(path: &Path) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        // 14 header rows (0-indexed 0..=13), data starts at row index 14 (Excel row 15).
        for r in 0..14u32 {
            worksheet.write(r, 1, format!("header {r}")).unwrap();
        }
        let date_format = Format::new().set_num_format("yyyy-mm-dd");
        // Row 15 (index 14): a full data row.
        worksheet.write(14, 1, 1).unwrap(); // stt
        worksheet.write(14, 2, "HD001").unwrap(); // so_hoa_don
        worksheet
            .write_with_format(
                14,
                3,
                &NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                &date_format,
            )
            .unwrap();
        worksheet.write(14, 4, "Cong ty A").unwrap();
        worksheet.write(14, 5, "0301234567").unwrap();
        worksheet.write(14, 6, 1_000_000).unwrap();
        worksheet.write(14, 7, 100_000).unwrap();
        // Row 16 (index 15): a second data row.
        worksheet.write(15, 1, 2).unwrap();
        worksheet.write(15, 2, "HD002").unwrap();
        worksheet
            .write_with_format(
                15,
                3,
                &NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
                &date_format,
            )
            .unwrap();
        worksheet.write(15, 4, "Cong ty B").unwrap();
        worksheet.write(15, 5, "0309876543").unwrap();
        worksheet.write(15, 6, 2_000_000).unwrap();
        worksheet.write(15, 7, 200_000).unwrap();
        // Row 17 (index 16): fully blank trailing row -- must be dropped.
        workbook.save(path).unwrap();
    }

    #[test]
    fn loads_starting_at_row_15_and_drops_blank_trailing_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.xlsx");
        write_fixture(&path);

        let rows = load_invoices(&path).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].excel_row, 15);
        assert_eq!(rows[0].invoice_number, CellValue::Text("HD001".into()));
        assert_eq!(
            rows[0].invoice_date.as_date(),
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        );
        assert_eq!(rows[1].excel_row, 16);
    }
}
