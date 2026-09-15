"""Rule registry and execution for invoice validation."""

import pandas as pd

from .config import Column
from .cross_row_rules import DuplicateRowsRule, OverThresholdRule
from .field_rules import AmountRule, InvalidDateRule, InvalidTaxCodeRule, MissingFieldRule
from .violations import Rule, Violation


DEFAULT_RULES: list[Rule] = [
    MissingFieldRule(Column.INVOICE_NUMBER.value, "Thieu so hoa don"),
    InvalidDateRule(),
    MissingFieldRule(Column.SELLER_NAME.value, "Thieu ten nguoi ban"),
    InvalidTaxCodeRule(),
    AmountRule(Column.AMOUNT_EX_VAT.value, "Doanh so mua chua co thue"),
    AmountRule(Column.VAT_AMOUNT.value, "Thue GTGT"),
    DuplicateRowsRule(),
    OverThresholdRule(),
]


def run_rules(df: pd.DataFrame, rules: list[Rule]) -> list[Violation]:
    """Run each registered rule, retaining registry order in the result."""
    violations: list[Violation] = []
    for rule in rules:
        violations.extend(rule.check(df))
    return violations
