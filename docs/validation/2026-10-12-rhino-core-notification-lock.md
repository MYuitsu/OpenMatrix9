# RCORE-01 — Reentrant notification logging

Đã sửa startup resource-deadlock tại FreeCAD NotificationArea: dựng nội dung dưới mutex rồi thả khóa trước khi mở popup; cảnh báo Qt có thể vào logger bình thường. Popup thật vẫn giữ thông báo và warning, không tắt thông báo để né lỗi.
**App273/plugin275/GUI287/kernel240:3375/3375 kiểm tra qua35 host sạch log, tất cả exit0.** Fixture popup RED2/3→GREEN3/3, stack native xác nhận nguyên nhân. Giữ các lỗi startup274/278/284 và các crash cũ làm evidence lịch sử. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi GUI native này.
**RCORE-01 vẫn đang triển khai:** còn các phần chưa có evidence đầy đủ về conversion, consumer/result, ownership/preservation, aggregate bounds và lifecycle/cache; T01-T11 đã có accepted slices, không mở lại toàn bộ chỉ vì checkbox lịch sử.

Native stack284 shows showInNotificationArea holding mutexNotification while NotificationBox::showText calls QWidget::raise. The offscreen platform emits a warning synchronously, Console forwards it to NotificationAreaObserver, and pushNotification attempts the same mutex, causing std::_Throw_Cpp_error. The fix reuses std::unique_lock and the existing owned QString; it unlocks before entering the window-show path.

A controlled fixture enables the existing notification preferences and records actual C++ exception stacks. The unchanged native GUI reports2/3 and resource-deadlock exceptions; corrected GUI reports3/3, zero matching exceptions, and the actual popup text retains the seeded message and Qt warning. Independent source review found no new defect in this branch.

Only NotificationArea.cpp is compiled. The accepted camera GUI object/link inputs, existing App273/plugin275 and SDK dependencies remain in use. Shared binaries are fingerprinted unchanged. All35 fresh matching hosts pass the clean-log gate, including actual Circle recipe/legacy, warm/cold FCStd, snapshots, native retirement, Python finalizers and the popup witness.

Remaining:

- The constructor now bounds aggregate owner-history/property records and logical snapshot/name storage before copies. Simultaneous journal/recovery/merge/inverse peak storage, opaque custom values/closures, allocator metadata and all native status/cache identities remain outside this proved bound.
- History destruction reentrancy is verified for regular native Undo/Redo cleanup and visible/held preserving-group disposal. Other derived-object/member destruction, global/source-participant callbacks and worker cancellation/close boundaries still need targeted lifecycle evidence; a single history fix does not certify every FreeCAD destructor.
- Continue the original representation/conversion/ownership/result-routing requirements with existing accepted fixtures. Historical unchecked rows are not automatically new failures: reconcile C01-C12/T01-T11 against their accepted slices, then fill missing families, maps, provenance, channels/attributes, enabled consumers and structural preservation.
- Original RCORE01 acceptance gaps have been reconciled separately. T01-T11 and several C contracts have accepted slices; their historical rows are not blanket failures. Six remaining groups concern complete conversion/correspondence, consumer/results/diagnostics, identity/ownership/attributes, preservation matrices, aggregate bounds/cancellation, and publication/lifetime/cache coverage.

Next: Continue remaining native derived-object/member lifecycle witnesses and aggregate preparation budgets, then mesh/special Edit GeometryResult consumers using existing FreeCAD ownership and prepared discrete snapshots.

[Machine evidence](2026-10-12-rhino-core-notification-lock.json).
