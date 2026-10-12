---
name: openmatrix9-workflow
description: Use when starting or resuming OpenMatrix9 work, checking implementation progress, or choosing which Matrix9 reference documents to read next.
---

# OpenMatrix9 routing

## Ưu tiên ngôn ngữ

Rust là ưu tiên số 1 cho code OpenMatrix9 mới và chuyển code hiện hữu. Trước khi chọn ngôn ngữ hoặc thiết kế FFI/worker, đọc [Rust trước, native bridge tối thiểu](references/rust-first.md). Safe Rust sở hữu logic và dữ liệu độc lập; C++ dùng cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Không coi native FFI là memory-safe hoặc cập nhật skill là đã hoàn tất migration.

Keep the user's Rust implementation goal. Guide documents describe VB6 recovery methods; they do not change that goal.

Resolve the OpenMatrix9 checkout from the user's path or repository. This machine uses `D:/FreeCAD-src/Mod/OpenMatrix9`. All document paths in [stages.json](references/stages.json) are relative to that checkout.

Run through Python, using this skill's actual installed or repository path:

```powershell
python <skill-directory>/scripts/reference_index.py --project-root D:/FreeCAD-src/Mod/OpenMatrix9 status
```

Then run `route <stage>`:

| Stage | Required skill |
|---|---|
| inventory | openmatrix9-workflow |
| ui-menu / ui-viewports | openmatrix9-ui |
| native-mapping | openmatrix9-native-mapping |
| feature-port | openmatrix9-feature-port |
| build-validation | openmatrix9-build-validation |

Read `read_now` in order; read conditional files only for the stated task. Feature lookup: `search "Gem Loader"`, then `feature OM9-GEM-001`. Load one spec, not the complete catalog or Ghidra export.

Reconcile `docs/openmatrix9-progress.json` with code and tests. Record completed evidence, uncertainty, next document and next action using [progress.md](references/progress.md). Plan checkboxes, source presence and catalog `implementation_status` cannot establish completion.

Source conflicts or missing manuals: consult [source-policy.md](references/source-policy.md). Continue independent authorized work; report only the evidence gap affecting the selected task.

## Launcher và cập nhật OM9 trên Windows

Quy tắc người dùng duyệt ngày 2026-10-12; áp dụng khi làm launcher/cập nhật OM9:

- Chỉ kiểm tra release khi người dùng mở OM9 qua launcher/icon, một lần cho mỗi lần mở. Không thêm lịch, service hay vòng lặp kiểm tra định kỳ khi đang dùng app. Mất mạng không chặn mở OM9.
- Có bản mới, người dùng nhấn **Update** thì tự tải gói chính thức, xác minh checksum/manifest và cài; không chuyển sang trang tải để yêu cầu tải/giải nén thủ công. Nhận cả version có timestamp và revision launcher, không hạ phiên bản.
- Nếu cần thay plugin đang được dùng, chờ người dùng lưu và đóng OM9/FreeCAD rồi mới cài, sau đó mở lại cùng host/profile. Không tự đóng hoặc buộc tắt phiên đang làm. **Later** để lần mở sau; **Cancel** hủy tải/chờ và giữ bản đang cài.
- Lỗi tải/xác minh giữ nguyên bản đang cài; cài lỗi phải rollback cả plugin và launcher/cache trong AppData, giữ cấu hình/bản vẽ của người dùng.
- `installed` chỉ chứng minh cài xong và đã yêu cầu mở lại; cần báo cáo startup để xác nhận workbench đã mở. Phân biệt code/gói đã kiểm chứng với ZIP release đã phát hành; launcher cũ có thể cần cài mới một lần để nhận luồng này.

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
