# Bộ skill OpenMatrix9

Nguồn skill được quản lý trong `skills/` của repository. Bản dùng cho Codex được cài vào `C:/Users/nguye/.codex/skills/`. Specs đích ở `OM9_Milestones/M01 - CAD Shell, Viewport/`; tài liệu gốc/private registry được giữ nguyên, đọc theo nhu cầu thay vì sao chép tất cả vào từng skill.

| Khi làm việc này | Skill | Tài liệu được chọn |
|---|---|---|
| Bắt đầu, tiếp tục, hỏi đã làm tới đâu | `openmatrix9-workflow` | Ledger tiến độ, bằng chứng code/test, bản đồ giai đoạn |
| Menu, sidebar, icon, màn hình khởi động, viewport | `openmatrix9-ui` | Thiết kế/kế hoạch đã có, INI/RUI, đúng form VB6 và đặc tả UI |
| Nghiệp vụ workspace đã duyệt: grid, Command, viewport | `openmatrix9-workspace-contract` | Quy tắc theo nhóm workspace và ảnh Matrix do người dùng cung cấp |
| Một event/hàm gốc chưa rõ | `openmatrix9-native-mapping` | Guide đối chiếu địa chỉ, đúng procedure VB6 và đúng hàm Ghidra |
| Một chức năng trang sức/hình học | `openmatrix9-feature-port` | Mã `OM9-*`, milestone spec của chức năng, migration/backend requirements; manual chỉ cho unknown |
| Build Rust OM9 độc lập + selective native backend và nghiệm thu | `openmatrix9-build-validation` | Cargo/CMake/ABI, cấu hình máy hiện tại, bằng chứng runtime |

## Cách gọi

Ví dụ yêu cầu trong Codex:

- “Dùng `$openmatrix9-workflow` xem đã làm tới đâu và tiếp tục menu.”
- “Dùng `$openmatrix9-ui` làm tiếp MAIN MENU theo ảnh và INI.”
- “Dùng `$openmatrix9-native-mapping` tìm hành vi `frmMaster.tmrProfileBrowser_Timer` để chuyển sang Rust.”
- “Dùng `$openmatrix9-feature-port` chuẩn hóa `OM9-L2-01-07` trước khi triển khai.”

Các description cũng hỗ trợ chọn skill tự động. Nếu catalog của phiên hiện tại chưa cập nhật, đọc trực tiếp `skills/<tên>/SKILL.md` trong repository; không cần chờ để sử dụng tài liệu.

## Tra cứu và tiếp tục

Chạy ở thư mục OpenMatrix9:

```powershell
rtk proxy python skills/openmatrix9-workflow/scripts/reference_index.py --project-root H:/OpenMatrix9 status
rtk proxy python skills/openmatrix9-workflow/scripts/reference_index.py --project-root H:/OpenMatrix9 route ui-menu
rtk proxy python skills/openmatrix9-workflow/scripts/reference_index.py --project-root H:/OpenMatrix9 search "Picking Selection"
rtk proxy python skills/openmatrix9-workflow/scripts/reference_index.py --project-root H:/OpenMatrix9 feature OM9-L2-01-07
rtk proxy python skills/openmatrix9-workflow/scripts/reference_index.py --project-root H:/OpenMatrix9 audit
```

`route` trả về tài liệu phải đọc ngay, tài liệu chỉ đọc khi điều kiện phù hợp và tiêu chí kết thúc giai đoạn. Các giai đoạn gồm `inventory`, `standalone-foundation`, `ui-menu`, `ui-viewports`, `native-mapping`, `feature-port`, `build-validation`.

`search` chỉ trả tối đa 5 kết quả mặc định; xem `total`, tăng `--limit` và lọc `--domain` khi cần. `feature` tra exact ID và trả file JSON L1/L2/L3/L4 cùng các hướng dẫn target thực có. `audit` kiểm tra toàn bộ đường dẫn đặc tả và tài liệu định tuyến, không chứng minh code đã hoạt động.

`status` đọc [openmatrix9-progress.json](openmatrix9-progress.json), sau đó đối chiếu dấu hiệu file. Tiến độ phải được cập nhật theo code/test/runtime, không dựa vào `implementation_status` của bộ spec. Old menu/native evidence là historical. Target standalone có own Rust document/session/history/UI; selected OCCT/openNURBS adapters và Matrix9 UI theo hợp đồng, QymCAD chỉ là code mẫu/routine.

## Khi có tài liệu mới

Giữ nguyên export gốc. Chạy lại `audit`; kiểm tra catalog ID và SHA-256 của input được dùng trong từng ghi chú phân tích. Nếu cấu trúc thư mục thay đổi, cập nhật `references/stages.json` và các bảng dẫn đường bị ảnh hưởng. Chỉnh nguồn trong `skills/`, kiểm chứng, rồi đồng bộ sang thư mục skill Codex. Không sửa `FEATURES.json` để thay thế ledger tiến độ.

## Khi nghiệp vụ thay đổi

Theo yêu cầu ngày 2026-10-05, sau khi code và kiểm tra xong, nếu có thay đổi nghiệp vụ thì trình bày hành vi trước/sau và nội dung đề xuất, rồi hỏi người dùng có ghi vào skill không. Chỉ cập nhật khi được đồng ý; yêu cầu cập nhật skill đã được người dùng nêu rõ cho cùng thay đổi không cần hỏi lại. Sửa kỹ thuật không đổi nghiệp vụ không cần câu hỏi này. Các skill áp dụng [quy tắc chung](../skills/openmatrix9-workflow/references/business-rule-updates.md).

Ngày 2026-10-06, người dùng duyệt lưu grid/Command/viewport vào skill nhóm mới `openmatrix9-workspace-contract`, kèm bốn ảnh tham chiếu và bản cài Codex.

Ngày2026-10-09, theo yêu cầu rõ của người dùng,7 skill OM9 ưu tiên Safe Rust số1 cho logic/state/validation/dữ liệu/index/cache/worker; C++ giữ bridge native, Python giữ bootstrap/API/test/tool khi cần. Đọc [quy tắc chung](../skills/openmatrix9-workflow/references/rust-first.md) trước lựa chọn ngôn ngữ/FFI/ownership. Đã kiểm chứng17 bản hiện hữu và3 reference đồng nhất; [bằng chứng](validation/2026-10-09-rust-first-skills.md). Thiết kế/kế hoạch chuyển logic Phase2 hiện tại sang Rust đã được duyệt; việc cập nhật skill không đồng nghĩa code đã chuyển xong.

Phase2 Rust migration đã được kiểm chứng riêng bằng code/native/application gates; xem [báo cáo](validation/2026-10-09-phase2-rust-owned-core.md). Việc kiểm tra skill chỉ đánh giá policy/routing.

## Đích mặc định 2026-10-19

Tất cả 8 skill nguồn và agents/openai.yaml dùng milestone specs và UI Matrix9; cả 8 skill được sync sang bản cài Codex để các liên kết chuyển giai đoạn hoạt động. Không fallback v1. Xem [quy tắc target chung](../skills/openmatrix9-workflow/references/milestones-target.md). Launcher/Box/Sphere và progress synchronization đã hợp nhất từ bản cài mới hơn, không làm mất nghiệp vụ đã duyệt.
