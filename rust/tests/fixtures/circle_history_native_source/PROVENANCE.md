# Circle History native-source diagnostic data

SPDX-License-Identifier: LGPL-2.1-or-later

These requests are reconstructed from authored synthetic FreeCAD macro output,
not intercepted C++→Rust packets. Runtime59 saved `Circle014` (deformable
AroundCurve), `Circle015` (bounded native Tangent), and `Circle016` (fixed first
contact and point dependency) in the all-mode fixture. Output, validation,
identity and operation context are retained; source_snapshots are reconstructed
from the owned input_snapshots indices without fabricating native geometry.
Complete replacement line/point sources come from the same synthetic document.

Original evidence: `build/solid-regression-circle-history-modes60-retry-offscreen-20261011/recorded-results.json`
in the FreeCAD workspace, described in
`docs/validation/2026-10-12-rhino-core-circle-history-modes.md`.
The authored fixture is `tests/rhino_core_circle_history_modes_result_smoke.FCMacro`.
The machine provenance receipt records exact file hashes.

Rust acceptance of these payloads does not prove independent BREP decoding,
kernel basis/master agreement, geometry conversion or all source families.
Negative tests isolate missing source/parameter/constraint correspondence.

The first complete replacement point has document token2 while the tangent
request uses token1. Its rejection proves that identity gate, not the circle
constraint. `replacement_point_same_document.json` uses the genuine CircleSide
point(0,10,0), token1/epoch3, without fabricating stamps. The complete initial
RTK output and current corrected test source before run65 are preserved in the
workspace red64 diagnostic archive; no frozen initial compiler-source hash
was captured, so the current source is not claimed as an exact red64 producer.
