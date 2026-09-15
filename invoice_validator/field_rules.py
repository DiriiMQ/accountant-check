"""Row-level data-quality validation rules."""

import pandas as pd

from .config import Column, VALID_TAX_CODE_LENGTHS, ViolationCategory
from .helpers import tax_code_digits
from .violations import Rule, Violation


def _blank_mask(values: pd.Series) -> pd.Series:
    return values.isna() | values.astype(str).str.strip().eq("")


class MissingFieldRule(Rule):
    code = "missing_field"
    category = ViolationCategory.DATA_ERROR.value

    def __init__(self, field: str, message: str):
        self.field = field
        self.message = message

    def check(self, df: pd.DataFrame) -> list[Violation]:
        return [
            Violation(int(excel_row), self.code, self.category, self.message)
            for excel_row in df.loc[_blank_mask(df[self.field]), "excel_row"]
        ]


class InvalidDateRule(Rule):
    code = "invalid_date"
    category = ViolationCategory.DATA_ERROR.value
    message = "Ngay lap hoa don khong hop le"

    def check(self, df: pd.DataFrame) -> list[Violation]:
        mask = ~df[Column.INVOICE_DATE.value].map(lambda value: isinstance(value, pd.Timestamp))
        return [Violation(int(excel_row), self.code, self.category, self.message) for excel_row in df.loc[mask, "excel_row"]]


class InvalidTaxCodeRule(Rule):
    code = "invalid_tax_code"
    category = ViolationCategory.DATA_ERROR.value
    message = "Ma so thue khong dung dinh dang"

    def check(self, df: pd.DataFrame) -> list[Violation]:
        digit_lengths = df[Column.TAX_CODE.value].map(tax_code_digits).str.len()
        return [
            Violation(int(excel_row), self.code, self.category, self.message)
            for excel_row in df.loc[~digit_lengths.isin(VALID_TAX_CODE_LENGTHS), "excel_row"]
        ]


class AmountRule(Rule):
    code = "invalid_amount"
    category = ViolationCategory.DATA_ERROR.value

    def __init__(self, field: str, label: str):
        self.field = field
        self.label = label

    def check(self, df: pd.DataFrame) -> list[Violation]:
        numeric = pd.to_numeric(df[self.field], errors="coerce")
        missing_or_non_numeric = numeric.isna()
        negative = numeric.lt(0) & ~missing_or_non_numeric
        violations = [
            Violation(int(excel_row), self.code, self.category, f"{self.label}: thieu hoac khong phai so")
            for excel_row in df.loc[missing_or_non_numeric, "excel_row"]
        ]
        violations.extend(
            Violation(int(excel_row), self.code, self.category, f"{self.label}: gia tri am")
            for excel_row in df.loc[negative, "excel_row"]
        )
        return violations
