# Import 3DM: sửa crash khi bấm menu Qt — 2026-10-08

File người dùng: `C:/Users/nguye/Downloads/nhan oval 11.91x8.37x4.97.3dm`, V5, 31 bản ghi nguồn. SHA-256: `5cdd84533c2dca20ef441fcbe2f1edd0226cc3ab02805caa4af60ef3dbba9997`. Không ghi/sửa file gốc.

## Nguyên nhân và sửa

Crash Windows ở Python 3.13 `_PyObject_Malloc`; các địa chỉ stack đã giải symbol nằm trên đường `PyUnicode_FromString → PyImport_ImportModule → executeThreeDm → RustCommand::activated → QAction/QMenu`. Đây là danh sách địa chỉ stack phù hợp symbol, không phải kết quả unwind đầy đủ. Module phát sinh lỗi `CoreThreeDm.cpp` dùng Python C API khi lệnh Qt được kích hoạt mà chưa lấy GIL. Native command của FreeCAD không tự khóa GIL cho mọi implementation; các đường `_runCommand` của host dùng `Base::PyGILStateLocker`.

Sửa nhỏ: lấy `Base::PyGILStateLocker` sau khi hoàn tất chọn file/mode và kiểm tra lại document, trước Python C API đầu tiên; giữ khóa qua xử lý exception và DECREF. Cùng boundary bảo vệ Import và Export. Không đổi thuật toán hình học, ngưỡng, chính sách lưu nguồn hay guard xuất V5.

Test UI trước đó gọi `Gui.runCommand` bên trong Python nên có GIL sẵn và không phát hiện lỗi này. Regression mới gửi **queued native QAction.trigger** rồi để callback Python trả về, trước khi Qt thực thi lệnh; không gọi Gui.runCommand cho action được đo.

## Kết quả đo

- RED binary trước sửa: cùng nguồn nhập qua API được, native QAction báo **Illegal storage access**, không hoàn thành nhập. Retry chỉ bổ sung capture hộp lỗi; không tính thành bộ test mới. Attempt đầu còn lỗi đo quá sớm, không dùng làm bằng chứng converter.
- GREEN binary sửa: **8 kiểm tra native QAction**, Import file người dùng tạo **52** hình học như đường API (tính cả geometry trong definition), tất cả BRep kiểm tra hợp lệ. Cửa sổ tương tác có 29 đối tượng nhập cấp trên, gồm block instances; không tính 52 là số đối tượng nhẫn hiển thị. Native Export kiểm tra bằng hộp độc lập 2×3×4 mm, xuất V5 hợp lệ. Export nhẫn người dùng và Rhino 5 roundtrip của file mới chưa được kiểm chứng trong đợt này.
- Hồi quy giao diện: **18 kiểm tra FreeCAD thật** đạt, dialog/cancel/selection/menu/toolbar và API roundtrip hộp. Process exit 0 ở cả hai GREEN.
- Source hash trước/sau giữ nguyên. [RED](native-menu-gil-20261008/red-results.json), [GREEN](native-menu-gil-20261008/green-results.json), [UI](native-menu-gil-20261008/ui-results.json), [hashes](native-menu-gil-20261008/manifest.json).

Build/runtime riêng: `H:/FreeCAD-src/build/om9-ui-3dm-gil-sdk`. Không ghi đè DLL mà cửa sổ người dùng đang tải. Launcher `H:/FreeCAD-src/build/launch-om9-ring.ps1` đã trỏ tới bản sửa; có tham số `-ThreeDmPath` để mở file nhẫn mới. Cửa sổ đã mở trước đó vẫn dùng binary cũ và cần mở lại bằng launcher mới.

Đợt regression native menu này thêm do lỗi thực tế người dùng. 0 bộ Rhino đã chuẩn bị đang chờ; 7 đợt openNURBS chưa đóng; tổng số bộ tương lai chưa xác định. Đây là kiểm chứng crash/menu trên binary mới, không thay bằng chứng 51 suite native/39 báo cáo hình học của checkpoint trước. Source chính chưa tích hợp, không push.
