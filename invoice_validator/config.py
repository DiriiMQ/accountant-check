"""Input layout and business-rule constants.

Input layout (columns B-I, header rows 1-14, data starting at row 15):
    B=STT  C=So hoa don  D=Ngay lap hoa don  E=Ten nguoi ban
    F=Ma so thue nguoi ban  G=Doanh so mua chua co thue  H=Thue GTGT  I=Ghi chu
"""

from enum import Enum


class Column(str, Enum):
    STT = "stt"
    INVOICE_NUMBER = "so_hoa_don"
    INVOICE_DATE = "ngay_lap_hoa_don"
    SELLER_NAME = "ten_nguoi_ban"
    TAX_CODE = "ma_so_thue"
    AMOUNT_EX_VAT = "doanh_so_mua_chua_thue"
    VAT_AMOUNT = "thue_gtgt"
    NOTE = "ghi_chu"


class ViolationCategory(str, Enum):
    DATA_ERROR = "Du_lieu_loi"
    DUPLICATE = "Trung_lap"
    OVER_THRESHOLD = "Vuot_nguong"


COLUMNS = [
    Column.STT.value,
    Column.INVOICE_NUMBER.value,
    Column.INVOICE_DATE.value,
    Column.SELLER_NAME.value,
    Column.TAX_CODE.value,
    Column.AMOUNT_EX_VAT.value,
    Column.VAT_AMOUNT.value,
    Column.NOTE.value,
]

HEADER_ROWS = 14  # rows 1-14 are title/header; data starts at row 15
EXCEL_ROW_OFFSET = HEADER_ROWS + 1

# Columns C-H: rows identical across all of these count as duplicates.
DUPLICATE_KEY = [
    Column.INVOICE_NUMBER.value,
    Column.INVOICE_DATE.value,
    Column.SELLER_NAME.value,
    Column.TAX_CODE.value,
    Column.AMOUNT_EX_VAT.value,
    Column.VAT_AMOUNT.value,
]

REPORT_CATEGORIES = (
    ViolationCategory.DATA_ERROR.value,
    ViolationCategory.DUPLICATE.value,
    ViolationCategory.OVER_THRESHOLD.value,
)

# Valid Vietnamese tax-code shapes seen in this dataset (digits only, dash/space
# separators ignored): 10 = organization MST, 12 = individual's CCCD used as MST,
# 13 = organization MST + 3-digit dependent-unit suffix. Don't tighten this to a
# stricter regex without re-checking the real data distribution first -- an
# earlier stricter pattern false-flagged 949 of ~18.7k rows that were valid.
VALID_TAX_CODE_LENGTHS = {10, 12, 13}

# Cash-payment / VAT-deduction threshold in VND, by invoice date. Per
# "docs/vietnam-e-invoice-knowledge-base.md" section 3.2, the 5,000,000 rule
# took effect 2025-07-01 (Luat Thue GTGT 2024 / Nghi dinh 181/2025/ND-CP);
# invoices dated before that still fall under the old 20,000,000 rule. Ordered
# oldest-first; the last entry with effective_from <= invoice date applies.
# Update this list (not a single constant) if the law changes again -- the
# knowledge base itself warns against hard-coding one flat threshold.
THRESHOLD_SCHEDULE = [
    # (effective_from, threshold_vnd)
    (None, 20_000_000),
    ("2025-07-01", 5_000_000),
]

# Used only when a flat --threshold override is passed on the CLI.
DEFAULT_THRESHOLD = 5_000_000

# Default report filename, written next to the input file unless overridden.
DEFAULT_OUTPUT_FILENAME = "Kết quả kiểm tra hóa đơn.xlsx"
