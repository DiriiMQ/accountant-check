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

### Desktop app (no terminal needed)

Download the `accountant-check.exe` asset from the [latest release](../../releases/latest) (Windows
only). Double-click it, pick your invoice `.xlsx` in the file dialog, and a summary appears when it's
done — `invalid_data.xlsx` is written next to your input file. No install, no Python required.

A new release (with a freshly built exe) is published automatically whenever a `vX.Y.Z` tag is pushed —
see `.github/workflows/release.yml`.

## Project structure

- `invoice_validator/` — the package. Rule-registry design: `helpers.py` (pure functions shared across
  rules), `field_rules.py` / `cross_row_rules.py` (rule classes), `engine.py` (default registry +
  runner), `report.py`, `cli.py`. Add a new check by implementing `Rule` and adding it to the registry —
  no existing rule code needs editing. `gui.py` is a minimal tkinter file-picker front-end for the same
  pipeline, used to build the desktop exe (`run_gui.py` is its PyInstaller entry point).
- `tests/` — `unittest` test suite (stdlib only, no pytest dependency)
- `docs/` — reference notes on the Vietnamese e-invoice rules this tool implements
- `.github/workflows/` — CI (tests on every push/PR to `main`) and the release build (Windows exe,
  triggered by version tags)
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

### Ứng dụng desktop (không cần dùng terminal)

Tải file `accountant-check.exe` trong mục [latest release](../../releases/latest) (chỉ dùng cho
Windows). Double-click để mở, chọn file hóa đơn `.xlsx` qua hộp thoại, xong sẽ hiện bảng tóm tắt kết
quả — file `invalid_data.xlsx` được lưu ngay cạnh file đầu vào. Không cần cài đặt, không cần Python.

Mỗi khi có một tag phiên bản mới dạng `vX.Y.Z` được đẩy lên, hệ thống sẽ tự động build lại file exe và
tạo release mới — xem `.github/workflows/release.yml`.

### Cấu trúc dự án

- `invoice_validator/` — package chính, thiết kế theo kiểu "rule registry": `helpers.py` (các hàm dùng
  chung), `field_rules.py` / `cross_row_rules.py` (các lớp rule), `engine.py` (danh sách rule mặc định +
  bộ chạy), `report.py`, `cli.py`. Muốn thêm một kiểm tra mới chỉ cần viết thêm một `Rule` và thêm vào
  danh sách — không cần sửa rule đã có. `gui.py` là giao diện chọn file đơn giản dùng tkinter, dựa trên
  cùng pipeline xử lý, dùng để build file exe desktop (`run_gui.py` là entry point cho PyInstaller).
- `tests/` — bộ test dùng `unittest` (thư viện chuẩn, không cần cài pytest)
- `docs/` — tài liệu tham khảo về quy định hóa đơn điện tử Việt Nam mà công cụ này áp dụng
- `.github/workflows/` — CI (chạy test mỗi khi push/PR vào `main`) và release build (file exe Windows,
  kích hoạt khi đẩy tag phiên bản)
- `CLAUDE.md` — hướng dẫn dành cho các AI coding agent khi làm việc trong repo này

### Chạy test

```
python3 -m unittest discover -s tests -t .
```
