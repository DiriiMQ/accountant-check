"""Cross-row business rules: duplicates and same-seller/same-day thresholds."""

import pandas as pd

from .config import Column, DUPLICATE_KEY, DerivedField, ViolationCategory
from .helpers import is_blank, threshold_for_date
from .violations import Rule, Violation


class DuplicateRowsRule(Rule):
    code = "duplicate_rows"
    category = ViolationCategory.DUPLICATE.value
    message = "Trung lap"

    def check(self, df: pd.DataFrame) -> list[Violation]:
        duplicates = df.loc[df.duplicated(subset=DUPLICATE_KEY, keep=False)].copy()
        duplicates[DerivedField.DUPLICATE_GROUP.value] = duplicates.groupby(DUPLICATE_KEY, dropna=False).ngroup() + 1
        return [
            Violation(
                int(row.excel_row),
                self.code,
                self.category,
                self.message,
                {DerivedField.DUPLICATE_GROUP.value: int(row.nhom_trung)},
            )
            for row in duplicates.sort_values([DerivedField.DUPLICATE_GROUP.value, "excel_row"])[
                ["excel_row", DerivedField.DUPLICATE_GROUP.value]
            ].itertuples(index=False)
        ]

class OverThresholdRule(Rule):
    code = "over_threshold"
    category = ViolationCategory.OVER_THRESHOLD.value
    message = "Vuot nguong"

    def __init__(self, threshold: int | None = None):
        self.threshold = threshold

    def check(self, df: pd.DataFrame) -> list[Violation]:
        valid = df.loc[
            ~df[Column.TAX_CODE.value].map(is_blank)
            & df[Column.INVOICE_DATE.value].map(lambda value: isinstance(value, pd.Timestamp))
        ].copy()
        valid[DerivedField.ROW_TOTAL.value] = pd.to_numeric(valid[Column.AMOUNT_EX_VAT.value], errors="coerce").fillna(0) + pd.to_numeric(
            valid[Column.VAT_AMOUNT.value], errors="coerce"
        ).fillna(0)
        valid[DerivedField.DAILY_TOTAL.value] = valid.groupby([Column.TAX_CODE.value, Column.INVOICE_DATE.value])[
            DerivedField.ROW_TOTAL.value
        ].transform("sum")
        valid[DerivedField.APPLIED_THRESHOLD.value] = (
            self.threshold if self.threshold is not None else valid[Column.INVOICE_DATE.value].map(threshold_for_date)
        )
        flagged = valid.loc[valid[DerivedField.DAILY_TOTAL.value] >= valid[DerivedField.APPLIED_THRESHOLD.value]]
        return [
            Violation(
                int(row.excel_row), self.code, self.category, self.message,
                {
                    DerivedField.DAILY_TOTAL.value: row.tong_theo_ngay,
                    DerivedField.APPLIED_THRESHOLD.value: row.nguong_ap_dung,
                },
            )
            for row in flagged.sort_values([Column.TAX_CODE.value, Column.INVOICE_DATE.value, "excel_row"])[
                ["excel_row", DerivedField.DAILY_TOTAL.value, DerivedField.APPLIED_THRESHOLD.value]
            ].itertuples(index=False)
        ]
