//! Row-level data-quality validation rules.

use crate::config::{CATEGORY_DU_LIEU_LOI, VALID_TAX_CODE_LENGTHS};
use crate::helpers::{is_blank, tax_code_digits};
use crate::row::{CellValue, InvoiceRow};
use crate::violations::{Rule, Violation};

pub struct MissingFieldRule {
    pub field: fn(&InvoiceRow) -> &CellValue,
    pub message: &'static str,
}

impl Rule for MissingFieldRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        rows.iter()
            .filter(|row| is_blank((self.field)(row)))
            .map(|row| {
                Violation::new(
                    row.excel_row,
                    "missing_field",
                    CATEGORY_DU_LIEU_LOI,
                    self.message,
                )
            })
            .collect()
    }
}

pub struct InvalidDateRule;

impl Rule for InvalidDateRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        rows.iter()
            .filter(|row| row.invoice_date.as_date().is_none())
            .map(|row| {
                Violation::new(
                    row.excel_row,
                    "invalid_date",
                    CATEGORY_DU_LIEU_LOI,
                    "Ngay lap hoa don khong hop le",
                )
            })
            .collect()
    }
}

pub struct InvalidTaxCodeRule;

impl Rule for InvalidTaxCodeRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        rows.iter()
            .filter(|row| {
                let len = tax_code_digits(&row.tax_code).chars().count();
                !VALID_TAX_CODE_LENGTHS.contains(&len)
            })
            .map(|row| {
                Violation::new(
                    row.excel_row,
                    "invalid_tax_code",
                    CATEGORY_DU_LIEU_LOI,
                    "Ma so thue khong dung dinh dang",
                )
            })
            .collect()
    }
}

pub struct AmountRule {
    pub field: fn(&InvoiceRow) -> &CellValue,
    pub label: &'static str,
}

impl Rule for AmountRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        let mut violations = Vec::new();
        for row in rows {
            match (self.field)(row).to_numeric() {
                None => violations.push(Violation::new(
                    row.excel_row,
                    "invalid_amount",
                    CATEGORY_DU_LIEU_LOI,
                    format!("{}: thieu hoac khong phai so", self.label),
                )),
                Some(v) if v < 0.0 => violations.push(Violation::new(
                    row.excel_row,
                    "invalid_amount",
                    CATEGORY_DU_LIEU_LOI,
                    format!("{}: gia tri am", self.label),
                )),
                Some(_) => {}
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn row(excel_row: usize) -> InvoiceRow {
        InvoiceRow {
            excel_row,
            row_number: CellValue::Number(1.0),
            invoice_number: CellValue::Text("HD001".into()),
            invoice_date: CellValue::Date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            seller_name: CellValue::Text("Cong ty A".into()),
            tax_code: CellValue::Text("0301234567".into()),
            amount_excl_vat: CellValue::Number(1_000_000.0),
            deductible_vat: CellValue::Number(100_000.0),
            note: CellValue::Blank,
        }
    }

    #[test]
    fn missing_field_flags_blank_only() {
        let mut r1 = row(15);
        r1.invoice_number = CellValue::Blank;
        let r2 = row(16);
        let rule = MissingFieldRule {
            field: |r| &r.invoice_number,
            message: "Thieu so hoa don",
        };
        let violations = rule.check(&[r1, r2]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].excel_row, 15);
    }

    #[test]
    fn invalid_date_flags_non_date_cells() {
        let mut r1 = row(15);
        r1.invoice_date = CellValue::Text("2025-01-01".into()); // text, not a real date cell
        let r2 = row(16);
        let violations = InvalidDateRule.check(&[r1, r2]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].excel_row, 15);
    }

    #[test]
    fn tax_code_lengths() {
        let mut valid10 = row(15);
        valid10.tax_code = CellValue::Text("030-1234567".into()); // 10 digits with dash
        let mut valid12 = row(16);
        valid12.tax_code = CellValue::Text("030123456789".into()); // 12 digits
        let mut valid13 = row(17);
        valid13.tax_code = CellValue::Text("0301234567-001".into()); // 13 digits
        let mut invalid11 = row(18);
        invalid11.tax_code = CellValue::Text("42002400380".into()); // 11 digits
        let violations = InvalidTaxCodeRule.check(&[valid10, valid12, valid13, invalid11]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].excel_row, 18);
    }

    #[test]
    fn amount_rule_flags_non_numeric_and_negative_separately() {
        let mut missing = row(15);
        missing.amount_excl_vat = CellValue::Blank;
        let mut non_numeric = row(16);
        non_numeric.amount_excl_vat = CellValue::Text("abc".into());
        let mut negative = row(17);
        negative.amount_excl_vat = CellValue::Number(-5.0);
        let ok = row(18);

        let rule = AmountRule {
            field: |r| &r.amount_excl_vat,
            label: "Doanh so mua chua co thue",
        };
        let violations = rule.check(&[missing, non_numeric, negative, ok]);
        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("thieu hoac khong phai so"));
        assert!(violations[1].message.contains("thieu hoac khong phai so"));
        assert!(violations[2].message.contains("gia tri am"));
    }
}
