//! Rule registry and execution for invoice validation.

use crate::cross_row_rules::{DuplicateRowsRule, OverThresholdRule};
use crate::field_rules::{AmountRule, InvalidDateRule, InvalidTaxCodeRule, MissingFieldRule};
use crate::row::InvoiceRow;
use crate::violations::{Rule, Violation};

/// Every rule but the threshold check, which needs a per-run `threshold` override
/// (see `default_rules_with_threshold`) -- mirrors Python's `DEFAULT_RULES[:-1]` splice
/// in `cli.py`/`gui.py`.
pub fn default_rules_without_threshold() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MissingFieldRule {
            field: |r| &r.invoice_number,
            message: "Thieu so hoa don",
        }),
        Box::new(InvalidDateRule),
        Box::new(MissingFieldRule {
            field: |r| &r.seller_name,
            message: "Thieu ten nguoi ban",
        }),
        Box::new(InvalidTaxCodeRule),
        Box::new(AmountRule {
            field: |r| &r.amount_excl_vat,
            label: "Doanh so mua chua co thue",
        }),
        Box::new(AmountRule {
            field: |r| &r.deductible_vat,
            label: "Thue GTGT",
        }),
        Box::new(DuplicateRowsRule),
    ]
}

/// The full default registry (used by the GUI, which never overrides the threshold).
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    let mut rules = default_rules_without_threshold();
    rules.push(Box::new(OverThresholdRule::new(None)));
    rules
}

/// The registry used by the CLI, which may pass a flat `--threshold` override.
pub fn default_rules_with_threshold(threshold: Option<i64>) -> Vec<Box<dyn Rule>> {
    let mut rules = default_rules_without_threshold();
    rules.push(Box::new(OverThresholdRule::new(threshold)));
    rules
}

/// Run each registered rule, retaining registry order in the result.
pub fn run_rules(rows: &[InvoiceRow], rules: &[Box<dyn Rule>]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for rule in rules {
        violations.extend(rule.check(rows));
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::CellValue;
    use chrono::NaiveDate;

    #[test]
    fn run_rules_preserves_registry_order() {
        let row = InvoiceRow {
            excel_row: 15,
            row_number: CellValue::Number(1.0),
            invoice_number: CellValue::Blank,
            invoice_date: CellValue::Date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            seller_name: CellValue::Blank,
            tax_code: CellValue::Text("0301234567".into()),
            amount_excl_vat: CellValue::Number(1.0),
            deductible_vat: CellValue::Number(1.0),
            note: CellValue::Blank,
        };
        let violations = run_rules(&[row], &default_rules_with_threshold(None));
        let codes: Vec<&str> = violations.iter().map(|v| v.rule_code).collect();
        assert_eq!(codes, vec!["missing_field", "missing_field"]);
    }
}
