//! Write rule violations out to a report workbook.
//!
//! Column headers written into the workbook intentionally match the original
//! Python tool's output verbatim (e.g. `so_hoa_don`, `nhom_trung`, `tong_theo_ngay`)
//! -- these are report *content*, not internal Rust identifiers, and changing them
//! would break the "same output workbook" goal of this rewrite.

use std::collections::HashMap;
use std::path::Path;

use rust_xlsxwriter::{Workbook, Worksheet};

use crate::config::{CATEGORY_DU_LIEU_LOI, CATEGORY_TRUNG_LAP, CATEGORY_VUOT_NGUONG};
use crate::row::{CellValue, InvoiceRow};
use crate::violations::{Extra, Violation};

const BASE_HEADERS: [&str; 9] = [
    "excel_row",
    "stt",
    "so_hoa_don",
    "ngay_lap_hoa_don",
    "ten_nguoi_ban",
    "ma_so_thue",
    "doanh_so_mua_chua_thue",
    "thue_gtgt",
    "ghi_chu",
];

pub fn write_report(
    output: &Path,
    total_rows: usize,
    rows: &[InvoiceRow],
    violations: &[Violation],
    threshold: Option<i64>,
) -> anyhow::Result<()> {
    let threshold_label = match threshold {
        Some(t) => format!("{} VND (flat override)", format_thousands(t)),
        None => "theo ngay hieu luc".to_string(),
    };

    let rows_by_excel_row: HashMap<usize, &InvoiceRow> =
        rows.iter().map(|r| (r.excel_row, r)).collect();

    // Preserve first-appearance order of categories, matching Python's
    // `dict.setdefault` insertion order (rule-registry order).
    let mut category_order: Vec<&'static str> = Vec::new();
    let mut by_category: HashMap<&'static str, Vec<&Violation>> = HashMap::new();
    for v in violations {
        by_category.entry(v.category).or_insert_with(|| {
            category_order.push(v.category);
            Vec::new()
        });
        by_category.get_mut(v.category).unwrap().push(v);
    }

    let mut workbook = Workbook::new();
    for category in &category_order {
        let category_violations = &by_category[category];
        let mut worksheet = Worksheet::new();
        worksheet.set_name(*category)?;
        write_category_sheet(
            &mut worksheet,
            &rows_by_excel_row,
            category,
            category_violations,
        )?;
        workbook.push_worksheet(worksheet);
    }

    let du_lieu_loi_count = row_count(by_category.get(CATEGORY_DU_LIEU_LOI));
    let trung_lap_count = row_count(by_category.get(CATEGORY_TRUNG_LAP));
    let vuot_nguong_count = row_count(by_category.get(CATEGORY_VUOT_NGUONG));

    let mut summary = Worksheet::new();
    summary.set_name("Tong_hop")?;
    summary.write(0, 0, "Loai kiem tra")?;
    summary.write(0, 1, "So dong")?;
    summary.write(1, 0, "Tong so dong")?;
    summary.write(1, 1, total_rows as f64)?;
    summary.write(2, 0, "Du lieu loi")?;
    summary.write(2, 1, du_lieu_loi_count as f64)?;
    summary.write(3, 0, "Trung lap")?;
    summary.write(3, 1, trung_lap_count as f64)?;
    summary.write(4, 0, format!("Vuot nguong ({threshold_label})"))?;
    summary.write(4, 1, vuot_nguong_count as f64)?;
    workbook.push_worksheet(summary);

    workbook.save(output)?;
    Ok(())
}

fn row_count(violations: Option<&Vec<&Violation>>) -> usize {
    let Some(violations) = violations else {
        return 0;
    };
    let mut seen: Vec<usize> = violations.iter().map(|v| v.excel_row).collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

fn write_category_sheet(
    worksheet: &mut Worksheet,
    rows_by_excel_row: &HashMap<usize, &InvoiceRow>,
    category: &str,
    violations: &[&Violation],
) -> anyhow::Result<()> {
    // Row -> joined messages, in the order violations for this category occurred.
    let mut messages: Vec<(usize, Vec<&str>)> = Vec::new();
    let mut message_index: HashMap<usize, usize> = HashMap::new();
    // First violation seen for a given row, used for per-row extras (nhom_trung, etc.)
    let mut first_extra: HashMap<usize, &Extra> = HashMap::new();
    for v in violations {
        let idx = *message_index.entry(v.excel_row).or_insert_with(|| {
            messages.push((v.excel_row, Vec::new()));
            messages.len() - 1
        });
        messages[idx].1.push(&v.message);
        first_extra.entry(v.excel_row).or_insert(&v.extra);
    }
    let joined_messages: HashMap<usize, String> = messages
        .iter()
        .map(|(row, msgs)| (*row, msgs.join(" | ")))
        .collect();

    let mut excel_rows: Vec<usize> = joined_messages.keys().copied().collect();

    match category {
        CATEGORY_TRUNG_LAP => {
            excel_rows.sort_by_key(|row| {
                let group = match first_extra.get(row) {
                    Some(Extra::Duplicate { group }) => *group,
                    _ => 0,
                };
                (group, *row)
            });
            write_headers(worksheet, &["nhom_trung"])?;
            for (r, excel_row) in excel_rows.iter().enumerate() {
                let invoice_row = rows_by_excel_row[excel_row];
                write_base_columns(worksheet, r as u32 + 1, invoice_row)?;
                let group = match first_extra.get(excel_row) {
                    Some(Extra::Duplicate { group }) => *group,
                    _ => 0,
                };
                worksheet.write(r as u32 + 1, BASE_HEADERS.len() as u16, group as f64)?;
            }
        }
        CATEGORY_VUOT_NGUONG => {
            excel_rows.sort_by(|a, b| {
                let ra = rows_by_excel_row[a];
                let rb = rows_by_excel_row[b];
                (
                    ra.tax_code.as_display_string(),
                    ra.invoice_date.as_date(),
                    *a,
                )
                    .cmp(&(
                        rb.tax_code.as_display_string(),
                        rb.invoice_date.as_date(),
                        *b,
                    ))
            });
            write_headers(
                worksheet,
                &["tong_tien", "tong_theo_ngay", "nguong_ap_dung"],
            )?;
            for (r, excel_row) in excel_rows.iter().enumerate() {
                let invoice_row = rows_by_excel_row[excel_row];
                write_base_columns(worksheet, r as u32 + 1, invoice_row)?;
                let row_total = invoice_row.amount_excl_vat.to_numeric().unwrap_or(0.0)
                    + invoice_row.deductible_vat.to_numeric().unwrap_or(0.0);
                let (daily_total, applied_threshold) = match first_extra.get(excel_row) {
                    Some(Extra::Threshold {
                        daily_total,
                        applied_threshold,
                    }) => (*daily_total, *applied_threshold),
                    _ => (0.0, 0),
                };
                let col = BASE_HEADERS.len() as u16;
                worksheet.write(r as u32 + 1, col, row_total)?;
                worksheet.write(r as u32 + 1, col + 1, daily_total)?;
                worksheet.write(r as u32 + 1, col + 2, applied_threshold as f64)?;
            }
        }
        // CATEGORY_DU_LIEU_LOI and any future/unknown category share this default shape.
        _ => {
            excel_rows.sort_unstable();
            write_headers(worksheet, &["loi"])?;
            for (r, excel_row) in excel_rows.iter().enumerate() {
                let invoice_row = rows_by_excel_row[excel_row];
                write_base_columns(worksheet, r as u32 + 1, invoice_row)?;
                let col = BASE_HEADERS.len() as u16;
                worksheet.write(r as u32 + 1, col, joined_messages[excel_row].as_str())?;
            }
        }
    }
    Ok(())
}

fn write_headers(worksheet: &mut Worksheet, extra: &[&str]) -> anyhow::Result<()> {
    for (col, header) in BASE_HEADERS.iter().enumerate() {
        worksheet.write(0, col as u16, *header)?;
    }
    for (i, header) in extra.iter().enumerate() {
        worksheet.write(0, (BASE_HEADERS.len() + i) as u16, *header)?;
    }
    Ok(())
}

fn write_base_columns(
    worksheet: &mut Worksheet,
    row: u32,
    invoice_row: &InvoiceRow,
) -> anyhow::Result<()> {
    worksheet.write(row, 0, invoice_row.excel_row as f64)?;
    write_cell(worksheet, row, 1, &invoice_row.row_number)?;
    write_cell(worksheet, row, 2, &invoice_row.invoice_number)?;
    write_cell(worksheet, row, 3, &invoice_row.invoice_date)?;
    write_cell(worksheet, row, 4, &invoice_row.seller_name)?;
    write_cell(worksheet, row, 5, &invoice_row.tax_code)?;
    write_cell(worksheet, row, 6, &invoice_row.amount_excl_vat)?;
    write_cell(worksheet, row, 7, &invoice_row.deductible_vat)?;
    write_cell(worksheet, row, 8, &invoice_row.note)?;
    Ok(())
}

fn write_cell(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    value: &CellValue,
) -> anyhow::Result<()> {
    match value {
        CellValue::Blank => {}
        CellValue::Text(s) => {
            worksheet.write(row, col, s.as_str())?;
        }
        CellValue::Number(n) => {
            worksheet.write(row, col, *n)?;
        }
        CellValue::Date(d) => {
            worksheet.write(row, col, d)?;
        }
    };
    Ok(())
}

fn format_thousands(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let grouped: String = grouped.chars().rev().collect();
    if negative {
        format!("-{grouped}")
    } else {
        grouped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{default_rules_with_threshold, run_rules};
    use calamine::{Reader, Xlsx, open_workbook};
    use chrono::NaiveDate;

    #[test]
    fn format_thousands_groups_correctly() {
        assert_eq!(format_thousands(5_000_000), "5,000,000");
        assert_eq!(format_thousands(999), "999");
        assert_eq!(format_thousands(1_000), "1,000");
    }

    fn row(excel_row: usize) -> InvoiceRow {
        InvoiceRow {
            excel_row,
            row_number: CellValue::Number(1.0),
            invoice_number: CellValue::Blank, // triggers missing_field
            invoice_date: CellValue::Date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            seller_name: CellValue::Text("Cong ty A".into()),
            tax_code: CellValue::Text("0301234567".into()),
            amount_excl_vat: CellValue::Number(30_000_000.0), // triggers over_threshold
            deductible_vat: CellValue::Number(0.0),
            note: CellValue::Blank,
        }
    }

    #[test]
    fn write_report_produces_expected_sheets_and_summary() {
        let rows = vec![row(15)];
        let violations = run_rules(&rows, &default_rules_with_threshold(None));
        assert!(!violations.is_empty());

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("report.xlsx");
        write_report(&output, rows.len(), &rows, &violations, None).unwrap();

        let mut workbook: Xlsx<_> = open_workbook(&output).unwrap();
        let sheet_names = workbook.sheet_names().to_vec();
        assert!(sheet_names.contains(&"Du_lieu_loi".to_string()));
        assert!(sheet_names.contains(&"Vuot_nguong".to_string()));
        assert!(sheet_names.contains(&"Tong_hop".to_string()));

        let summary = workbook.worksheet_range("Tong_hop").unwrap();
        // Row 1 (index 1): "Tong so dong", 1
        assert_eq!(summary.get_value((1, 1)).unwrap().to_string(), "1");
    }
}
