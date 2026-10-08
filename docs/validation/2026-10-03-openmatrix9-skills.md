# Kiểm chứng bộ skill OpenMatrix9 — 2026-10-03

## Phạm vi

Chuyển ba bộ tài liệu mới thành năm skill tham chiếu theo giai đoạn; có công cụ tra cứu, ledger tiến độ, registry manual và hướng dẫn tự chọn skill trong repository. Không triển khai hoặc build giao diện trong lần làm này. Giữ nguyên các export/tài liệu gốc.

Nguồn nằm ở `skills/`; bản cài ở `C:/Users/Admin/.codex/skills/`. Mục tiêu triển khai vẫn là Rust + host C++/Qt FreeCAD.

## Kết quả kiểm tra

- `python -m unittest discover -s tools/tests -p test_skill_reference_index.py -v`: **8/8 đạt**. Bảy test đầu đã thất bại khi helper chưa tồn tại, rồi đạt sau triển khai. Test bổ sung xác nhận cả sáu giai đoạn chuyển tới skill thực sự tồn tại.
- Helper `audit`: **607 spec, 16 domain, 6 giai đoạn**, không có đường dẫn spec sai, không thiếu tài liệu bắt buộc hoặc có điều kiện.
- `quick_validate.py`: năm nguồn và năm bản cài đều hợp lệ. Dùng `D:/FreeCAD-src/.pixi/envs/default/python.exe -X utf8` vì Python mặc định không có PyYAML; helper tra cứu chỉ dùng standard library.
- Metadata `agents/openai.yaml`: tên khớp thư mục, mô tả ngắn 25–64 ký tự, prompt chứa đúng `$skill-name`; các link hỗ trợ trong Markdown đều tồn tại. Mỗi SKILL dưới 500 từ.
- SHA-256 mọi file trong bản cài khớp nguồn repository. Helper chạy từ bản cài cũng đạt audit.
- Ba manual PDF trong Downloads và ảnh tham chiếu đăng ký trong `docs/openmatrix9-reference-locations.json` đều tồn tại.

## Thử truy hồi/áp dụng độc lập

Baseline không có skill vẫn tìm được phần lớn nguồn đúng, nhưng phải tự khám phá nhiều file; có lỗi tìm wildcard PowerShell và nguy cơ nhầm trạng thái catalog với code. Không ghi nhận baseline này là thất bại toàn bộ.

| Skill | Tình huống và kết quả |
|---|---|
| workflow | Xác định menu chỉ mới planned, tra Gem Loader đúng ID/page, giữ mục tiêu Rust khi chọn native route. Ban đầu thiếu skill đồng hành vì suite đang viết; toàn bộ handoff được kiểm lại sau khi đủ năm skill. |
| ui | Chọn đúng thiết kế/Task1/INI/form; SNAPS và Layers tra được ID mà không đọc Ghidra. Sửa hai gap: đọc layout dừng ở Attribute VB_Name, và tăng limit để thấy đủ16 Snap entries; retest đạt. |
| native-mapping | Timer map0x014DE3A0 → FUN_014de3a0; phân biệt EXACT/A với độ chắc chắn hành vi, nhận ra nhánh lỗi bị pseudocode bỏ sót. Form_Load timeout phân loại EXACT_DECOMP_FAIL/B. Không thực hiện phục hồi toàn bộ VB6. |
| feature-port | Gem Loader OM9-GEM-001/gvLoader, Clay Edit OM9-SUBD-001/ClayEdit; giữ SubD riêng T-Splines. Phát hiện manual ở Downloads và trang tiếp nối Gem Loader; bổ sung range176–177 và registry, retest đạt. |
| build-validation | Đối chiếu code/ABI/parent/cache: menu chưa triển khai, target chưa có, cache cũ E:. Chọn chuẩn bị build mới sau sửa code/registration; không suy thành công từ FreeCAD.exe hiện có. Cả sáu handoff giải đúng file skill. |

Các thử nghiệm chỉ đọc, không build hoặc chạy FreeCAD. Test truy hồi không chứng minh các chức năng Matrix9 đã được port.

## Fingerprint nguồn tra cứu

SHA-256 đo trực tiếp tại lần kiểm tra này:

| Input | SHA-256 |
|---|---|
| `ref/MainMenu.ini` | `1E3B28C5265D86C32828C963C21FB4EC8F1142121C0DBB47E6FAEB6788843D12` |
| `ref/matrix9/OpenMatrix9_Codex_Spec_v1/FEATURES.json` | `2F84351E7F0F76A7A921A802DD3F3A018EA1DA42DFF5BB47E7F6444333713B81` |
| `ref/matrix9/Matrix90_Codex_Guide_MD/README.md` | `AB23B7E2B9A0427B9588C8C40221EB3EA24096C31F42558B07FD2F998FFDEEAC` |
| `ref/matrix9/ghidra/Matrix90.exe.c` | `DCBEE0F15CA62C18C19273403165AF3CCDBB7064ACC5988090148F46F0C1E2CB` |

Đây là fingerprint của các input được nêu, không phải hash toàn bộ cây tài liệu. Khi export/catalog thay đổi, chạy lại audit và xác minh các note/phân tích bị ảnh hưởng.

## Tiến độ sản phẩm

`docs/openmatrix9-progress.json` giữ MAIN MENU ở `planned`. Code Rust hiện chưa cung cấp đầy đủ ABI; sidebar và build/runtime UI chưa có bằng chứng hoàn thành. Bước tiếp theo là Task1 trong kế hoạch menu, thông qua `openmatrix9-ui`.
