import unittest
from datetime import datetime

import pandas as pd

from invoice_validator.field_rules import AmountRule, InvalidDateRule, InvalidTaxCodeRule, MissingFieldRule
from invoice_validator.helpers import tax_code_digits

VALID_ROW = {
    "excel_row": 15, "stt": 1, "so_hoa_don": "00000123", "ngay_lap_hoa_don": pd.Timestamp(datetime(2025, 1, 15)),
    "ten_nguoi_ban": "Cong ty TNHH Vi Du", "ma_so_thue": "0100150619", "doanh_so_mua_chua_thue": 1_000_000,
    "thue_gtgt": 100_000, "ghi_chu": "",
}


def frame(**overrides):
    return pd.DataFrame([{**VALID_ROW, **overrides}])


class TestTaxCodeDigits(unittest.TestCase):
    def test_strips_dash_and_space(self):
        self.assertEqual(tax_code_digits("0100150619-047"), "0100150619047")
        self.assertEqual(tax_code_digits("0106869738 -034"), "0106869738034")

    def test_blank_is_empty(self):
        self.assertEqual(tax_code_digits(None), "")
        self.assertEqual(tax_code_digits(""), "")
        self.assertEqual(tax_code_digits(float("nan")), "")


class TestFieldRules(unittest.TestCase):
    def test_valid_row_has_no_errors(self):
        rules = [MissingFieldRule("so_hoa_don", "Thieu so hoa don"), InvalidDateRule(), MissingFieldRule("ten_nguoi_ban", "Thieu ten nguoi ban"), InvalidTaxCodeRule(), AmountRule("doanh_so_mua_chua_thue", "Doanh so mua chua co thue"), AmountRule("thue_gtgt", "Thue GTGT")]
        self.assertEqual([rule.check(frame()) for rule in rules], [[], [], [], [], [], []])

    def test_missing_invoice_number(self):
        self.assertEqual(MissingFieldRule("so_hoa_don", "Thieu so hoa don").check(frame(so_hoa_don=""))[0].message, "Thieu so hoa don")

    def test_invalid_date(self):
        self.assertEqual(InvalidDateRule().check(frame(ngay_lap_hoa_don="31/1/25"))[0].message, "Ngay lap hoa don khong hop le")

    def test_missing_seller_name(self):
        self.assertEqual(MissingFieldRule("ten_nguoi_ban", "Thieu ten nguoi ban").check(frame(ten_nguoi_ban=None))[0].message, "Thieu ten nguoi ban")

    def test_tax_code_lengths(self):
        rule = InvalidTaxCodeRule()
        for valid in ("0100150619", "056080007214", "0100150619047"):
            self.assertEqual(rule.check(frame(ma_so_thue=valid)), [])
        self.assertTrue(rule.check(frame(ma_so_thue="42002400380")))
        self.assertTrue(rule.check(frame(ma_so_thue=None)))

    def test_negative_amount(self):
        self.assertIn("gia tri am", AmountRule("doanh_so_mua_chua_thue", "Doanh so mua chua co thue").check(frame(doanh_so_mua_chua_thue=-500))[0].message)

    def test_non_numeric_amount(self):
        self.assertIn("khong phai so", AmountRule("thue_gtgt", "Thue GTGT").check(frame(thue_gtgt="khong phai so"))[0].message)
