# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

This is **not a source-code repository** — there is no buildable/lintable/testable source anywhere in
this tree (no `.py`/`.js`/`.ts`/etc. files exist). It is a small working folder around one compiled
Windows tool plus its input/output data:

- `final/run.exe` — a compiled Windows console executable (PE32+, no source present in this repo) that
  processes an invoice list and flags rows that violate cash-payment/VAT-deduction thresholds.
- `final/README.pdf` — usage instructions for `run.exe`, authored by Phan Minh Quang (matches this
  workspace's user).
- `final/sample.xlsx` — a sample input showing the exact layout `run.exe` expects.
- `final.zip` — a zip bundle containing the same `final/` folder (README.pdf, run.exe, sample.xlsx).
- `Danh sach hoa don dau vao 2025.xlsx` — a real 2025 input invoice list (~3,033 rows) in the same layout.
- `docs/vietnam-e-invoice-knowledge-base.md` — reference notes on Vietnamese e-invoice numbering and the
  cash-payment/VAT-deduction threshold rules.

There is no build system, package manifest, or test suite to run.

## How `run.exe` is used (per `final/README.pdf`)

1. Rename the input Excel file to `data.xlsx` and place it in the same folder as `run.exe`.
2. Double-click `run.exe` (icon: a cat).
3. `data.xlsx` must have its first data cell at **B15**, matching the layout of `sample.xlsx`.
4. Output: two new files are generated next to the input —
   - `duplicated_bills.xlsx` — rows that are duplicates.
   - `over_maxcost.xlsx` — rows/groups whose total exceeds the **20 million VND** threshold.
5. Note: leading zeros in invoice numbers are stripped by the tool.

Since `run.exe` is a compiled binary with no accompanying source in this repo, its exact logic can only
be inferred from the README and the header comments embedded in the spreadsheets themselves (see below)
— it cannot be read or modified directly.

## Input spreadsheet layout (`sample.xlsx`, `Danh sach hoa don dau vao 2025.xlsx`)

Single sheet, header block in rows 2–14, data starting at row 15, columns B–I:

| Col | Tag | Field |
|---|---|---|
| B | `[1]` | STT (row number) |
| C | `[2]` | Số hoá đơn (invoice number) |
| D | `[3]` | Ngày, tháng, năm lập hóa đơn (invoice date) |
| E | `[4]` | Tên người bán (seller name) |
| F | `[5]` | Mã số thuế người bán (seller tax code) |
| G | `[6]` | Doanh số mua chưa có thuế (purchase amount excl. VAT) |
| H | `[7]` | Thuế GTGT đủ điều kiện khấu trừ thuế (deductible VAT) |
| I | `[8]` | Ghi chú (note) |

`Danh sach hoa don dau vao 2025.xlsx` holds 18,695 data rows in total; the STT column (col B) resets to
`1` at the start of each of 12 monthly batches concatenated one after another from row 15 onward (no
header rows in between), so STT is not a running total across the whole file.

Rows 2 and 4 of the sheet embed the business rules the tool implements, in Vietnamese:
- Row 2: if the sum of columns 6+7 for one seller (col 5) on one day (col 3) reaches ≥20,000,000 VND,
  exclude/flag it.
- Row 4: if columns 2,3,4,5,6,7 are identical across rows, treat as duplicate and flag it.

## Important: threshold discrepancy to be aware of

`docs/vietnam-e-invoice-knowledge-base.md` documents that the **20,000,000 VND** threshold used by the
spreadsheet/tool is the **old rule**. Per the knowledge base, current regulation
(Luật Thuế GTGT 2024; Nghị định 181/2025/NĐ-CP) lowered the non-cash-payment threshold for VAT input
deduction to **5,000,000 VND (VAT-inclusive)**, and it must be aggregated per seller per day, and applies
differently to installment/deferred-payment cases. If asked to update or reimplement this tool's logic,
flag this gap explicitly rather than assuming the spreadsheet's hardcoded 20tr rule is current.

## `validate_invoices.py`

A standalone Python package re-implementation (pandas + openpyxl, no build step) that validates an input
file in the layout above and writes flagged rows to a separate report workbook instead of modifying the input.

```
python3 -m invoice_validator <input.xlsx> [-o report.xlsx] [--threshold 5000000]
```

By default, the threshold is selected by invoice date: 20,000,000 VND before 2025-07-01 and 5,000,000 VND
from that date; `--threshold` applies one flat override. Output workbook sheets:
`Du_lieu_loi` (missing/malformed fields — invoice number, date, seller name, tax code, negative or
non-numeric amounts), `Trung_lap` (duplicate rows per the col C–H rule), `Vuot_nguong` (same seller/day
groups at or above the threshold, with the group total), `Tong_hop` (counts).

Tax-code validity is length-based, not a strict regex: valid shapes found in this dataset are 10 digits
(org MST), 12 digits (individual's CCCD used as MST), 13 digits (MST + 3-digit dependent-unit suffix),
with dash/space separators ignored. Don't reintroduce a stricter regex without re-checking the real data
distribution first — an earlier stricter pattern false-flagged 949 of ~18.7k rows that were actually valid.

The extensible rule registry is split across `invoice_validator/violations.py` (`Violation` and the `Rule`
interface), `helpers.py` (pure functions shared across rules: `is_blank`, `tax_code_digits`,
`threshold_for_date` — no `Rule`/DataFrame-row coupling), `field_rules.py` (field-level rules),
`cross_row_rules.py` (duplicate and threshold rules), and `engine.py` (the default registry and runner).
`config.py` stays pure data (column layout, thresholds, etc.) with no functions in it.
`report.py` builds one output sheet per rule category, while
`cli.py` loads input, runs the registry, and writes the report. Add a rule by implementing `Rule` and adding
it to the registry; a new category automatically receives its own worksheet.

## Desktop app and CI

`invoice_validator/gui.py` is a minimal tkinter front-end (file-picker dialog -> runs the same
`engine.run_rules`/`report.write_report` pipeline as the CLI -> summary message box) — chosen over a web
UI because it needs no server/browser, is stdlib-only, and packages cleanly into a single exe. The
root-level `run_gui.py` is the PyInstaller entry point.

`.github/workflows/ci.yml` runs the test suite on every push/PR to `main`. `.github/workflows/release.yml`
runs on `vX.Y.Z` tags: reruns tests, then builds `accountant-check.exe` on `windows-latest` via PyInstaller
(`--onefile --windowed run_gui.py`) and publishes it as a GitHub Release asset via `gh release create`
(no third-party release action). If `gui.py`'s dependencies on `engine`/`report`/`loader` change shape,
update it alongside `cli.py` — they intentionally duplicate a small amount of orchestration rather than
share a return-type contract with `cli.run()`.
