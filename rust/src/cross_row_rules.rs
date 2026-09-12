//! Cross-row business rules: duplicates and same-seller/same-day thresholds.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::config::{CATEGORY_TRUNG_LAP, CATEGORY_VUOT_NGUONG};
use crate::helpers::threshold_for_date;
use crate::row::InvoiceRow;
use crate::violations::{Extra, Rule, Violation};

pub struct DuplicateRowsRule;

impl Rule for DuplicateRowsRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        // BTreeMap keeps groups in sorted-key order, matching pandas'
        // `groupby(..., dropna=False).ngroup()` (sort=True by default). Note: `CellKey`
        // sorts `Blank` before other variants, which differs from pandas sorting NaN
        // *last* -- a cosmetic-only deviation in which integer group label a duplicate
        // set with blank key fields gets, not in which rows are flagged.
        let mut groups: BTreeMap<[crate::row::CellKey; 6], Vec<usize>> = BTreeMap::new();
        for (idx, row) in rows.iter().enumerate() {
            groups.entry(row.duplicate_key()).or_default().push(idx);
        }

        let mut violations = Vec::new();
        let mut group_number = 0usize;
        for (_, indices) in groups {
            if indices.len() < 2 {
                continue;
            }
            group_number += 1;
            for idx in indices {
                violations.push(Violation {
                    excel_row: rows[idx].excel_row,
                    rule_code: "duplicate_rows",
                    category: CATEGORY_TRUNG_LAP,
                    message: "Trung lap".to_string(),
                    extra: Extra::Duplicate {
                        group: group_number,
                    },
                });
            }
        }
        violations
    }
}

pub struct OverThresholdRule {
    pub threshold: Option<i64>,
}

impl OverThresholdRule {
    pub fn new(threshold: Option<i64>) -> Self {
        OverThresholdRule { threshold }
    }
}

impl Rule for OverThresholdRule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation> {
        let mut per_row: Vec<(usize, String, NaiveDate, f64)> = Vec::new();
        for (idx, row) in rows.iter().enumerate() {
            if row.tax_code.is_blank() {
                continue;
            }
            let Some(date) = row.invoice_date.as_date() else {
                continue;
            };
            let row_total = row.amount_excl_vat.to_numeric().unwrap_or(0.0)
                + row.deductible_vat.to_numeric().unwrap_or(0.0);
            per_row.push((idx, row.tax_code.as_display_string(), date, row_total));
        }

        let mut totals: BTreeMap<(String, NaiveDate), f64> = BTreeMap::new();
        for (_, tax_code, date, row_total) in &per_row {
            *totals.entry((tax_code.clone(), *date)).or_insert(0.0) += row_total;
        }

        let mut flagged: Vec<(usize, String, NaiveDate, f64, i64)> = Vec::new();
        for (idx, tax_code, date, _) in &per_row {
            let daily_total = totals[&(tax_code.clone(), *date)];
            let applied_threshold = self.threshold.unwrap_or_else(|| threshold_for_date(*date));
            if daily_total >= applied_threshold as f64 {
                flagged.push((
                    *idx,
                    tax_code.clone(),
                    *date,
                    daily_total,
                    applied_threshold,
                ));
            }
        }

        flagged.sort_by(|a, b| {
            (&a.1, a.2, rows[a.0].excel_row).cmp(&(&b.1, b.2, rows[b.0].excel_row))
        });

        flagged
            .into_iter()
            .map(|(idx, _, _, daily_total, applied_threshold)| Violation {
                excel_row: rows[idx].excel_row,
                rule_code: "over_threshold",
                category: CATEGORY_VUOT_NGUONG,
                message: "Vuot nguong".to_string(),
                extra: Extra::Threshold {
                    daily_total,
                    applied_threshold,
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::CellValue;

    fn base_row(excel_row: usize) -> InvoiceRow {
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
    fn duplicate_rows_flags_exact_repeats_and_numbers_groups() {
        let r1 = base_row(15);
        let r2 = base_row(16); // exact duplicate of r1
        let mut r3 = base_row(17);
        r3.invoice_number = CellValue::Text("HD002".into());
        let r4 = base_row(18); // another duplicate of r1/r2's key
        let violations = DuplicateRowsRule.check(&[r1, r2, r3, r4]);
        assert_eq!(violations.len(), 3);
        for v in &violations {
            assert_eq!(v.extra, Extra::Duplicate { group: 1 });
        }
        let flagged_rows: Vec<usize> = violations.iter().map(|v| v.excel_row).collect();
        assert_eq!(flagged_rows, vec![15, 16, 18]);
    }

    #[test]
    fn duplicate_rows_ignores_unique_rows() {
        let r1 = base_row(15);
        let mut r2 = base_row(16);
        r2.invoice_number = CellValue::Text("HD002".into());
        assert!(DuplicateRowsRule.check(&[r1, r2]).is_empty());
    }

    #[test]
    fn over_threshold_sums_same_seller_same_day() {
        let mut r1 = base_row(15);
        r1.amount_excl_vat = CellValue::Number(15_000_000.0);
        r1.deductible_vat = CellValue::Number(0.0);
        let mut r2 = base_row(16);
        r2.invoice_number = CellValue::Text("HD002".into());
        r2.amount_excl_vat = CellValue::Number(10_000_000.0);
        r2.deductible_vat = CellValue::Number(0.0);
        // total for this seller/day = 25,000,000 >= 20,000,000 (2025-01-01 is before the cutover)
        let violations = OverThresholdRule::new(None).check(&[r1, r2]);
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert_eq!(
                v.extra,
                Extra::Threshold {
                    daily_total: 25_000_000.0,
                    applied_threshold: 20_000_000
                }
            );
        }
    }

    #[test]
    fn over_threshold_below_threshold_not_flagged() {
        let mut r1 = base_row(15);
        r1.amount_excl_vat = CellValue::Number(1_000_000.0);
        r1.deductible_vat = CellValue::Number(0.0);
        assert!(OverThresholdRule::new(None).check(&[r1]).is_empty());
    }

    #[test]
    fn over_threshold_different_sellers_not_combined() {
        let mut r1 = base_row(15);
        r1.amount_excl_vat = CellValue::Number(15_000_000.0);
        let mut r2 = base_row(16);
        r2.tax_code = CellValue::Text("9999999999".into());
        r2.amount_excl_vat = CellValue::Number(15_000_000.0);
        assert!(OverThresholdRule::new(None).check(&[r1, r2]).is_empty());
    }

    #[test]
    fn over_threshold_flat_override() {
        let mut r1 = base_row(15);
        r1.amount_excl_vat = CellValue::Number(6_000_000.0);
        r1.deductible_vat = CellValue::Number(0.0);
        // Below the default (20M pre-cutover) but above a flat 5,000,000 override.
        let violations = OverThresholdRule::new(Some(5_000_000)).check(&[r1]);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].extra,
            Extra::Threshold {
                daily_total: 6_000_000.0,
                applied_threshold: 5_000_000
            }
        );
    }

    #[test]
    fn over_threshold_uses_schedule_after_cutover() {
        let mut r1 = base_row(15);
        r1.invoice_date = CellValue::Date(NaiveDate::from_ymd_opt(2025, 7, 1).unwrap());
        r1.amount_excl_vat = CellValue::Number(6_000_000.0);
        r1.deductible_vat = CellValue::Number(0.0);
        let violations = OverThresholdRule::new(None).check(&[r1]);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].extra,
            Extra::Threshold {
                daily_total: 6_000_000.0,
                applied_threshold: 5_000_000
            }
        );
    }
}
