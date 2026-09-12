# Knowledge Base: Hóa đơn điện tử tại Việt Nam — Ký hiệu, số hóa đơn và ngưỡng thanh toán

> **Cập nhật:** 12/09/2026  
> **Phạm vi:** Quy định hiện hành về hóa đơn điện tử và điều kiện thanh toán không dùng tiền mặt để khấu trừ thuế GTGT.  
> **Lưu ý:** Nội dung này là bản tổng hợp phục vụ tra cứu/triển khai hệ thống; khi xử lý hồ sơ thuế thực tế cần đối chiếu văn bản pháp luật hiện hành.

---

## 1. Ký hiệu mẫu số và ký hiệu hóa đơn

### 1.1. Ký hiệu mẫu số hóa đơn

Ký hiệu mẫu số là **01 chữ số**, dùng để nhận biết loại hóa đơn/chứng từ.

Theo quy định hiện hành, các nhóm gồm:

| Ký hiệu | Loại |
|---|---|
| `1` | Hóa đơn giá trị gia tăng |
| `2` | Hóa đơn bán hàng |
| `3` | Hóa đơn bán tài sản công |
| `4` | Hóa đơn bán hàng dự trữ quốc gia |
| `5` | Các loại hóa đơn điện tử khác |
| `6` | Phiếu xuất kho kiêm vận chuyển nội bộ / hàng gửi bán đại lý |
| `7` | Hóa đơn thương mại điện tử |
| `8` | Hóa đơn GTGT tích hợp biên lai |
| `9` | Hóa đơn bán hàng tích hợp biên lai |

**Reference:** Thông tư 91/2026/TT-BTC.

---

### 1.2. Ký hiệu hóa đơn

Ký hiệu hóa đơn gồm **06 ký tự**.

Ví dụ:

```text
C26TAA
```

Có thể hiểu theo cấu trúc:

```text
C 26 T AA
│ │  │  └── Ký hiệu do người bán xác định
│ │  └───── Loại hóa đơn
│ └──────── Năm lập hóa đơn
└────────── Hóa đơn có mã cơ quan thuế
```

Một cách nhận diện đầy đủ có thể được biểu diễn:

```text
1C26TAA
```

Trong đó:

- `1`: ký hiệu mẫu số — hóa đơn GTGT.
- `C`: hóa đơn có mã của cơ quan thuế.
- `26`: năm lập hóa đơn 2026.
- `T`: loại hóa đơn theo quy định.
- `AA`: ký hiệu do người bán xác định để phân biệt.

**Reference:** Thông tư 91/2026/TT-BTC.

---

## 2. Số hóa đơn

**Số hóa đơn** không phải là ký hiệu hóa đơn.

Ví dụ:

```text
Ký hiệu mẫu số: 1
Ký hiệu hóa đơn: C26TAA
Số hóa đơn: 00001234
```

Số hóa đơn là **số thứ tự bằng chữ số Ả-rập**, được sử dụng để đánh số hóa đơn trong phạm vi ký hiệu/mẫu số tương ứng.

Theo quy định hiện hành:

- Số hóa đơn tối đa **08 chữ số**.
- Bắt đầu từ số `1`.
- Số hóa đơn được lập theo thứ tự.
- Số tối đa là `99.999.999`.

Ví dụ:

```text
00000001
00000002
00000003
...
00001234
```

### 2.1. Phân biệt các thành phần

| Thành phần | Ví dụ | Ý nghĩa |
|---|---|---|
| Ký hiệu mẫu số | `1` | Loại hóa đơn |
| Ký hiệu hóa đơn | `C26TAA` | Nhận diện series/ký hiệu |
| Số hóa đơn | `00001234` | Số thứ tự của hóa đơn |

**Reference:** Thông tư 91/2026/TT-BTC.

---

# 3. Ngưỡng 20 triệu và 5 triệu

## 3.1. Quy định cũ: ngưỡng 20 triệu đồng

Trước đây, quy định về điều kiện khấu trừ thuế GTGT thường được nhắc đến với mốc:

> **20 triệu đồng**

Hàng hóa/dịch vụ mua vào từ mức này trở lên phải đáp ứng điều kiện về **thanh toán không dùng tiền mặt** để được khấu trừ thuế GTGT đầu vào.

Vì vậy câu nói phổ biến:

> "Hóa đơn trên 20 triệu phải chuyển khoản."

là cách diễn đạt của **quy định cũ**.

---

## 3.2. Quy định hiện hành: ngưỡng 5 triệu đồng

### Thời điểm áp dụng

Mốc **5.000.000 đồng** bắt đầu áp dụng từ **01/07/2025**.

- **Trước 01/07/2025:** quy định cũ sử dụng ngưỡng **20.000.000 đồng**.
- **Từ 01/07/2025:** ngưỡng mới là **5.000.000 đồng**.
- **Hiện tại:** ngưỡng **5.000.000 đồng** đang được áp dụng.

Theo quy định hiện hành, ngưỡng liên quan đến điều kiện thanh toán không dùng tiền mặt là:

> **Từ 5.000.000 đồng trở lên, đã bao gồm thuế GTGT.**

Nếu muốn khấu trừ thuế GTGT đầu vào, giao dịch thuộc ngưỡng này phải đáp ứng điều kiện về **chứng từ thanh toán không dùng tiền mặt**.

### Timeline

| Thời gian | Ngưỡng |
|---|---:|
| Trước 01/07/2025 | 20.000.000 đồng |
| Từ 01/07/2025 | **5.000.000 đồng** |
| Hiện tại | **5.000.000 đồng** |

**Effective date:** `2025-07-01`  
**Primary reference:** Luật Thuế GTGT 2024; Nghị định 181/2025/NĐ-CP.

### Ví dụ

| Giá trị thanh toán | Điều kiện theo ngưỡng 5 triệu |
|---:|---|
| 4.999.000 | Không thuộc ngưỡng này |
| 5.000.000 | Thuộc ngưỡng |
| 5.500.000 | Thuộc ngưỡng |
| 20.000.000 | Thuộc ngưỡng |

**Reference:** Luật Thuế GTGT 2024; Nghị định 181/2025/NĐ-CP.

---

# 4. Nhiều giao dịch trong cùng một ngày

Không nên chỉ kiểm tra:

```text
invoice.amount >= 5,000,000
```

mà bỏ qua các giao dịch khác trong cùng ngày.

Trường hợp mua hàng hóa/dịch vụ của **cùng một người bán trong cùng một ngày**, nếu tổng giá trị đạt ngưỡng theo quy định thì điều kiện thanh toán không dùng tiền mặt cũng cần được xem xét.

### Ví dụ

```text
09:00  — Mua hàng: 3.000.000
14:00  — Mua hàng: 2.500.000
-------------------------------
Tổng:             5.500.000
```

Tổng trong ngày:

```text
5.500.000 >= 5.000.000
```

Do đó không thể chỉ dựa vào việc từng hóa đơn riêng lẻ đều dưới 5 triệu để kết luận rằng có thể thanh toán tiền mặt mà vẫn đáp ứng điều kiện khấu trừ VAT.

**Reference:** Nghị định 181/2025/NĐ-CP.

---

# 5. Thanh toán trả chậm / trả góp

Đối với giao dịch từ ngưỡng 5 triệu đồng thuộc trường hợp **trả chậm hoặc trả góp**, cần phân biệt:

1. Thời điểm lập hóa đơn / khấu trừ VAT.
2. Thời điểm đến hạn thanh toán.
3. Việc thực tế có chứng từ thanh toán không dùng tiền mặt hay không.

Nếu chưa đến thời hạn thanh toán theo hợp đồng, người mua có thể xử lý khấu trừ theo điều kiện áp dụng.

Khi đến hạn thanh toán, nếu không đáp ứng điều kiện thanh toán không dùng tiền mặt thì phải xử lý điều chỉnh phần thuế GTGT đầu vào đã khấu trừ theo quy định.

**Reference:** Nghị định 144/2026/NĐ-CP và các quy định về điều kiện khấu trừ thuế GTGT hiện hành.

---

# 6. Quy tắc triển khai hệ thống

Nếu xây dựng hệ thống kế toán/invoice, nên tách riêng các khái niệm:

```text
invoice_template_code
invoice_symbol
invoice_number
```

Ví dụ:

```json
{
  "invoice_template_code": "1",
  "invoice_symbol": "C26TAA",
  "invoice_number": "00001234"
}
```

Không nên coi:

```text
C26TAA
```

là invoice number.

Đúng hơn:

```text
Invoice symbol = C26TAA
Invoice number = 00001234
```

---

## 7. Rule kiểm tra ngưỡng thanh toán

Ở mức business logic, có thể mô hình hóa:

```text
if total_purchase_value >= 5,000,000 VND:
    require_non_cash_payment_evidence = true
```

Nhưng `total_purchase_value` không nhất thiết chỉ là giá trị của một invoice. Cần xác định phạm vi aggregation theo quy định, đặc biệt:

- Cùng người bán.
- Cùng ngày.
- Giá trị đã bao gồm VAT.
- Trường hợp trả chậm/trả góp.
- Các trường hợp ngoại lệ/đặc thù được pháp luật quy định.

**Không nên hard-code quy tắc cũ `20,000,000` nếu hệ thống đang áp dụng quy định hiện hành.**

---

# 8. Quick Reference

| Chủ đề | Quy tắc |
|---|---|
| Ký hiệu mẫu số | 01 chữ số |
| Ký hiệu hóa đơn | 06 ký tự |
| Số hóa đơn | Tối đa 08 chữ số |
| Số hóa đơn bắt đầu | `1` |
| Mức tối đa | `99.999.999` |
| Ngưỡng thanh toán hiện hành | **5 triệu đồng** |
| Effective date của mốc 5 triệu | **01/07/2025** |
| Trước 01/07/2025 | Ngưỡng cũ **20 triệu đồng** |
| 20 triệu | Ngưỡng quen thuộc của quy định cũ |
| 5 triệu | Tính theo giá trị **đã bao gồm VAT** |
| Nhiều giao dịch cùng người bán/cùng ngày | Cần xem xét tổng giá trị |
| Trả chậm/trả góp | Có quy định riêng về thời điểm thanh toán |

---

# 9. References

1. **Thông tư 91/2026/TT-BTC** — quy định chi tiết về hóa đơn, chứng từ và ký hiệu hóa đơn.
2. **Nghị định 254/2026/NĐ-CP** — quy định về hóa đơn, chứng từ.
3. **Luật Thuế giá trị gia tăng 2024** — Luật số 48/2024/QH15.
4. **Nghị định 181/2025/NĐ-CP** — quy định chi tiết thi hành một số điều của Luật Thuế GTGT.
5. **Nghị định 144/2026/NĐ-CP** — sửa đổi, bổ sung một số quy định về thuế GTGT.

### Online references

- Thông tư 91/2026/TT-BTC: https://thuvienphapluat.vn/van-ban/Thue-Phi-Le-Phi/Circular-91-2026-TT-BTC-elaboration-Decree-254-2026-ND-CP-716304.aspx
- Nghị định 181/2025/NĐ-CP: https://vanban.chinhphu.vn/
- Cổng thông tin Chính phủ: https://vanban.chinhphu.vn/
- Cổng thông tin Bộ Tài chính: https://mof.gov.vn/

---

## 10. Important note

Các quy định về hóa đơn điện tử và điều kiện khấu trừ VAT có thể được sửa đổi/bổ sung. Khi implement hệ thống production, nên lưu:

- `effective_from`
- `effective_to`
- version của rule
- nguồn văn bản pháp luật

thay vì hard-code một ngưỡng duy nhất trong business logic.
