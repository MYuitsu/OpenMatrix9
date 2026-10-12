# Rhino core P0 — shared point consumers — 2026-10-09

Status: **validated bounded increment; all P0 remains in progress**. Parent `e6e08e5`.

Distance, Angle and PictureFrame now share the Rust Cartesian point/length resolver.
Bare coordinates use the captured CPlane; world/relative forms remain explicitly
tagged. Unsuffixed values use captured document scale; mm/cm/m/in/ft suffixes
convert exactly once. World mouse/snap/F4 input bypasses both conversions.
Distance Unit= controls output only and still defaults to mm in this increment.

Owned input records retain exact original text (including surrounding spaces),
accepted world coordinates or scalar length, previous accepted point, actual
document owner/UID, logical view slot, frame revision/basis and unit context.
Grammar/locale are explicitly cartesian_v1/dot_decimal. Snap identity is marked
unavailable. Frame is latched at command start, before the PictureFrame image
dialog. Unit/document changes reject the old session before acceptance/commit.

Measurements publish a UI report and world-coordinate transcript without document
mutation. Invalid Angle endpoints do not advance the relative base. PictureFrame
stores its tagged inputs and unit context in the image creation transaction.
Rejected transaction start is checked before mutation and retains the corner for
retry. Undo/Redo and a fresh-process FCStd reopen preserve exact provenance;
the embedded image survives deletion of the external test fixture.

Review exposed three defects with native RED evidence: loose CPlane rigidity,
ignored transaction-start failure, and router whitespace loss. They are repaired
and included in the final runs. Frame::new now rejects dimensionless Gram error
above 64*f64::EPSILON (~1.42e-14), instead of silently accepting scale/shear up to
1e-6. It never normalizes imported axes. 4096 independent quaternion samples and
arbitrary rotation/three-point tests retain ordinary valid frames; old rounded
saved/imported frames outside the bound now reject explicitly.

## Verification

- Matching SDK build passed; final native SHA256 `c59095ab83fa786ddee25300fae0f75263f11d1c06daa99907c2b329c10f05bd`.
- Full Rust: 364 passed; Clippy exit 0 (existing warnings); owned Rust files formatted.
- Python: 49 passed, 1 skipped. Source audit: 2054 files, zero errors.
- Native: 324 passed assertions in nine exit-0 processes; each macro/module hash verified.
- Spec manifest: 664 entries; JSON/YAML equivalent; only three exact feature rows promoted to a partial validated slice.

| Suite | Assertions | Directory in build/merge_replay-1 |
| --- | ---: | --- |
| shared-input | 33 | `d22e08b99eb04154bdf399a0d9678d9c` |
| shared-input-cold | 4 | `66b6730ef65d4236bbf82a0aaf1fdb3e` |
| distance | 28 | `4f0022941fc54705a955b93475387d44` |
| angle | 19 | `b88212a0e4c64f2999d932c568e16458` |
| picture-frame | 47 | `1240cd70aff24826b7669eaae99d614c` |
| invalid-cplane | 14 | `acc78cb4430743adad553f6f4e3bde12` |
| snap-point-tools | 7 | `894d2de6312b4587b1a202ff803703c6` |
| snap-lifecycle | 10 | `8d823ca1b03946bb824af166973884d9` |
| coordinates | 162 | `0f582c3ac0e541e4a666529604598d00` |

Failed baselines and superseded green runs are excluded from this total. Exact
hashes, outputs and limits are in [machine evidence](2026-10-09-rhino-core-point-consumers-evidence.json).
Independent reviews and their repair dispositions are recorded there. No arbitrary
post-add property/recompute fault was injected; only rejected-start atomicity and
ordinary successful transaction behavior were executed.

## Ownership and remaining work

Safe Rust owns tagged validation, frame math, input resolution and bounded owned
serialization. C++ captures actual FreeCAD document/view context, drives Qt/Coin
interaction and native image transactions. Python is the FreeCAD test harness.
Borrowed UTF-8/array pointers last one synchronous call, never enter stored Rust
state, and numeric/report buffers remain unchanged on invalid/short output calls.
Release panic=abort and native dependency safety remain explicit limitations.

Full grammar, all-family session reduction, view identity, snap source identity,
layer/selection policy and durable History remain required P0 work. Distance's
Model_Units default, Angle TwoObjects and PictureFrame embedding/external/alpha
controls remain unsatisfied feature contracts. No all-command or Rhino runtime
parity is inferred. The full acceptance matrix keeps these requirements open.
