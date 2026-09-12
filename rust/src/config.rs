//! Input layout and business-rule constants.
//!
//! Input layout (columns B-I, header rows 1-14, data starting at row 15):
//!     B=STT  C=So hoa don  D=Ngay lap hoa don  E=Ten nguoi ban
//!     F=Ma so thue nguoi ban  G=Doanh so mua chua co thue  H=Thue GTGT  I=Ghi chu

/// Rows 1-14 are title/header; data starts at row 15.
pub const HEADER_ROWS: usize = 14;
pub const EXCEL_ROW_OFFSET: usize = HEADER_ROWS + 1;

/// Valid Vietnamese tax-code shapes seen in this dataset (digits only, dash/space
/// separators ignored): 10 = organization MST, 12 = individual's CCCD used as MST,
/// 13 = organization MST + 3-digit dependent-unit suffix. Don't tighten this to a
/// stricter regex without re-checking the real data distribution first -- an
/// earlier stricter pattern false-flagged 949 of ~18.7k rows that were valid.
pub const VALID_TAX_CODE_LENGTHS: [usize; 3] = [10, 12, 13];

/// Cash-payment / VAT-deduction threshold in VND, by invoice date. Per
/// "docs/vietnam-e-invoice-knowledge-base.md" section 3.2, the 5,000,000 rule
/// took effect 2025-07-01 (Luat Thue GTGT 2024 / Nghi dinh 181/2025/ND-CP);
/// invoices dated before that still fall under the old 20,000,000 rule. Ordered
/// oldest-first; the last entry with effective_from <= invoice date applies.
/// Update this list (not a single constant) if the law changes again -- the
/// knowledge base itself warns against hard-coding one flat threshold.
pub const THRESHOLD_SCHEDULE: &[(Option<&str>, i64)] =
    &[(None, 20_000_000), (Some("2025-07-01"), 5_000_000)];

/// Used only when a flat --threshold override is passed on the CLI.
pub const DEFAULT_THRESHOLD: i64 = 5_000_000;

/// Default report filename, written next to the input file unless overridden.
pub const DEFAULT_OUTPUT_FILENAME: &str = "Kết quả kiểm tra hóa đơn.xlsx";

pub const CATEGORY_DU_LIEU_LOI: &str = "Du_lieu_loi";
pub const CATEGORY_TRUNG_LAP: &str = "Trung_lap";
pub const CATEGORY_VUOT_NGUONG: &str = "Vuot_nguong";
