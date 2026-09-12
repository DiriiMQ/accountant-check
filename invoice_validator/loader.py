"""Read the invoice list sheet into a flat DataFrame."""

from pathlib import Path

import pandas as pd

from .config import COLUMNS, HEADER_ROWS

# so_hoa_don and ma_so_thue must stay text: if pandas infers a column as
# uniformly numeric it silently drops leading zeros (e.g. "00002224" -> 2224).
# The real file only avoids this by accident (a few non-numeric values like
# "976." force object dtype) -- don't rely on that, force it explicitly.
TEXT_COLUMNS = {COLUMNS.index("so_hoa_don"): str, COLUMNS.index("ma_so_thue"): str}


def load_invoices(path: Path) -> pd.DataFrame:
    df = pd.read_excel(path, header=None, skiprows=HEADER_ROWS, usecols="B:I", names=COLUMNS, dtype=TEXT_COLUMNS)
    df["excel_row"] = df.index + HEADER_ROWS + 1
    df = df.dropna(how="all", subset=COLUMNS)
    return df[["excel_row"] + COLUMNS].reset_index(drop=True)
