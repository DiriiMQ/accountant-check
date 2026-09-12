import unittest

import pandas as pd

from invoice_validator.engine import run_rules
from invoice_validator.field_rules import MissingFieldRule


class TestRunRules(unittest.TestCase):
    def test_concatenates_rule_results_in_registry_order(self):
        df = pd.DataFrame([{"excel_row": 15, "first": "", "second": ""}])
        rules = [MissingFieldRule("first", "first error"), MissingFieldRule("second", "second error")]
        self.assertEqual([v.message for v in run_rules(df, rules)], ["first error", "second error"])
