# Rhino core P0 — extended numeric input — 2026-10-09

Status: **validated bounded increment; all P0 remains in progress**. Parent `349203d`.

The common Rust grammar now resolves polar `D<A`, polar height `D<A,Z`, spherical
`D<A<E`, XY elevation `X,Y<E` and quadrant surveyor `D<N/S angle E/W` forms.
Documented local/world/relative prefixes are explicit; `rw` is polar-only and `@w`
surveyor-only. Fractions, mixed fractions, feet/inches, the `millimeters` suffix and
independent degree/DMS angles are supported through the shared units parser.
Angles never receive a length scale. Original text, evaluated world point, frame,
units and notation are retained in shared reports; angles are radians in notation.
The report remains schema1 with additive notation and declares `rhino5_point_v2`.

## Reuse before implementation

Line/Polyline geometry already existed in `Gui/CurveController.cpp`, which calls
FreeCAD `Part.makePolygon`. Existing Rust sessions, Circle/Ellipse constructors,
Rectangle geometry, interpolation, Box/Sphere native construction, measurements
and ImagePlane creation are reused. This increment changes missing parsing,
phase routing and accepted-input state; it does not introduce replacement solvers.
Host Draft also already has `draftmake/make_line.py::make_line` and
`draftmake/make_wire.py::make_wire`; its presence alone is not evidence of the
specific OM9 command, frame and Undo contract.

Safe Rust owns grammar, numeric limits, notation and session state. No native
parser fallback or Python business state was added. FreeCAD/Qt/OCCT continue to
own host UI/geometry; Python macro files are application acceptance harnesses.

## Reviewed repairs

Exact angle tokens just above 1e9 degrees can round to the allowed bound; exact
bounded fraction/decimal comparison now rejects them. XY elevation rejects exact
odd quarter-turns in degrees before tangent conversion, including many revolutions.
Both have failing regressions before repair and successful retries with no phase
or document mutation after repair.

Sphere now keeps an actual accepted point separately from synthetic construction
coordinates. First relative input in FitPoints/Tangent rejects without a base;
the third FourPoint input becomes the correct base; Undo restores point/reference
and batch context. Circle accepted Radius/Orientation input and options use bounded
input checkpoints, retaining the real relative base across errors and Undo.
Hover works on a clone and does not create an input checkpoint.

Rust also supplies opaque monotonic input-context identities; Undo restores an old
identity while later input never reuses discarded branch identities. A thin native
adapter restores path/tangent geometry and Direction/Radius source identities for
that context before pruning. Native snapshots hold private OCCT shape copies and
value guards/metadata, with immutable geometry shared between contexts. The new
snapshot store retains no Python objects; live wrappers are recreated on restore.
The existing source ledger compares saved identities against the live document
before use, without dereferencing saved object addresses. Composite native input
captures its intermediate events. F4 now captures/verifies native references before
accepting the origin. Source edits still drive restored associative History.
This is a recorded native ownership exception: host geometry cannot live in Rust.
The 1024 recent-checkpoint/1025 context limits are OM9 policy, not Rhino parity.

## Verification

- Matching SDK build passed; final native SHA256 `71a762ab77445d52ef113d3a578c65a7c79c177d2e31cdcd50901b2739078e45`.
- Full Rust: 421 passed; Clippy exit0 with existing warnings.
- Python tooling: 49 passed,1 skipped.
- Native: 1108 passed assertions in 21 exit0 processes with current macro/module hashes.
- Source audit and exact spec/catalog synchronization are recorded in machine evidence.

| Suite | Assertions | Directory in build/merge_replay-1 |
| --- | ---: | --- |
| circle-context | 16 | `97245d1aa9a546c69c6a212d761b9b06` |
| extended-input | 58 | `c7bb56dc826d47b7883713ae8af1730c` |
| extended-input-cold | 4 | `a1f2a4e052724596b7b9233d7bad0945` |
| shared-input | 33 | `d64ce2380ca143cd9320badc3d155b93` |
| distance | 28 | `dd0a1a2700924f5e9280239600d71097` |
| angle | 19 | `5fe3f159bded4f65a35c90ebd70d1058` |
| picture-frame | 47 | `0fe2d6cd3aba45fcbe9f249cd6958520` |
| invalid-cplane | 14 | `3781dbeb9d6d45e4983866612c66ad10` |
| snap-tools | 7 | `2f79927d7ec4445dabf209a1a5778b2c` |
| snap-lifecycle | 10 | `b6009a01951740769a7dc31c40cfa2af` |
| coordinates | 162 | `d47cef4dd937474796fa8bbe0f79ef17` |
| circle-basic | 42 | `f849f46aa1b64e8d95dea9b9dc69b4e6` |
| circle-advanced | 64 | `290cedbc79044de680d21539f09a1f42` |
| circle-nonplanar | 30 | `e517ee753070493f8b918bff408de005` |
| circle-history | 89 | `c7b6c464b577479f8ff92d465efb1adb` |
| circle-spatial-history | 53 | `657966bc522d46da94f40ba89f051340` |
| solid-options | 252 | `3b1019806d814b6197b6a4030990ad37` |
| curve-basic | 27 | `e83e9b23bfb6427d93e718d172b7ca41` |
| curve-spline | 41 | `a55e86ffbfcc4bd8937331e30c696884` |
| rectangle | 37 | `ed7fe4793350462aa7180be4948f0824` |
| ellipse | 75 | `ed2694a5aee54e88924daa91f23c55b0` |

Measurements are read-only; malformed extended input preserves accepted context.
Existing native geometry for eight command families matches independent world-space
oracles on a rotated/translated frame. PictureFrame polar placement, dimensions,
Undo/Redo and a separate-process FCStd reload match exact provenance; its embedded
image remains usable after the external fixture is removed.

The FourPoint Sphere fixture initially used the default native bounding-box center
as its analytic center. Diagnostic replay found the exact radius1 surface centered
at zero and volume4π/3, while default bounds had centerX≈−0.000948336 mm. The fixture
now checks the analytic surface center/radius and independent volume. The failed
oracle runs are retained; no Sphere production repair was made for that discrepancy.

The [Rhino5 numeric-entry help](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/unit_systems.htm)
and mapped guide pages supplied documented syntax. Mathematical oracles and FreeCAD
runtime establish this OM9 slice. The installed Rhino5 application crashed during
two isolated startup-only/numeric probes, before any script; these failures do not
establish numeric semantics. No exhaustive Rhino runtime parity is inferred.

## Remaining scope

- All P0 remains in progress. Live distance/one-shot-angle constraints, locale variants, other CPlane modes, common all-family sessions and snap identity remain open.
- Solid retains its legacy mm/cm/in and bare-mm callback; fractions/feet/meters/model-unit capture for Solid are not supplied by this increment.
- Distance Model_Units, Angle TwoObjects and PictureFrame explicit embed/external/alpha controls remain open. Persisted provenance is verified for PictureFrame, not every geometry family or associative History.
- Report schema_version=1 retains original fields and adds typed notation; grammar rhino5_point_v2 replaces cartesian_v1. Consumers must read the declared grammar.
- New numeric bounds and ambiguity rejection are OM9 policy. Legacy bare decimal length parse underflow remains unchanged. Native dependency safety and release panic=abort remain limits.
- Circle retains the most recent 1024 accepted-input Undo checkpoints and at most1025 native contexts. Geometry copies are shared across contexts, but aggregate source-geometry/ledger RAM is not certified; source byte budget remains open.
- Rhino5 5.14.522.8390 is installed, but two isolated startup probes exited3 before Python executed (MSVCR100.dll, exception0x40000015). No new Rhino runtime numeric parity is claimed.

[Hash-bound evidence](2026-10-09-rhino-core-extended-input-evidence.json) preserves final runs, excluded
baselines, source/review hashes and precise limits. The all-P0 matrix keeps full
fixtures open; no completion percentage is inferred from test totals.
