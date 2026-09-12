//! The per-row data model that replaces pandas' `DataFrame` in the Python version.
//!
//! Each of the 8 input columns is kept as a [`CellValue`] rather than being eagerly
//! coerced, because the report needs to reproduce a flagged row's *original* cell
//! content (see `report.rs`), not just the value used for rule evaluation.

use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Blank,
    Text(String),
    Number(f64),
    Date(NaiveDate),
}

impl CellValue {
    /// Mirrors Python's `helpers.is_blank`: `pd.isna(value) or str(value).strip() == ""`.
    pub fn is_blank(&self) -> bool {
        match self {
            CellValue::Blank => true,
            CellValue::Text(s) => s.trim().is_empty(),
            CellValue::Number(_) | CellValue::Date(_) => false,
        }
    }

    /// Mirrors Python's implicit `str(value)` used before regex/length checks.
    pub fn as_display_string(&self) -> String {
        match self {
            CellValue::Blank => String::new(),
            CellValue::Text(s) => s.clone(),
            CellValue::Number(n) => format_number(*n),
            CellValue::Date(d) => d.format("%Y-%m-%d").to_string(),
        }
    }

    /// Mirrors `pd.to_numeric(series, errors="coerce")`: `None` for blank, non-numeric
    /// text, or a date cell; `Some(v)` for a genuinely numeric cell or parseable text.
    pub fn to_numeric(&self) -> Option<f64> {
        match self {
            CellValue::Blank => None,
            CellValue::Number(n) => Some(*n),
            CellValue::Text(s) => s.trim().parse::<f64>().ok(),
            CellValue::Date(_) => None,
        }
    }

    /// `Some(date)` only for a genuinely date-typed cell -- mirrors Python's
    /// `isinstance(value, pd.Timestamp)` check, which does NOT parse date-looking text.
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            CellValue::Date(d) => Some(*d),
            _ => None,
        }
    }

    /// A hashable/orderable normalization used for duplicate-key and group-key
    /// comparisons (see `cross_row_rules.rs`). Note: this sorts `Blank` before any
    /// other variant, which differs from pandas' groupby(sort=True), where NaN keys
    /// sort *last* -- a documented, cosmetic-only deviation (see `CellKey`).
    pub fn key(&self) -> CellKey {
        match self {
            CellValue::Blank => CellKey::Blank,
            CellValue::Text(s) => CellKey::Text(s.clone()),
            CellValue::Number(n) => CellKey::Number(n.to_bits()),
            CellValue::Date(d) => CellKey::Date(*d),
        }
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKey {
    Blank,
    Text(String),
    Number(u64),
    Date(NaiveDate),
}

/// One data row (columns B-I). Field names are English even though the source
/// spreadsheet and the business messages/report sheet names are Vietnamese --
/// see `config::CATEGORY_*` and the rule messages, which stay Vietnamese since
/// they're user-facing output, not internal identifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct InvoiceRow {
    pub excel_row: usize,
    pub row_number: CellValue,
    pub invoice_number: CellValue,
    pub invoice_date: CellValue,
    pub seller_name: CellValue,
    pub tax_code: CellValue,
    pub amount_excl_vat: CellValue,
    pub deductible_vat: CellValue,
    pub note: CellValue,
}

impl InvoiceRow {
    /// Mirrors `df.dropna(how="all", subset=COLUMNS)` in `loader.py`.
    pub fn is_completely_blank(&self) -> bool {
        self.row_number.is_blank()
            && self.invoice_number.is_blank()
            && self.invoice_date.is_blank()
            && self.seller_name.is_blank()
            && self.tax_code.is_blank()
            && self.amount_excl_vat.is_blank()
            && self.deductible_vat.is_blank()
            && self.note.is_blank()
    }

    /// The 6-field duplicate key from `config.DUPLICATE_KEY`.
    pub fn duplicate_key(&self) -> [CellKey; 6] {
        [
            self.invoice_number.key(),
            self.invoice_date.key(),
            self.seller_name.key(),
            self.tax_code.key(),
            self.amount_excl_vat.key(),
            self.deductible_vat.key(),
        ]
    }
}
