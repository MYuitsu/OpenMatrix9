# RCORE-01 — saved Surface History admission

Status: scoped native validation; **full RCORE-01 remains in progress**.
Recorded 2026-10-11T07:13:51+07:00. [Evidence receipt](2026-10-11-rhino-core-surface-history-admission.json).

## Implemented and reused

Rust reads independent saved options/group metadata with the existing owned JSON
parser and native Surface capability registry. It checks bytes1MiB, nodes16384,
depth32, groups256/count4096, slashes64 and the existing number/bool/schema and
logical input cardinality constraints before native source capture. Unknown
bounded fields stay compatible; duplicate/trailing/invalid data rejects.

A fixed caller-owned scalar ABI transfers admitted options/groups/slashes only.
No raw document/kernel pointer survives the synchronous call; caller guarantees
live, initialized disjoint byte/output storage. Native adapter resolves actual
source objects and keeps FreeCAD/OCCT Loft/Sweep solvers. Saved closure hints are
not current native facts: Stage::Saved defers closure eligibility to existing
Stage::Construct with actual native wires. No solver was replaced.

Malformed saved metadata reports InvalidBasis, Surface History and the child name,
clears stale Shape and retains raw recipe/source links/History and executable Redo.
Repair reuses the existing native solver. This does not certify every native error
path or every family's ownership/transaction behavior.

## Verification

- Original native RED on observed26E606 plugin: 11/16 pass, five failures, actual
  host exit0. Bytes/depth reached Qt/native parsing; malformed child context missing.
- Final admission GREEN55 includes nine invalid JSON/schema/budget cases,
  source/raw/link/Undo/Redo/repair checks, bounded unknown fields, closed Sweep1/2
  stale false hints and open slashed Sweep2 stale true hints.
- Rail compatibility portable RED failed as intended; native RED52 had two
  failures after valid closed Sweep baselines. Both mismatch paths pass now.
- A preliminary embedded-NUL native test failed because Python PropertyString
  conversion truncated the value before the reader. It was removed from reader
  rejection claims; borrowed Rust ABI tests explicitly reject embedded NUL and
  invalid UTF-8. Preserved green1/red4 artifacts show that distinction.
- The first cold Sweep fixture failed because its old branch moved the profile
  using the Loft-specific edit. Cold fixtures now grow Sweep rails (both for
  Sweep2) and retain the independent expected area ratio5/3. Repaired fixtures
  and fresh matching warm models are used for all three cold runs.
- Existing native diagnostic smoke caught missing Loft/source context (4/6
  pass, actual host exit0). Complete rejected requests now retain operation and
  actual linked identity; original assertions pass on the final plugin.
- Full Rust 767 tests/109 nonempty suites, actual exit0, including the
  nine saved-reader tests and existing native Surface policy truth tables.
- Standalone plugin build3 actual exit0; shared FreeCAD SDK reused, no root build.
  Existing OCCT deprecated-header and Rust path/cache warnings remain diagnostic.
- Independent review found and then confirmed the rail hint compatibility fix;
  no other important source finding. See [review](../reviews/2026-10-11-surface-history-admission-review.md).

| Matching native run | Checks | Actual host exit |
|---|---:|---:|
| green1 | 55 | 0 |
| commands1 | 97 | 0 |
| options1 | 180 | 0 |
| advanced1 | 108 | 0 |
| constraints1 | 85 | 0 |
| history1 | 65 | 0 |
| cold-sweep1 | 3 | 0 |
| cold-sweep2 | 3 | 0 |
| cold-loft1 | 3 | 0 |
| policy1 | 29 | 0 |
| diagnostics1 | 6 | 0 |
| builder1 | 17 | 0 |
| prepared1 | 47 | 0 |

Total 698 checks, 13 matching actual host exits0, plugin SHA256
`6534bf47b9336fbfff10538ddbaf2fbcef557b134c4bf4fe46ebfc69c2416d98`. Fresh-process Sweep1/Sweep2/Loft restore uses models made by
the matching warm History run. Source/runtime/fixture/report/log fingerprints are
in the receipt. Earlier candidate hashes and unrelated concurrent changes are
excluded from this scoped source-completion claim.

## Remaining original scope

- Original global conversion/deviation bounds, full-domain parameter correspondence and all-family GeometryResult consumers.
- Uniform attributes/preservation, native source identity/lock/revision/generation, cancellation/cache/transaction contracts across all enabled entry points.
- Original representation families and adapters1–7, C01–C12, T01–T11 final acceptance, including aggregate native/worker budgets.
- Standalone raw JSON construction/serialization and unrelated legacy native readers are outside this bounded saved-reader checkpoint.
