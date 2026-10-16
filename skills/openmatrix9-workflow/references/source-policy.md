# Nguồn và thẩm quyền

[Target specs v2](spec-v2-target.md) là mặc định. User requirements/AGENTS hiện hành đứng trước imported guides; mission recovery VB6 không thay target OM9 Rust.

| Câu hỏi | Nguồn trước |
|---|---|
| Target/architecture | V2 README/AGENTS/RCORE-00/CODE_GUIDE/BACKEND_POLICY |
| UI và workflow | V2 MATRIX9_UI_CONTRACT, exact feature/UI_DETAILS; authored Resources/menu/MainMenu.ini cho IDs/order |
| Parameters/defaults/units/tolerance | Exact feature v2 và engineering contracts; unknown chỉ tra đúng source/manual/procedure còn thiếu |
| Status | Code/test/runtime fingerprint + docs/openmatrix9-progress.json, không catalog/source presence |
| Port hiện hữu | V2 MIGRATION_LEDGER và migration/legacy-* evidence; source slices/native reports là historical |
| QymCAD routines | V2 UPSTREAM_REUSE/UPSTREAM_SOURCE_AUDIT/LICENSING_AND_PROVENANCE; code mẫu, không target UI |
| VA/signature/native behavior chưa rõ | Registry recovered_inputs + Matrix90_Codex_Guide_MD, exact definition/callers cần thiết |

User đã chọn local specs làm nguồn đủ cho triển khai; không bắt đọc lại PDFs cho thông tin đã normalized. Manual/raw sources chỉ cho unknown thật, không điền defaults bằng cue. Check file existence và hash khi dùng; giữ raw inputs/exports bất biến, không copy commercial artwork/manual/decompile/private models vào sản phẩm.

Nguồn riêng được resolve qua `docs/openmatrix9-reference-locations.json` (`manuals`, `recovered_inputs`, `reference_guides`, `other_private_references`). Không dùng Downloads hoặc ref/matrix9 như portable constants. Guide ở `Matrix90_Codex_Guide_MD` của checkout; runtime assets dùng authored Resources, độc lập máy này. Không đổi tên đường dẫn kho manual v1 lịch sử thành v2 nếu file thật vẫn ở đó.
