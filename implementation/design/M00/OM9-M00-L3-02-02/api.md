# Hợp đồng lifetime / lỗi / thread — gate M00-L3-02-02

Pin: OCCT **8.0.1** (`native/occt/pin.json`). Harness: `native/occt-rust-proof` (gate, không phải code sản phẩm).

## 1. RAII

- Mỗi con trỏ kernel trong bridge là một `Handle` cục bộ, chết ở cuối scope; không có handle nào đi qua ABI.
- Bằng chứng: refcount `1 → 2` khi copy, `1` sau `Nullify()` (`OM9-M00-L4-02-02-01`).

## 2. Lỗi

| Nguồn | Cách trả | Ví dụ |
| --- | --- | --- |
| Guard của bridge (NaN/∞/≤0) | `kInvalidInput` + message | `box dimensions must be finite and positive` |
| Ngoại lệ OCCT (`Standard_Failure`) | `kKernelFailure` + `what()` hoặc `ExceptionType()` | `BRep_Builder::Infinite parameter`, `Standard_ConstructionError` |
| Shape null/không valid | `kKernelFailure` + message | `boolean result is not a valid solid` |

**Không có kết quả một nửa**: hàm hoặc trả số, hoặc trả lỗi; không có trạng thái trung gian nào để rollback.

## 3. Thread

- **Chính sách**: gọi kernel **một lần một lúc**, qua một mutex ở tầng ứng dụng; UI thread không bao giờ gọi
  kernel trực tiếp mà post request rồi áp kết quả trên UI thread.
- Bằng chứng: 1 000 call từ 4 thread qua mutex → 0 sai; 100 call độc lập không mutex → 0 sai (ghi là **quan sát**,
  không phải cam kết thread-safety).

## 4. Bài test

`native/occt-rust-proof/tests/lifetime.rs`: refcount baseline, 9 ca invalid, 10k create/destroy với ngưỡng
tăng working set < 8 MiB, 400 call song song qua mutex.
