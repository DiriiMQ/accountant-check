"""Small pure functions shared across rule modules (no Rule/Violation/config-data coupling)."""

import re

import pandas as pd

from .config import THRESHOLD_SCHEDULE


def is_blank(value) -> bool:
    return pd.isna(value) or str(value).strip() == ""


def tax_code_digits(value) -> str:
    if is_blank(value):
        return ""
    return re.sub(r"[^0-9]", "", str(value))


def threshold_for_date(invoice_date: pd.Timestamp) -> int:
    """Threshold in effect on a given invoice date, per config.THRESHOLD_SCHEDULE."""
    applicable = THRESHOLD_SCHEDULE[0][1]
    for effective_from, amount in THRESHOLD_SCHEDULE:
        if effective_from is None or invoice_date >= pd.Timestamp(effective_from):
            applicable = amount
        else:
            break
    return applicable
