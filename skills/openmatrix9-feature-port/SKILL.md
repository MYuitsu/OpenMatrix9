---
name: openmatrix9-feature-port
description: Use when implementing or specifying an OM9 feature such as Gem Loader, builders, settings, cutters, render, Clayoo/SubD or Emboss from the Matrix9 functional catalog into Rust and FreeCAD.
---

# One feature at a time

Resolve the OpenMatrix9 checkout and use openmatrix9-workflow `search`, then `feature <exact-ID>`. Read the returned spec, not all 607 entries. Run `route feature-port` for implementation rules, source map and terminology.

Before geometry-critical work, verify the cited manual page and follow the command's continuation pages until the next feature heading. Record the verified page range. `source_page_pdf` is 1-based; printed page is separate. Option cues and detected mentions are retrieval hints, not defaults or dependency graphs. Local manual locations are in `docs/openmatrix9-reference-locations.json`; check existence when used.

Create the feature's implementation record under `docs/features/<ID>.md`, preserving source-derived statements separately from OpenMatrix9 decisions. Use [feature-contract.md](references/feature-contract.md) to normalize selection/inputs, units/defaults, output, applicable lifecycle, error handling, undo/history/persistence and tolerances. Preserve original reference files.

Read Builder/History/Styles framework specs only when applicable to this feature. A detected word “Builder” does not establish an interactive builder lifecycle. For an unresolved original event, use openmatrix9-native-mapping on its exact procedure.

Implement the verified slice in Rust with native FreeCAD integration. Keep exact OM9 IDs in code/test/command mapping; Rhino command text alone is not a FreeCAD implementation. Use semantic tests for geometry/state and native host integration, then openmatrix9-build-validation for runtime proof.

If the manual is absent, identify the exact file/page and leave unproven semantics as `TODO_EVIDENCE`. Continue independent menu/infrastructure work; ask for missing evidence when it blocks the chosen geometry decision. A new OpenMatrix9 design choice must be labeled as such rather than claimed as recovered behavior.

After implementation and checks, follow [progress synchronization](../openmatrix9-workflow/references/progress.md): check the exact Spec v1 acceptance items, update implementation status and the project README, then reconcile the live ledger and its next document. An unchanged catalog `implementation_status: not_started` is not evidence that code is absent. Keep Clayoo/SubD (`14-subd`) distinct from T-Splines (`06-tsplines`), and Emboss (`15-emboss`) distinct from Matrix Art (`07-matrix-art`).

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](../openmatrix9-workflow/references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
