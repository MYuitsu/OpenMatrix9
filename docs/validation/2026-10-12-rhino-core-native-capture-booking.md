# RCORE-01 — Strong journal capture and booking policy

Native journal capture đã chuẩn bị metadata/copy trước khi ghi entry; **34/34** kiểm tra, gồm first/later/removal Copy failure và retry.
Regular booking không khôi phục Redo đã loại bỏ sau accepted replay/abort; **80/80** so sánh hành vi FreeCAD native ở0/1,plain/booked/active/preserving.
Runtime138/App141 đạt lại **432 semantic +4 owner-close +109 modes +33 Curve**; tổng **692 assertion/6 host0**;5 host sạch, capture34 có initial macro deadlock.
String scope744 vẫn có deadlock lúc xử lý macro khởi động, **không phải bằng chứng lỗi shutdown**. Retirement callbacks, global participants/budgets và full RCORE-01/Task2/3 còn mở.

Native REDcapture13431/34, booking14077/80 and timed-out capture133 are preserved. Isolated App135/App141 builds reuse shared SDK objects, without root reconfigure or replacing SDK DLL/lib/PDB. Runtime138 imports App135; App141 changes only the private PIMPL implementation header, with identical public headers. Matching loaded App/Gui/plugin fingerprints are checked; this is not a complete ABI certificate. String-scope139744/744 has unhandled resource deadlock before “Loading OpenMatrix9 GUI”, followed by a normal shutdown. The previous shutdown attribution is corrected; cause remains open.

Remaining requirements:

- Native journal capture now stages metadata/copy and object records before publishing. Named first/later/removal Copy exceptions and retry first-write behavior pass34. App135 capture136 is clean; App141 capture143 again logs initial macro/workbench resource deadlock despite all34 assertions and host0. Complete dynamic removal, null capture, allocation failures, rename/move capture and native status/cache semantics are not certified by this cohort.
- Named native comparison across plain/booked/active/preserving, UndoMode0/1 and commit/abort/Undo/Redo passes80. Regular accepted first-write now discards prior Redo; preserving booking retains its existing policy. Global/multidocument descriptors and participants, signalOpen callback state and every native history API remain open.
- Commit journal retirement and private property destruction callbacks still occur after transfer/completion; latch rejection, reversible transfer and complete lifecycle protection through release require further native tests and implementation. Tasks2/3 and complete savepoint acceptance remain open.
- Native String scope744 passes assertions but logs unhandled resource deadlock during initial macro/workbench handling. Earlier checkpoints described this as shutdown; current startup.log orders it before OpenMatrix9 native load, while host shutdown is normal. The exact cause is unresolved. Double OM9 workbench registration also appears in shared-host initialization; four named Circle/Curve hosts have no unhandled/deadlock but that does not certify all startup paths.
- All original RCORE01 requirements remain authoritative: adapters1-7,C01-C12,T01-T11; stable refs/world transforms/owned geometry-topology-attributes-units; all rational/periodic curves/surfaces/UV trim/pcurve/seam families; deviation/locus/parameter maps; operation/representation/options/backend registry; GeometryResult for all enabled consumers; worker ownership and aggregate allocation/traversal/cancellation; checked publication/History/cache/locks/lifetime; solid validity/manifold/cavities; independent read/edit/retain/export; generic Snap/mouse/render/persistence/lifecycle. No original scope is reduced.
- Rust suite was not rerun in this checkpoint. Original raw native offset-D3 discrepancy remains RED and unchanged; plugin build source candidates are not a full compiler dependency, ABI or peak-allocation certificate.

Next: Add native journal retirement/destruction callback witnesses and reversible acceptance where needed; audit exact descriptors/multidocument participants and budgets. Diagnose initial macro/workbench deadlock with its real phase, then continue complete original RCORE01 acceptance.

[Machine evidence](2026-10-12-rhino-core-native-capture-booking.json).
