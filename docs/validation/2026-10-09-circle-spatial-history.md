# Circle spatial tangent / AroundCurve History — 2026-10-09

Feature `OM9-CURVE-005` remains `partially_implemented` /
`validated_supported_slice`. [Current scope](../features/OM9-CURVE-005.md).
Source reread: Matrix8 Book1 PDF138–140 (printed128–130), Circle through its
continuation before Ellipse. Host choices are recorded separately from source.

The later [remaining-branches validation](2026-10-09-circle-complete-branches.md) supersedes the nonplanar Tangent, other History and layer limitations in this historical report.

The Record/Update pause/resume evidence below records the earlier policy. The
later [RCORE-09 implementation policy](../features/history-native-contract.md)
uses Record only for new links; Update independently controls existing records.
Historical result counts below do not establish validation of that revision.

## Native evidence

Matching MSVC2022/Qt6/OpenCascade8 native module builds and runs successfully.
All reports below have `ok=true`, every check passed, and runner process exit0.
Selected `OpenMatrix9Gui.__file__` is explicitly verified in each test.
SDK executable: `H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe`.
Module: `H:/OpenMatrix9-private-backups/20261006-140212-public-prep/isolated-runtime/bin/OpenMatrix9Gui.pyd`; final SHA256 `abe3dd064817eeb02e430fefbc04a57524f1ecd3ce0924913da8b88d0c4ad440`.

| Suite | Checks | Report relative to module root |
|---|---:|---|
| circle_spatial_history_smoke | 51 | `build/circle_spatial_history_smoke-1/b4eb25e8c6534b96b2b2b067b751c424/results.json` |
| circle_history_restore | 3 | `build/circle_history_restore-1/636271b937ad46e3bf9ac3dffbcde74c/results.json` |
| circle_advanced_smoke | 64 | `build/circle_advanced_smoke-1/a8afe08309924e36afb4cb9a0bd465ad/results.json` |
| circle_smoke | 42 | `build/circle_smoke-1/d98e66912c66452ea429f5d0a954b5c5/results.json` |
| rectangle_smoke | 37 | `build/rectangle_smoke-1/179443162053452ba0273d05c1e7cdf2/results.json` |
| curve_smoke | 27 | `build/curve_smoke-1/f5f1588b0dc14b86a5b9a4c6d0dcdee3/results.json` |
| curve_spline_smoke | 41 | `build/curve_spline_smoke-1/b2159ff3be704eb28fd73504a98e6b19/results.json` |
| history_smoke | 83 | `build/history_smoke-1/2a6fc7fd99484a36accbaa0f6bcc5daa/results.json` |
| surface_history_smoke | 65 | `build/surface_history_smoke-1/98d7b07edb8d4909acef145d943f755c/results.json` |

New coverage: YZ and tilted world-placed line/circle/NURBS constraints, spatial
three-curve incircle, tiny conics, Vertical failure/recovery, fixed contact and
Point, skew/nonplanar rejection; optional History edge/fraction/radius metadata,
straight and curved source edits, parent translation/rotation, source replacement
rejection, Record/Update pause/resume, Lock and locked source deletion, child-edit
detach, creation/parent/source Undo/Redo, invalid source recovery and FCStd.
Cold restore autoloads the native type and updates its parent without activating
OpenMatrix9. Curve regression totals211 checks, shared History/Surface regression
totals148 checks. Defaults remain snapshot.

## Ownership, corrections and checks

Rust owns plane inference, numeric Circle plan validation, tangent acceptance
and command options. Native OCCT constructs/checks exact bounded contacts;
FreeCAD owns document links, callbacks, properties and transactions. Existing
Rust History graph policies schedule Record/Update/Lock. Native exceptions are
required API adapters; FFI/OCCT are not certified memory-safe.

Rust tests were written first and failed on the missing API; final complete run
has171 passing tests in33 nonempty suites, including26 Circle cases. Python tools
have23 passing tests. Clippy exits0 with23 existing warnings outside Circle.
Selected Rust files pass rustfmt; Git diff whitespace check uses cr-at-eol for
existing CRLF files. Source audit flags290 pre-existing private/reference artifacts
and validates510 icon bindings (exit1); no publication performed.

Independent review found redundant initial Shape assignment detached a new
History feature; initialized native output now skips that write. It also found
automatic source unlink under Lock and absolute plane tolerance on tiny conics;
native regression covers both fixes. Parent Undo initially reproduced stale
geometry; document Undo/Redo notifications now refresh recorded dependencies.
Shutdown checks exposed retained transient Python curve references after commit;
they now release immediately after persistence. The complete native run and cold
restore exit0, rather than accepting successful UI assertions with a crashed process.

## Remaining scope

General nonplanar Tangent, exhaustive/certified NURBS root enumeration, individual
surface edit-control-point selection, History for other Circle constructions,
active-layer integration and full Matrix compatibility remain unaccepted.
Stored EdgeN/fraction is not a general topology correspondence mechanism.
History is analytic AroundCurve only; radius remains fixed, center/normal update.
