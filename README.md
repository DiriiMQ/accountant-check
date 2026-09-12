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

---

## Tiếng Việt

Kiểm tra dữ liệu của một file danh sách hóa đơn (Excel) tại Việt Nam: phát hiện lỗi dữ liệu và các
trường hợp vi phạm ngưỡng thanh toán không dùng tiền mặt / điều kiện khấu trừ thuế GTGT, sau đó ghi các
dòng bị gắn cờ ra một file báo cáo riêng. File dữ liệu đầu vào không bị chỉnh sửa.

### Các mục kiểm tra

- **Thiếu/sai định dạng dữ liệu** — số hóa đơn, ngày lập, tên người bán, mã số thuế (định dạng), số tiền
- **Dòng trùng lặp** — cùng số hóa đơn, ngày, người bán, mã số thuế và số tiền
- **Tổng tiền trong ngày của cùng một người bán vượt ngưỡng thanh toán** — tính theo ngày hóa đơn:
  20.000.000 VNĐ trước ngày 01/07/2025, 5.000.000 VNĐ từ ngày đó trở đi, theo quy định hiện hành (xem
  `docs/vietnam-e-invoice-knowledge-base.md`)

### Yêu cầu

Python 3.10 trở lên, pandas, openpyxl.

```
pip install -r requirements.txt
```

### Cách sử dụng

```
python3 -m invoice_validator <file_dau_vao.xlsx> [-o bao_cao.xlsx] [--threshold 5000000]
```

Cấu trúc file đầu vào: một sheet duy nhất, các dòng tiêu đề từ 1-14, dữ liệu bắt đầu từ dòng 15, các cột
B-I (xem `CLAUDE.md` để biết chi tiết ánh xạ từng cột).

Kết quả: một file báo cáo với một sheet cho mỗi loại lỗi/vi phạm (`Du_lieu_loi`, `Trung_lap`,
`Vuot_nguong`) và một sheet tổng hợp `Tong_hop`.

### Cấu trúc dự án

- `invoice_validator/` — package chính, thiết kế theo kiểu "rule registry": `helpers.py` (các hàm dùng
  chung), `field_rules.py` / `cross_row_rules.py` (các lớp rule), `engine.py` (danh sách rule mặc định +
  bộ chạy), `report.py`, `cli.py`. Muốn thêm một kiểm tra mới chỉ cần viết thêm một `Rule` và thêm vào
  danh sách — không cần sửa rule đã có.
- `tests/` — bộ test dùng `unittest` (thư viện chuẩn, không cần cài pytest)
- `docs/` — tài liệu tham khảo về quy định hóa đơn điện tử Việt Nam mà công cụ này áp dụng
- `CLAUDE.md` — hướng dẫn dành cho các AI coding agent khi làm việc trong repo này

### Chạy test

```
python3 -m unittest discover -s tests -t .
```
