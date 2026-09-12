//! Small pure functions shared across rule modules (no Rule/Violation/config-data coupling).

use chrono::NaiveDate;

use crate::config::THRESHOLD_SCHEDULE;
use crate::row::CellValue;

pub fn is_blank(value: &CellValue) -> bool {
    value.is_blank()
}

pub fn tax_code_digits(value: &CellValue) -> String {
    if is_blank(value) {
        return String::new();
    }
    value
        .as_display_string()
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect()
}

/// Threshold in effect on a given invoice date, per `config::THRESHOLD_SCHEDULE`.
pub fn threshold_for_date(invoice_date: NaiveDate) -> i64 {
    let mut applicable = THRESHOLD_SCHEDULE[0].1;
    for (effective_from, amount) in THRESHOLD_SCHEDULE {
        let applies = match effective_from {
            None => true,
            Some(date_str) => {
                let cutoff = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .expect("THRESHOLD_SCHEDULE dates must be valid YYYY-MM-DD");
                invoice_date >= cutoff
            }
        };
        if applies {
            applicable = *amount;
        } else {
            break;
        }
    }
    applicable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tax_code_digits_strips_dashes_and_spaces() {
        assert_eq!(
            tax_code_digits(&CellValue::Text("030-123-4567".into())),
            "0301234567"
        );
        assert_eq!(
            tax_code_digits(&CellValue::Text("03 01234567".into())),
            "0301234567"
        );
    }

    #[test]
    fn tax_code_digits_blank_is_empty() {
        assert_eq!(tax_code_digits(&CellValue::Blank), "");
        assert_eq!(tax_code_digits(&CellValue::Text("   ".into())), "");
    }

    #[test]
    fn threshold_before_and_after_cutover() {
        let before = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let on_cutover = NaiveDate::from_ymd_opt(2025, 7, 1).unwrap();
        let after = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert_eq!(threshold_for_date(before), 20_000_000);
        assert_eq!(threshold_for_date(on_cutover), 5_000_000);
        assert_eq!(threshold_for_date(after), 5_000_000);
    }
}
