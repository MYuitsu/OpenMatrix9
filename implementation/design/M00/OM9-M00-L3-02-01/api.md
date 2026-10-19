# C ABI contract — OCCT 8.0.1 gate proof (`native/occt-rust-proof`)

Pin: `native/occt/pin.json` (OCCT **8.0.1**, tag `V8_0_1`, commit `b8f597c…`).

Đây là **hợp đồng của proof harness** dùng cho gate M00. Crate sản phẩm `crates/om9-kernel` sẽ được tạo ở **M01-L2-01-05** với cùng hợp đồng này; M00 không chứa code sản phẩm.

## 1. Ranh giới

Rust **không** nhận con trỏ kernel. Mỗi hàm là **một thao tác hoàn chỉnh**: nhận số, trả số (hoặc mã lỗi +
message). `TopoDS_Shape`, `Handle(...)`, `Standard_Transient` không bao giờ đi qua ABI, nên không có
lifetime/refcount nào để Rust phải quản lý.

```
Rust  om9_occt_rust_proof::{version, box_volume, drilled_volume, reject_invalid_cylinder}
                      │  (C ABI, extern "C")
C++   native/occt-rust-proof/src/bridge.cpp  ──►  OCCT 8.0.1 (DLL, LGPL-2.1 + exception)
```

## 2. Hàm

| Hàm | Vào | Ra | Ghi chú |
| --- | --- | --- | --- |
| `om9_occt_version()` | — | `const char*` (static) | `OCC_VERSION_COMPLETE`, phải là `8.0.1` |
| `om9_occt_box_volume(w,h,d, *volume, msg, cap)` | 3 số thực dương hữu hạn | `0/1/2` + `volume` | `BRepPrimAPI_MakeBox` + `BRepGProp` + `BRepCheck_Analyzer` |
| `om9_occt_drilled_volume(w,h,d,r, *volume, msg, cap)` | 4 số thực dương hữu hạn | `0/1/2` + `volume` | `BRepAlgoAPI_Cut(box, cylinder)` + `BRepCheck` |
| `om9_occt_invalid_cylinder(r, msg, cap)` | 1 số (cố ý sai) | `0/2` | chứng minh ngoại lệ kernel trở thành mã lỗi |

## 3. Mã trạng thái và ánh xạ lỗi

| Mã | Ý nghĩa | Rust |
| --- | --- | --- |
| `0` | thành công | `Ok(value)` |
| `1` | input bị từ chối trước khi gọi kernel (NaN/∞/≤0, con trỏ null) | `Err(KernelError::InvalidInput(msg))` |
| `2` | kernel ném ngoại lệ, hoặc shape trả về null/không valid | `Err(KernelError::Failure(msg))` |

`msg` là buffer **cố định 512 byte**, luôn NUL-terminated, không bao giờ ghi quá `cap−1`. Ngoại lệ được bắt
bằng `catch (const std::exception&)`: từ OCCT 8.0, `Standard_Failure` kế thừa `std::exception`, nên một
`catch` là đủ cho cả lỗi OCCT và lỗi chuẩn C++. Trên MSVC bridge biên dịch với `/EHa` để access violation
của kernel cũng thành ngoại lệ C++ thay vì giết process.

## 4. Bất biến

1. **Không có trạng thái ẩn**: mỗi call tạo shape, dùng, rồi huỷ trong cùng một hàm.
2. **Không có con trỏ ra ngoài**: mọi thứ kernel tạo ra chết trong bridge.
3. **Kiểm trước khi trả**: kết quả boolean phải `BRepCheck_Analyzer(...).IsValid()`; nếu không → `Failure`.
4. **Chỉ đọc tham số môi trường lúc build**: `OCCT_INCLUDE_DIR`/`OCCT_LIB_DIR`/`OM9_OCCT_SDK`; lúc chạy cần
   `occt-8.0.1-sdk/bin` + `tbb12.dll` + `jemalloc.dll` trên `PATH` (hoặc cạnh exe).
5. **Version là một phần của hợp đồng**: test khẳng định `version().starts_with("8.0.")`; nâng OCCT phải
   chạy `implementation/tools/occt_upgrade_gate.py` trước.

## 5. Fixture

- Dương: `box_volume(10,10,4) = 400` (sai ≤ 1e-9); `drilled_volume(10,10,4,2) = 400 − π·2²·4` (sai ≤ 1e-6).
- Âm: `box_volume(0,1,1)` → `InvalidInput`; `reject_invalid_cylinder(-1)` → `Failure` có message.
