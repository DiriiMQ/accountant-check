"""Cross-row business rules: duplicates and same-seller/same-day thresholds."""

import pandas as pd

from .config import DUPLICATE_KEY
from .helpers import is_blank, threshold_for_date
from .violations import Rule, Violation


class DuplicateRowsRule(Rule):
    code = "duplicate_rows"
    category = "Trung_lap"
    message = "Trung lap"

    def check(self, df: pd.DataFrame) -> list[Violation]:
        duplicates = df.loc[df.duplicated(subset=DUPLICATE_KEY, keep=False)].copy()
        duplicates["nhom_trung"] = duplicates.groupby(DUPLICATE_KEY, dropna=False).ngroup() + 1
        return [
            Violation(int(row.excel_row), self.code, self.category, self.message, {"nhom_trung": int(row.nhom_trung)})
            for row in duplicates.sort_values(["nhom_trung", "excel_row"])[["excel_row", "nhom_trung"]].itertuples(index=False)
        ]

class OverThresholdRule(Rule):
    code = "over_threshold"
    category = "Vuot_nguong"
    message = "Vuot nguong"

    def __init__(self, threshold: int | None = None):
        self.threshold = threshold

    def check(self, df: pd.DataFrame) -> list[Violation]:
        valid = df.loc[
            ~df["ma_so_thue"].map(is_blank)
            & df["ngay_lap_hoa_don"].map(lambda value: isinstance(value, pd.Timestamp))
        ].copy()
        valid["tong_tien"] = pd.to_numeric(valid["doanh_so_mua_chua_thue"], errors="coerce").fillna(0) + pd.to_numeric(
            valid["thue_gtgt"], errors="coerce"
        ).fillna(0)
        valid["tong_theo_ngay"] = valid.groupby(["ma_so_thue", "ngay_lap_hoa_don"])["tong_tien"].transform("sum")
        valid["nguong_ap_dung"] = self.threshold if self.threshold is not None else valid["ngay_lap_hoa_don"].map(threshold_for_date)
        flagged = valid.loc[valid["tong_theo_ngay"] >= valid["nguong_ap_dung"]]
        return [
            Violation(
                int(row.excel_row), self.code, self.category, self.message,
                {"tong_theo_ngay": row.tong_theo_ngay, "nguong_ap_dung": row.nguong_ap_dung},
            )
            for row in flagged.sort_values(["ma_so_thue", "ngay_lap_hoa_don", "excel_row"])[
                ["excel_row", "tong_theo_ngay", "nguong_ap_dung"]
            ].itertuples(index=False)
        ]
