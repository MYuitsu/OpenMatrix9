# M08/E13 — G-FULL release gate (planning-only)

**Entry:** G00 signed PASS, E01…E12 PASS và **G-MVP signed PASS**. M08 chưa có bằng chứng được phép bắt đầu; không được lấy kế hoạch làm gate. **Exit:** tất cả 73 L3 / 221 L4 có implementation evidence PASS thực, chức năng phù hợp giấy phép, người dùng nghiệm thu interactive FEBE, QA độc lập duyệt G-FULL.

## 1. Nhóm tiêu chí định lượng và hành vi

1. **Smart Target:** cập nhật mạng spline khi điểm điều khiển di chuyển, không mất continuity G0 và mức G1/G2 nếu mode yêu cầu; geometry valid, cập nhật history deterministically.
2. **Smart Flow:** mapping UV corner/inside, normal orientation, periodic surface seams, overhang, tolerance; rigid placement giữ shape gem (độ dài/volume không đổi trong dung sai được duyệt), bezels không tách không cảnh báo. Base/Target updates phải rebuild đúng DAG; Break giữ geometry, reconnect có kiểm tra nguồn.
3. **Orient/Array/Flow/Splop:** arc-length spacing, copies/count, local frame, trimmed UV, no seam duplicates or normal flip; source identity preserved when Copy. Regression ở ring/gem settings từ MVP.
4. **Deformations:** Bend/Taper/Stretch/Twist/Shear/Maelstrom/MoveUVN/SoftMove/Cage/Symmetry: identity/axis cases, singular inputs, Jacobian/topology validity; preview/cancel atomic; live mirror không tạo dependency cycle; Release cage không xóa hình đã commit.
5. **Ornament Builders:** dimensions, knot crossing order, spiral progression, jump-ring open-vs-closed geometry, style/history roundtrip; không thêm bất kỳ Gemvision asset nào nếu chưa cấp phép.
6. **Smart Pattern:** deterministic rows×columns theo UV trim policy; PatternType and attractor Towards; Group Output and Output Mesh must be deliberately separate; safety cap on enormous arrays from agreed resource budgets; Undo/Save regression.
7. **Rope:** count, thread diameter, signed pitch/spacing, left/right twist, caps None/Flat/Round; `None` là open by design và không được coi là closed STL; check winding/solids prior to production.
8. **User Libraries:** category folder catalog, indexing, thumbnails, gem search, asset selection, Import vs Open semantics, Save/Import roundtrip; validate atomic save, path traversal, symlink escape, corrupt files, license, unique ID remapping, user document not overwritten on cancel.
9. **Transform toolbar bridging:** all 22 actions go through one owner service registry, consistent context menu/F6/command dispatch, disabled on unsupported selection; no duplicate geometry or parallel history.

## 2. Required evidence per task

`source_sha`, `tested_sha`, `commands+exit_codes`, `Rust/OCCT toolchain`, signed tolerance/fixture with units, `expected_actual_numeric_diffs`, geometry topology validity, DocumentId/FeatureId deltas, preview/commit/Undo/Redo/Save/Open, baseline MVP regression hash, screenshots/video/tester for FEBE, legal review for imported assets and release reviewer.

## 3. Fault injection and negative gates

- Invalid BRep, degenerate curves/cages, zero normal/tangent, seam/trim crossing, NaN/Inf, resource exhaustion, interrupted history update.
- Cancel mid-preview/mid-recompute → document unchanged; one user Commit = one transaction; no orphaned entities, topology naming stable.
- Crash/disk-full, bad library asset, disallowed external path → keep previous model and catalog intact; errors observable, not silent.
- Open signed G-MVP models, regress 3DM import, ring/gem/setting/cutter/STL on same fixtures. Any change needs versioned migration and reviewer approval.

## 4. Block and sign-off

Missing source/gate/manual/fixtures = `BLOCKED`. Partial UI demo = `READY_FOR_MANUAL_TEST`, not feature PASS. Geometry invalid, leaked handles, lost files, wrong units, broken MVP baseline = `FAIL`. No tolerance threshold is invented here: numerical pass conditions are specified at G00 and project-specific approved fixtures. `G-FULL` requires human reviewer sign-off; no automation should self-sign.
