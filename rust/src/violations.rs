//! Common interfaces for invoice-validation rules and their results.

use crate::row::InvoiceRow;

/// Extra structured data carried by some violation categories, written into the
/// report's per-category sheet (see `report.rs`). Mirrors Python's free-form
/// `Violation.extra: dict`, but typed since only two shapes ever occur.
#[derive(Debug, Clone, PartialEq)]
pub enum Extra {
    None,
    Duplicate {
        group: usize,
    },
    Threshold {
        daily_total: f64,
        applied_threshold: i64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub excel_row: usize,
    pub rule_code: &'static str,
    pub category: &'static str,
    pub message: String,
    pub extra: Extra,
}

impl Violation {
    pub fn new(
        excel_row: usize,
        rule_code: &'static str,
        category: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Violation {
            excel_row,
            rule_code,
            category,
            message: message.into(),
            extra: Extra::None,
        }
    }
}

/// A validation rule that returns one violation for every flagged row.
pub trait Rule {
    fn check(&self, rows: &[InvoiceRow]) -> Vec<Violation>;
}
