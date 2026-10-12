# RCORE-01 — Checked native retirement verdict

App227/Runtime229 dùng lại transaction, native Property và Python exception của FreeCAD. Finalizer chạy trước verdict của commit/cleanup; callback bị chặn không còn báo thành công. Giữ inverse nếu retirement trước đó đã bị từ chối.
**3145/3145 assertion qua28 host** trên runtime tương ứng, gồm Python identity/private inverse72/201, finalizer6, frozen source180, native journal54, verdict50, cleanup14 và inverse retention12.
**RCORE-01/Task2/3 vẫn đang triển khai.** Test gốc Python final release 5/8 và detached object 16/32 còn RED; Rust121 đạt/1 lỗi OffsetBezier D3. Các checkpoint phía dưới là bằng chứng lịch sử.

Sửa C++ giới hạn ở API private/native của FreeCAD: journal, ownership, observer barrier và chuyển exception qua API Python. Portable policy/data tiếp tục thuộc Rust; macro Python chỉ là fixture. Không reconfigure/build FreeCAD root, không thay shared DLL hay clone runtime; chỉ build object App cần thiết với DLL/import/PDB riêng và standalone OpenMatrix9Gui trên cache/SDK hiện hữu.

Baseline thật: release-red2134/8; release-verdict-red2154/10 rồi host exit-1073740791 khi finalizer gọi closeDocument; cleanup-verdict-red21910/12; retirement-inverse-red2268/12. Test gốc identity/rollback giữ nguyên. KernelFailure/unverified cleanup là trạng thái lỗi còn mở, không phải hoàn tất atomic contract. Normal PyObject identity/Undo/Redo và frozen-source composition nay được kiểm lại trên cùng App227/Runtime229. Startup vẫn có cảnh báo duplicate workbench registration; native hashes không chứng nhận toàn bộ ABI, allocator hay memory safety.

Các yêu cầu còn mở:

- Original Python final-release identity/checked rollback witness remains RED: 5/8. A finalizer veto is now observed before the commit verdict, but a physically released sole-reference Python value cannot be restored by this inverse. No full savepoint acceptance.
- Original detached native object disposal remains RED: 16/32. Usable property inverse is retained after an earlier veto; native object identity, original booking/Redo and complete paired cleanup remain unverified after actual object destruction.
- Full current Rust suite remains RED121pass/1fail, exit101: raw native OffsetBezier D3 comparison in support_snapshot::evaluation_tests::owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families.
- Complete native value admission remains open for custom/nonpersistent/opaque properties, Python/validator/expression payload retirement callbacks, topology/shape identity, status/touch/cache, factory/Copy/Paste/Save/Restore/allocation failures and shared native dependencies.
- Aggregate/peak allocation and traversal preflight remains open: constructor copies history/names before complete bounded admission; expression getMemSize reports0 and estimated text/map accounting is not a complete retained-tree or closure allocation certificate. Source/property guard counts are not full bounded-worker proof.
- All global/multidocument/source participants, locks and before-mutation controls, old move/rename histories, all cleanup/open/restore callbacks, GUI/startup paths and persisted caches require complete acceptance. Selected startup still reports duplicate workbench registration; earlier delayed-import deadlocks remain preserved, not solved by early import.
- Every original RCORE01 adapter1-7,C01-C12,T01-T11 stays open until its full audit: refs/world-once/ownership; curve/surface/trim/pcurve/seam families; conversion/deviation/locus/maps; registry/GeometryResult all consumers; workers/aggregate bounds/cancel; atomic publication/History/cache/locks/lifetime; valid oriented manifold solids/cavities; read/edit/retain/export; generic Snap/mouse/render/persistence/lifecycle. Component passes do not prove full RCORE01.
- Native source190/build/loaded hashes and selected SDK/import do not establish a complete ABI/compiler dependency/native memory-safety certificate. Plugin full prebuild source inventory was not frozen; no production host replacement or publish is claimed.

Tiếp theo: Resolve native final-disposal recovery without weakening the original identity witnesses; investigate raw native OffsetBezier D3, aggregate admission/global participants, then continue every original RCORE01 adapter and C01-C12/T01-T11.

[Bằng chứng máy](2026-10-12-rhino-core-native-retirement-verdict.json).
