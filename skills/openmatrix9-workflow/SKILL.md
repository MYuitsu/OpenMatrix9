---
name: openmatrix9-workflow
description: Use when starting or resuming OpenMatrix9 work, checking implementation progress, or choosing which Matrix9 reference documents to read next.
---

# OpenMatrix9 routing

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

Reconcile `docs/openmatrix9-progress.json` with code and tests. Record completed evidence, uncertainty, next document and next action using [progress.md](references/progress.md). After implementation and checks, follow its required Spec v1/checklist and README synchronization before reporting completion. Plan checkboxes, source presence and catalog `implementation_status` cannot establish completion.

Source conflicts or missing manuals: consult [source-policy.md](references/source-policy.md). Continue independent authorized work; report only the evidence gap affecting the selected task.

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
