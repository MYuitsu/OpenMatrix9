# OM9_Milestones — ánh xạ tên thư mục sau khi rút ngắn đường dẫn

- Tạo: 2026-10-16T03:52:29Z | ngân sách: mọi đường dẫn ≤ **210** ký tự.
- Lý do: giới hạn MAX_PATH 260 của Windows khi cây bị sao chép vào tiền tố dài hơn (staging `build/...`, worktree agent, thư mục tạm).
- **Không mất thông tin:** tiêu đề đầy đủ của mỗi cấp vẫn nằm trong `L1.json`, `L2.json`, `l3.json`, `l4.json` tương ứng; bảng dưới chỉ là ánh xạ tên thư mục.
- `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment` chua doi duoc trong phien nay (thu muc bi giu handle); ten du kien: `M00 - Feasibility, Legal`. Hoan tat sau: `python implementation/tools/om9_shorten_milestone_paths.py --finish-module "M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment=M00 - Feasibility, Legal"`.
- `M03 - Curve, Surface, Solid and 3DM Exchange` chua doi duoc trong phien nay (thu muc bi giu handle); ten du kien: `M03 - Geometry 3DM`. Hoan tat sau: `python implementation/tools/om9_shorten_milestone_paths.py --finish-module "M03 - Curve, Surface, Solid and 3DM Exchange=M03 - Geometry 3DM"`.

## Thư mục module

| Trước | Sau | Trạng thái |
|---|---|---|
| `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment` | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment` | giữ nguyên |
| `M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport Foundation` | `M01 - CAD Shell, Viewport` | đã đổi |
| `M02 - Viewport Picking, Snapping, Navigation, Gumball and Matrix Interaction` | `M02 - Viewport, Gumball` | đã đổi |
| `M03 - Curve, Surface, Solid and 3DM Exchange` | `M03 - Curve, Surface, Solid and 3DM Exchange` | giữ nguyên |
| `M04 - Builder, History and Ring` | `M04 - Builder and History` | đã đổi |
| `M07 - Cutters and Manufacturing Validation` | `M07 - Cutters` | đã đổi |
| `M08 - Advanced Jewelry Modeling` | `M08 - Jewelry Modeling` | đã đổi |
| `ZAFTER - Post-Release Extensions` | `ZAFTER - Extensions` | đã đổi |

## Thư mục L2/L3/L4 đã đổi tên

| Cấp | Đường dẫn cũ | Tên mới | ID | Tiêu đề đầy đủ |
|---|---|---|---|---|
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-01 - QymCAD GitHub & source reuse` | `L2-01 - QymCAD` | OM9-M00-L2-01 | QymCAD GitHub & source reuse |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-01 - QymCAD GitHub & source reuse/L3-01 - Repository provenance & pinned checkout` | `L3-01 - Repository provenance & pinned` | OM9-M00-L3-01-01 | Repository provenance & pinned checkout |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-01 - QymCAD GitHub & source reuse/L3-02 - Workspace map and dependency audit` | `L3-02 - Workspace map` | OM9-M00-L3-01-02 | Workspace map and dependency audit |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-01 - QymCAD GitHub & source reuse/L3-03 - REUSE ADAPT ADD architecture decision` | `L3-03 - REUSE ADAPT ADD architecture` | OM9-M00-L3-01-03 | REUSE ADAPT ADD architecture decision |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-02 - OCCT bridge & numerical geometry` | `L2-02 - OCCT` | OM9-M00-L2-02 | OCCT bridge & numerical geometry |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-03 - UI - Viewport - GPU technology` | `L2-03 - UI` | OM9-M00-L2-03 | UI/Viewport/GPU technology |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-04 - 3DM - openNURBS interoperability` | `L2-04 - 3DM` | OM9-M00-L2-04 | 3DM / openNURBS interoperability |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-04 - 3DM - openNURBS interoperability/L3-01 - 3DM capability gap and dependency candidate` | `L3-01 - 3DM capability gap and dependency` | OM9-M00-L3-04-01 | 3DM capability gap and dependency candidate |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-04 - 3DM - openNURBS interoperability/L3-03 - Import security and transparent limitations` | `L3-03 - Import security and transparent` | OM9-M00-L3-04-03 | Import security and transparent limitations |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-05 - Associative history & stable topology` | `L2-05 - Associative` | OM9-M00-L2-05 | Associative history & stable topology |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-05 - Associative history & stable topology/L3-01 - Naming and reference architecture audit` | `L3-01 - Naming and reference architecture` | OM9-M00-L3-05-01 | Naming and reference architecture audit |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-05 - Associative history & stable topology/L3-02 - Regeneration and persistence stress proof` | `L3-02 - Regeneration and persistence` | OM9-M00-L3-05-02 | Regeneration and persistence stress proof |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-06 - License - redistribution compliance` | `L2-06 - License` | OM9-M00-L2-06 | License / redistribution compliance |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-06 - License - redistribution compliance/L3-01 - Software license and reuse strategy` | `L3-01 - Software license and reuse` | OM9-M00-L3-06-01 | Software license and reuse strategy |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-06 - License - redistribution compliance/L3-02 - Assets, trademarks and release notices` | `L3-02 - Assets, trademarks and release` | OM9-M00-L3-06-02 | Assets, trademarks and release notices |
| L2 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-07 - Reproducible build, CI and release baseline` | `L2-07 - Reproducible` | OM9-M00-L2-07 | Reproducible build, CI and release baseline |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-07 - Reproducible build, CI and release baseline/L3-01 - Windows 10 - 11 x64 baseline` | `L3-01 - Windows 10 - 11 x64` | OM9-M00-L3-07-01 | Windows 10/11 x64 baseline |
| L3 | `M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L2-07 - Reproducible build, CI and release baseline/L3-03 - Continuous verification and gate handoff` | `L3-03 - Continuous verification and gate` | OM9-M00-L3-07-03 | Continuous verification and gate handoff |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-09 - 01-be-occt-geometry-creation-ops` | `L2-01-09 - 01-be-occt-geometry` | OM9-L2-01-09 | 01-be-occt-geometry-creation-ops |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-002 - B-Rep Boolean Difference` | `L3-002 - B-Rep Boolean` | OM9-L3-01-10-002 | B-Rep Boolean Difference |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-003 - B-Rep Boolean Intersection` | `L3-003 - B-Rep Boolean` | OM9-L3-01-10-003 | B-Rep Boolean Intersection |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-004 - Curve And Face Split Intersections` | `L3-004 - Curve And Face Split` | OM9-L3-01-10-004 | Curve And Face Split Intersections |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-006 - Fillet And Chamfer B-Rep Edges` | `L3-006 - Fillet And Chamfer B-Rep` | OM9-L3-01-10-006 | Fillet And Chamfer B-Rep Edges |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-007 - Offset Face And Thicken Shell` | `L3-007 - Offset Face And Thicken` | OM9-L3-01-10-007 | Offset Face And Thicken Shell |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-008 - Face Extract Unjoin And Rejoin` | `L3-008 - Face Extract Unjoin` | OM9-L3-01-10-008 | Face Extract Unjoin And Rejoin |
| L3 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-009 - Geometry Health And Naked Edge Query` | `L3-009 - Geometry Health` | OM9-L3-01-10-009 | Geometry Health And Naked Edge Query |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-01-11 - 03-be-occt-transforms-and-pattern-ops` | `L2-01-11 - 03-be-occt` | OM9-L2-01-11 | 03-be-occt-transforms-and-pattern-ops |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-02-04 - 04-febe-freeform-curve-modeling` | `L2-02-04 - 04` | OM9-L2-02-04 | 04-febe-freeform-curve-modeling |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-02-05 - 05-febe-freeform-surface-modeling` | `L2-02-05 - 05` | OM9-L2-02-05 | 05-febe-freeform-surface-modeling |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-02-06 - 06-febe-rhino-transform-orient-workflows` | `L2-02-06 - 06-febe-rhino` | OM9-L2-02-06 | 06-febe-rhino-transform-orient-workflows |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-08 - 07-febe-matrix-top11-and-shortcut-actions` | `L2-03-08 - 07-febe-matrix-top11` | OM9-L2-03-08 | 07-febe-matrix-top11-and-shortcut-actions |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-16 - 08-febe-matrix-curve-menu-workflows` | `L2-03-16 - 08` | OM9-L2-03-16 | 08-febe-matrix-curve-menu-workflows |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-17 - 09-febe-matrix-surface-menu-workflows` | `L2-03-17 - 09-febe` | OM9-L2-03-17 | 09-febe-matrix-surface-menu-workflows |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-18 - 10-febe-matrix-solid-menu-workflows` | `L2-03-18 - 10-febe` | OM9-L2-03-18 | 10-febe-matrix-solid-menu-workflows |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-19 - 11-febe-matrix-utilities-and-mesh-health` | `L2-03-19 - 11-febe-matrix` | OM9-L2-03-19 | 11-febe-matrix-utilities-and-mesh-health |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-03-20 - 12-febe-matrix-measure-and-model-statistics` | `L2-03-20 - 12-febe-matrix` | OM9-L2-03-20 | 12-febe-matrix-measure-and-model-statistics |
| L2 | `M03 - Curve, Surface, Solid and 3DM Exchange/L2-04-01 - 13-febe-matrix-transform-toolbar-bridges` | `L2-04-01 - 13-febe` | OM9-L2-04-01 | 13-febe-matrix-transform-toolbar-bridges |
| L2 | `M04 - Builder, History and Ring/L2-03-13 - 04-febe-matrix-history-associative-updates` | `L2-03-13 - 04-febe-matrix-history` | OM9-L2-03-13 | 04-febe-matrix-history-associative-updates |
| L2 | `M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster-baguette-layouts` | `L2-04-11 - 03-febe-gem-halo-cluster` | OM9-L2-04-11 | 03-febe-gem-halo-cluster-baguette-layouts |
| L2 | `M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead-settings` | `L2-04-15 - 02-febe-prong-channel-metal-and-bead` | OM9-L2-04-15 | 02-febe-prong-channel-metal-and-bead-settings |
| L2 | `M08 - Advanced Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement-tools` | `L2-04-07 - 04-febe-jewelry-surface-object-placement` | OM9-L2-04-07 | 04-febe-jewelry-surface-object-placement-tools |
| L2 | `ZAFTER - Post-Release Extensions/L2-08-01 - 01-be-shared-offline-render-material-light-pipeline` | `L2-08-01 - 01-be-shared-offline-render` | OM9-L2-08-01 | 01-be-shared-offline-render-material-light-pipeline |
| L2 | `ZAFTER - Post-Release Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light-rigs` | `L2-08-02 - 02-febe-jewelry-render-lookdev-and-light` | OM9-L2-08-02 | 02-febe-jewelry-render-lookdev-and-light-rigs |
| L2 | `ZAFTER - Post-Release Extensions/L2-08-04 - 04-febe-jewelry-batch-render-and-postprocessing` | `L2-08-04 - 04-febe-jewelry-batch-render` | OM9-L2-08-04 | 04-febe-jewelry-batch-render-and-postprocessing |
| L2 | `ZAFTER - Post-Release Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command-compat` | `L2-09-01 - 01-febe-tsplines-legacy-hud-command` | OM9-L2-09-01 | 01-febe-tsplines-legacy-hud-command-compat |
| L2 | `ZAFTER - Post-Release Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder-compat` | `L2-09-02 - 02-febe-tsplines-legacy-primitive-builder` | OM9-L2-09-02 | 02-febe-tsplines-legacy-primitive-builder-compat |
| L2 | `ZAFTER - Post-Release Extensions/L2-09-12 - 12-febe-clayoo-selection-set-operations` | `L2-09-12 - 12-febe-clayoo-selection-set` | OM9-L2-09-12 | 12-febe-clayoo-selection-set-operations |
| L2 | `ZAFTER - Post-Release Extensions/L2-10-03 - 03-febe-jewelry-print-supports-and-sprues` | `L2-10-03 - 03-febe-jewelry-print-supports` | OM9-L2-10-03 | 03-febe-jewelry-print-supports-and-sprues |
