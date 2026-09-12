"""Rule registry and execution for invoice validation."""

import pandas as pd

from .cross_row_rules import DuplicateRowsRule, OverThresholdRule
from .field_rules import AmountRule, InvalidDateRule, InvalidTaxCodeRule, MissingFieldRule
from .violations import Rule, Violation


DEFAULT_RULES: list[Rule] = [
    MissingFieldRule("so_hoa_don", "Thieu so hoa don"),
    InvalidDateRule(),
    MissingFieldRule("ten_nguoi_ban", "Thieu ten nguoi ban"),
    InvalidTaxCodeRule(),
    AmountRule("doanh_so_mua_chua_thue", "Doanh so mua chua co thue"),
    AmountRule("thue_gtgt", "Thue GTGT"),
    DuplicateRowsRule(),
    OverThresholdRule(),
]


def run_rules(df: pd.DataFrame, rules: list[Rule]) -> list[Violation]:
    """Run each registered rule, retaining registry order in the result."""
    violations: list[Violation] = []
    for rule in rules:
        violations.extend(rule.check(df))
    return violations
