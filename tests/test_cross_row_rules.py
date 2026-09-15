import unittest
from datetime import datetime

import pandas as pd

from invoice_validator.config import DerivedField
from invoice_validator.cross_row_rules import DuplicateRowsRule, OverThresholdRule
from invoice_validator.helpers import threshold_for_date

DAY = pd.Timestamp(datetime(2025, 1, 15))


def row(**overrides):
    base = {"excel_row": 15, "stt": 1, "so_hoa_don": "00000123", "ngay_lap_hoa_don": DAY, "ten_nguoi_ban": "Cong ty TNHH Vi Du", "ma_so_thue": "0100150619", "doanh_so_mua_chua_thue": 1_000_000, "thue_gtgt": 100_000, "ghi_chu": ""}
    return {**base, **overrides}


class TestDuplicateRowsRule(unittest.TestCase):
    def test_flags_exact_duplicate_rows(self):
        violations = DuplicateRowsRule().check(pd.DataFrame([row(excel_row=15), row(excel_row=16), row(excel_row=17, so_hoa_don="00000999")]))
        self.assertEqual(sorted(v.excel_row for v in violations), [15, 16])
        self.assertEqual({v.extra[DerivedField.DUPLICATE_GROUP.value] for v in violations}, {1})

    def test_no_duplicates_when_all_rows_distinct(self):
        self.assertEqual(DuplicateRowsRule().check(pd.DataFrame([row(excel_row=15), row(excel_row=16, so_hoa_don="00000999")])), [])


class TestOverThresholdRule(unittest.TestCase):
    def test_sums_same_seller_same_day(self):
        flagged = OverThresholdRule(5_000_000).check(pd.DataFrame([row(excel_row=15, doanh_so_mua_chua_thue=3_000_000, thue_gtgt=0), row(excel_row=16, doanh_so_mua_chua_thue=2_500_000, thue_gtgt=0)]))
        self.assertEqual(sorted(v.excel_row for v in flagged), [15, 16])
        self.assertTrue(all(v.extra[DerivedField.DAILY_TOTAL.value] == 5_500_000 for v in flagged))

    def test_below_threshold_not_flagged(self):
        self.assertEqual(OverThresholdRule(5_000_000).check(pd.DataFrame([row(excel_row=15, doanh_so_mua_chua_thue=1_000_000, thue_gtgt=0), row(excel_row=16, doanh_so_mua_chua_thue=1_000_000, thue_gtgt=0)])), [])

    def test_different_sellers_not_combined(self):
        self.assertEqual(OverThresholdRule(5_000_000).check(pd.DataFrame([row(excel_row=15, doanh_so_mua_chua_thue=3_000_000, thue_gtgt=0), row(excel_row=16, ma_so_thue="0309944010", doanh_so_mua_chua_thue=3_000_000, thue_gtgt=0)])), [])

    def test_default_uses_schedule_20tr_before_2025_07_01(self):
        self.assertEqual(OverThresholdRule().check(pd.DataFrame([row(excel_row=15, doanh_so_mua_chua_thue=6_000_000, thue_gtgt=0), row(excel_row=16, doanh_so_mua_chua_thue=4_000_000, thue_gtgt=0)])), [])

    def test_default_uses_schedule_5tr_from_2025_07_01(self):
        flagged = OverThresholdRule().check(pd.DataFrame([row(excel_row=15, ngay_lap_hoa_don=pd.Timestamp(2025, 7, 1), doanh_so_mua_chua_thue=6_000_000, thue_gtgt=0), row(excel_row=16, ngay_lap_hoa_don=pd.Timestamp(2025, 7, 1), doanh_so_mua_chua_thue=4_000_000, thue_gtgt=0)]))
        self.assertEqual(sorted(v.excel_row for v in flagged), [15, 16])
        self.assertTrue(all(v.extra[DerivedField.APPLIED_THRESHOLD.value] == 5_000_000 for v in flagged))

    def test_flat_override_ignores_schedule(self):
        flagged = OverThresholdRule(5_000_000).check(pd.DataFrame([row(excel_row=15, doanh_so_mua_chua_thue=6_000_000, thue_gtgt=0), row(excel_row=16, doanh_so_mua_chua_thue=4_000_000, thue_gtgt=0)]))
        self.assertEqual(sorted(v.excel_row for v in flagged), [15, 16])


class TestThresholdForDate(unittest.TestCase):
    def test_before_effective_date_is_old_threshold(self): self.assertEqual(threshold_for_date(pd.Timestamp(2025, 6, 30)), 20_000_000)
    def test_on_and_after_effective_date_is_new_threshold(self):
        self.assertEqual(threshold_for_date(pd.Timestamp(2025, 7, 1)), 5_000_000)
        self.assertEqual(threshold_for_date(pd.Timestamp(2025, 12, 31)), 5_000_000)
