# accountant-check

Validates a Vietnamese invoice list (Excel) for data-quality issues and cash-payment/VAT-deduction
threshold violations, writing flagged rows to a separate report workbook. The input file is never
modified.

## What it checks

- **Missing/malformed fields** — invoice number, date, seller name, tax code (format), amounts
- **Duplicate rows** — same invoice number, date, seller, tax code, and amounts
- **Same-seller/same-day totals over the cash-payment threshold** — date-aware: 20,000,000 VND before
  2025-07-01, 5,000,000 VND from that date on, per current regulation (see
  `docs/vietnam-e-invoice-knowledge-base.md`)

## Requirements

Python 3.10+, pandas, openpyxl.

```
pip install -r requirements.txt
```

## Usage

```
python3 -m invoice_validator <input.xlsx> [-o report.xlsx] [--threshold 5000000]
```

Input layout: a single sheet, header rows 1-14, data starting at row 15, columns B-I (see `CLAUDE.md`
for the exact column mapping).

Output: a report workbook with one sheet per violation category (`Du_lieu_loi`, `Trung_lap`,
`Vuot_nguong`) plus a `Tong_hop` summary sheet.

## Project structure

- `invoice_validator/` — the package. Rule-registry design: `helpers.py` (pure functions shared across
  rules), `field_rules.py` / `cross_row_rules.py` (rule classes), `engine.py` (default registry +
  runner), `report.py`, `cli.py`. Add a new check by implementing `Rule` and adding it to the registry —
  no existing rule code needs editing.
- `tests/` — `unittest` test suite (stdlib only, no pytest dependency)
- `docs/` — reference notes on the Vietnamese e-invoice rules this tool implements
- `CLAUDE.md` — guidance for AI coding agents working in this repo

## Testing

```
python3 -m unittest discover -s tests -t .
```
