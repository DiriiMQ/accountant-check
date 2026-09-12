"""Common interfaces for invoice-validation rules and their results."""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field

import pandas as pd


@dataclass
class Violation:
    excel_row: int
    rule_code: str
    category: str
    message: str
    extra: dict = field(default_factory=dict)


class Rule(ABC):
    """A validation rule that returns one violation for every flagged row."""

    code: str
    category: str

    @abstractmethod
    def check(self, df: pd.DataFrame) -> list[Violation]:
        """Check *df* and return the violations found."""
