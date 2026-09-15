"""Write rule violations out to a report workbook."""

from pathlib import Path

import pandas as pd

from .config import Column, ViolationCategory
from .violations import Violation


def write_report(
    output: Path,
    total_rows: int,
    df: pd.DataFrame,
    violations: list[Violation],
    threshold: int | None,
) -> None:
    """Write one worksheet per violation category plus the summary sheet."""
    threshold_label = f"{threshold:,} VND (flat override)" if threshold is not None else "theo ngay hieu luc"
    by_category: dict[str, list[Violation]] = {}
    for violation in violations:
        by_category.setdefault(violation.category, []).append(violation)

    with pd.ExcelWriter(output, engine="openpyxl") as writer:
        for category, category_violations in by_category.items():
            _category_frame(df, category, category_violations).to_excel(writer, sheet_name=category, index=False)

        pd.DataFrame(
            {
                "Loai kiem tra": [
                    "Tong so dong",
                    "Du lieu loi",
                    "Trung lap",
                    f"Vuot nguong ({threshold_label})",
                ],
                "So dong": [
                    total_rows,
                    _row_count(by_category.get(ViolationCategory.DATA_ERROR.value, [])),
                    _row_count(by_category.get(ViolationCategory.DUPLICATE.value, [])),
                    _row_count(by_category.get(ViolationCategory.OVER_THRESHOLD.value, [])),
                ],
            }
        ).to_excel(writer, sheet_name="Tong_hop", index=False)


def _row_count(violations: list[Violation]) -> int:
    return len({violation.excel_row for violation in violations})


def _category_frame(df: pd.DataFrame, category: str, violations: list[Violation]) -> pd.DataFrame:
    """Build a category worksheet, retaining the established report layout."""
    messages = _messages_by_row(violations)
    rows = df.loc[df["excel_row"].isin(messages)].copy()

    if category == ViolationCategory.DATA_ERROR.value:
        rows["loi"] = rows["excel_row"].map(messages)
        return rows

    extras = pd.DataFrame(
        [{"excel_row": violation.excel_row, **violation.extra} for violation in violations]
    ).drop_duplicates(subset="excel_row")
    rows = rows.merge(extras, on="excel_row", how="inner")

    if category == ViolationCategory.DUPLICATE.value:
        return rows.sort_values(["nhom_trung", "excel_row"])

    if category == ViolationCategory.OVER_THRESHOLD.value:
        # This derived column existed in the pre-registry report and remains useful
        # when reconciling a same-seller/same-day total.
        rows["tong_tien"] = pd.to_numeric(rows[Column.AMOUNT_EX_VAT.value], errors="coerce").fillna(0) + pd.to_numeric(
            rows[Column.VAT_AMOUNT.value], errors="coerce"
        ).fillna(0)
        columns = [column for column in df.columns] + ["tong_tien", "tong_theo_ngay", "nguong_ap_dung"]
        return rows.loc[:, columns].sort_values([Column.TAX_CODE.value, Column.INVOICE_DATE.value, "excel_row"])

    # New categories need no report.py changes: provide the flagged rows and
    # their combined messages as a generally useful default layout.
    rows["loi"] = rows["excel_row"].map(messages)
    return rows


def _messages_by_row(violations: list[Violation]) -> dict[int, str]:
    messages: dict[int, list[str]] = {}
    for violation in violations:
        messages.setdefault(violation.excel_row, []).append(violation.message)
    return {excel_row: " | ".join(row_messages) for excel_row, row_messages in messages.items()}
