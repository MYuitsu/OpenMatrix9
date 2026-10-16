# M09 — Hai release gates, không phát sinh L2 owner

## G-MVP — core jewelry CAD

**Prereqs:** G00, E01–E12 PASS; tất cả blocking severity đã đóng; geometry tests / cross-workflow tests PASS.

- [ ] App native, 4 viewports, menu/snap/selection/docking, Save/Open.
- [ ] Core Curve/Surface/Solid + geometry validity; undo/cancel/redo stable.
- [ ] Import .3dm được kiểm chứng đúng phạm vi, unsupported data warning; không ngầm yêu cầu export .3dm.
- [ ] Ring/Profile History; placement Gem/Pavé; Head/Bezel/Prong; Cutters.
- [ ] STL export đúng mm/manifold khi fixture kỳ vọng; kiểm external viewer.
- [ ] Release evidence: source/build SHA, tests, fixture provenance, OS/GPU, license, signoff và limitation công khai.

## G-FULL — advanced jewelry CAD (không phải toàn bộ Matrix 9)

**Prereqs:** G-MVP đã ký + E13 PASS.

- [ ] SmartFlow and rigid placement, deformation, ornament builders.
- [ ] Smart Pattern, Rope, user library and file roundtrip.
- [ ] MVP regression unchanged; no silent API duplicates.
- [ ] Advanced known gaps công khai: Clayoo/SubD/T-Splines/Emboss/Render/CAM nằm AFTER.

**Không gọi hai gates này là đã PASS chỉ vì tài liệu hiện diện.** Trạng thái lúc tạo plan: NOT_TESTED.

## Kế hoạch L4 kiểm định M09

Chi tiết [M09/README.md](M09/README.md): 18 L4 cho G-MVP và 12 L4 cho G-FULL, các ca kiểm thử, DAG, biểu mẫu evidence và prompt GPT-6 Luna. Không tạo module owner L2, không thay thế signed acceptance thực tế.
