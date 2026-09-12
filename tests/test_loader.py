import tempfile
import unittest
from datetime import datetime
from pathlib import Path

import openpyxl

from invoice_validator.loader import load_invoices


def build_sample_workbook(path: Path) -> None:
    wb = openpyxl.Workbook()
    ws = wb.active
    # Rows 1-14 are header/notes in the real file; content doesn't matter for loading.
    for _ in range(14):
        ws.append([])
    ws.append([None, 1, "00000123", datetime(2025, 1, 15), "Cong ty A", "0100150619", 1_000_000, 100_000, "Note1"])
    ws.append([None, 2, "00000124", datetime(2025, 1, 16), "Cong ty B", "0309944010", 500_000, 0, "Note2"])
    ws.append([None])  # fully blank trailing row, should be dropped
    wb.save(path)


class TestLoadInvoices(unittest.TestCase):
    def test_reads_data_starting_at_row_15(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.xlsx"
            build_sample_workbook(path)

            df = load_invoices(path)

            self.assertEqual(len(df), 2)
            self.assertEqual(df.iloc[0]["excel_row"], 15)
            self.assertEqual(df.iloc[0]["so_hoa_don"], "00000123")
            self.assertEqual(df.iloc[1]["excel_row"], 16)
            self.assertEqual(df.iloc[1]["ten_nguoi_ban"], "Cong ty B")

    def test_blank_trailing_rows_are_dropped(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.xlsx"
            build_sample_workbook(path)

            df = load_invoices(path)

            self.assertNotIn(17, df["excel_row"].tolist())


if __name__ == "__main__":
    unittest.main()
