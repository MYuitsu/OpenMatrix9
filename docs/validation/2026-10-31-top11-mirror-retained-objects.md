# Top11 Mirror: Hatch, Light and retained native BRep — 2026-10-31

Status: partial automated evidence; full Top11 and physical M03-UI remain open.

Mirror now admits Hatch and Light selections and mirrors their placements. It also mirrors the placement of a NativeBrep, preserving its opaque geometry payload rather than converting display triangles into exact geometry. This uses the same world reflection and atomic placement/copy path as block instances. Geometry and attributes are retained unchanged; their world interpretation follows the reflected placement.

A controller fixture tests each kind separately through CMD and toolbar with Copy=Yes and Copy=No. It checks reflected placement, preserved geometry and attributes, Light location and direction after world transformation, scene publication and pick, .om9 roundtrip, Undo and Redo. The NativeBrep fixture deliberately uses a synthetic nonempty byte payload and a display triangle; it proves byte preservation and scene behavior only, not validity or 3DM roundtrip of an actual native BRep. The Light test proves its stored geometric transformation, not the output of a lighting renderer. These are controller and scene picks, not physical viewport interaction.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 61 selected core tests, complete scene suite 112 tests, app Top11/options/BRep/snapshot 198 tests and no-default Top11/options 113 tests passed. The final focused fixture including scene picks also passed on the final test source. No-default emits existing unrelated warnings. No native code or openNURBS measurement was performed.

Actual retained BRep and Hatch/Light 3DM export, lighting behavior, other transforms and physical M03-UI remain unverified. No completion claim is made for the entire Top11.

Source SHA-256:

- `crates/om9-core/src/top11.rs`: `5ac01fdcc7517303473ffc456eb91c97c76aaf70de6dba1d9ac126cc5cdd4764`
- `crates/om9-app/src/session/top11.rs`: `df15d2d0a1f25e90181cb410c22aca4d4f1eb4da65fae85bc8a43e70ae7639c3`
- `crates/om9-app/tests/top11.rs`: `14c82d92a03da122c1e18e81b8c58b27a5eee84a939c0fd3d66ac9ceb18741b3`
