# Solid menu validation

All 21 bindings continue to dispatch through the existing Rust Controller and OCCT owners.

Ellipsoid correction (2026-10-23): safe Rust constructs a rational biquadratic 9×5
net; the native adapter builds one closed B-Rep solid. Geometry thresholds, serialized
source reload, menu/CMD, Undo/Redo and `.om9` persistence pass. The native Windows
Ellipsoid workflow was observed in four GPU views, including invalid input and cancel.

[Measured report](../../../../docs/validation/2026-10-23-exact-ellipsoid.md) ·
[Receipt](../../../evidence/M03/ellipsoid-exact-2026-10-23/receipt.json) ·
[UI review awaiting human signoff](../../../evidence/M03/ellipsoid-exact-2026-10-23/ui-review.json)

The other twenty Solid UI workflows are not newly certified by this Ellipsoid run.
M03-UI remains pending until a human reviewer records acceptance.

## Solid follow-up — 2026-10-23

Exact curve extrusion and one-transaction ExtrudeAll, Text/Boolean menu prompts, and span-aware volume integration are verified. Kernel 71 passed; app 191 passed; native Text/Boolean/ExtrudeAll saved files cold-measured. The earlier Ellipsoid-only checkpoint above is historical.

[Latest report](../../../../docs/validation/2026-10-23-solid-ui-followup.md) · [Human worksheet](../../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/M03-UI-CHECKLIST.md). M03-UI remains unsigned.
