# Gumball lõi — evidence 2026-10-11

Ba spec `OM9-INFO-020/021/022`; supported native slice đã triển khai.
Các representation/capability chưa hỗ trợ giữ `partially_implemented`,
`validated_supported_slice`. Smart Targets và Clayoo/SubD ngoài phạm vi user chọn.

FreeCAD có native SoTransformDragger, SoLinearDragger, SoPlanarDragger,
SoRotationDragger và camera/world-ray APIs. Chúng là primitives đã tái sử dụng;
OM9 thêm scale/origin handles, Rust interaction policy và publication adapters.
Không coi FreeCAD có đầy đủ Rhino Gumball sẵn.

Nguồn Matrix 8 Book 1 PDF 45–47 (printed 35–37), đã xem continuation tới
History ở PDF 48. Rhino 5 [Gumball Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
đã kiểm ngày 2026-10-11. Manual Y xanh lam/cyan; OM9 Y xanh lá theo spec và
thiết kế đã duyệt, không nâng visual parity do matching icon/colors.

Build chỉ OpenMatrix9Gui, shared SDK `H:/FreeCAD-src/build/relWithDebInfo`;
runtime `H:/FreeCAD-src/build/gumball-core-runtime-20261011`, cache
`build/openmatrix9-history-cage`, 14 workers. Không build root FreeCAD/copy host.
Build helper giữ per-cache mutex/configure runtime đúng; host runner kiểm
actual module path, report ok/all checks và actual exit code.

## Verification

Final results được ghi tại `H:/FreeCAD-src/build/shared-host-tests/gumball-final-20261011/`.
Full suite command:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File tests/run_gumball_smoke.ps1 -Scope All -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/gumball-final-20261011
rtk cargo test --manifest-path rust/Cargo.toml
rtk proxy python tools/public_source_audit.py
```

Core: independent OCCT staging; canonical/legacy commands; object/group/Link/
Mesh/Points ownership; lifecycle; synchronous read-only/cancel callbacks;
actual QTest pointer/editor/input at DPI 1 and 1.5: 201 checks / 8 processes.
Native picking proves all 13 paths; Move XYZ/three planes/origin, Rotate XYZ,
Scale1D/Shift3D/negative, numeric doubles, angle-step drag, Alt repeat/Copy,
Ctrl-start relocation versus held-Ctrl extrusion. Native IME commit/focus
and mouse-grab while typing CMD are checked; this does not prove every platform IME.

Persistence: warm Qt edits → save FCStd → separate process cold reopen.
14 saved objects compare independent durable facts (bounds, volume/topology,
Placement, native curve type, report JSON, unit metadata, layer/color/visibility,
groups/members, shared Link definition, Copy and retained UUIDs).
Native 3DM export after warm and cold verifies retained geometry/UUIDs independently.
Warm20/cold17 (37 total) passed; regression185 / 5 processes passed.
Regression runs existing Curve, CoreMouse, layer permission, History and 3DM
preservation with native Gumball state actually enabled in fresh profiles.

Current standalone plugin SHA256:
`8DDF092F2F406107BC20F383CF491C3796ED916A99344E995AFCBBD4300851F4`.
Split-suite total 423 checks / 15 processes, actual exits0; final combined
run recorded by Task5 verification. Full Rust806 / 115 suites; source audit
zero errors. Native checks run on the authorized dirty-source baseline;
Gumball commits intentionally exclude unrelated family edits.

## Failures that drove fixes

- Native handles initially absent; normalized arrow/arc units and camera size.
- Empty bounds cached by Coin after fitAll culled native paths; owned overlay
  disables pick/render culling. Navigation lifecycle rerun restored picking.
- Planar handles rendered too large; color-pixel render check fails old binary.
- Legacy command name borrowed a temporary string; now owns full-lifetime name.
- Copy group/layer membership assigned before native attribute inheritance.
- Same-source/second-source/Link definition callback interference aborts batch.
- Read-only gumballState inside Placement callback cancelled its own publication;
  pinned publication inputs and suppressed recapture during publication.
- Link property lookup forwarded report to shared definition; own dynamic-property
  lookup stores provenance/Copy identity on the occurrence, not the definition.
- Ctrl release before mouse-up was sticky Extrude; Rust now follows held modifier
  and native release reads the actual QMouseEvent modifiers.
- Holed Boolean face was a one-Face Shell; bounded exact-one-child unwrap
  permits native prism, preserves source/hole and negative volume orientation.
- Object alignment single-edge rotated Wire now uses native tangent, unresolved
  Mesh component retains prior alignment.
- Export fixture used conservative bbox while saved geometry used optimal bounds;
  both independent native observations now use optimalBoundingBox(False,False).

## Limits

Scale parametric/App::Link owners is UnsupportedOwner before transaction;
external-document Link needs a multi-document atomic adapter. Object alignment
Mesh components/light/camera and unresolved frames are explicit unsupported.
All pointer snap modes and all Matrix/Rhino representation parity remain open.
Single-owner pivot cache is view/session state, not FCStd state; preferences persist.
Gumball does not create new associative History links; rigid parametric native
owners retain their existing dependencies and recompute.

**Host Redo limitation:** FreeCAD `_openTransaction` clears old Redo before the
first write. `abortTransaction` rolls back model/Undo but does not restore that
old Redo queue after synchronous callback interference. Native report records
RedoBefore=1, RedoAfter=0. Pretransaction stale/input/capability rejects preserve
Redo. No private ABI access or root FreeCAD rebuild was introduced to hide this.

Tolerance is explicit: typed values 1e-10 mm, box dimensions/volume 1e-8,
ellipse optimal bounds 1e-6 mm (OCCT native edge tolerance 1e-7 each side).
Geometry is native BRep/Mesh/Points; conic under affine may be rational BSpline,
which is preserved and observed, not replaced by a tessellation.

## Review and decisions

One fresh final reviewer and rulings ledger are pending. Final proof/ledger
will be retained outside transient `.superpowers` workspace before cleanup.
[Gumball](../features/OM9-INFO-020.md), [Alignment](../features/OM9-INFO-021.md),
[Relocate](../features/OM9-INFO-022.md) separate source statements from OM9 decisions.
