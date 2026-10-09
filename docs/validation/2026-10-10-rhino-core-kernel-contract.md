# Rhino core shared kernel contracts — 2026-10-10

Status: **validated bounded increment; all P0 remains in progress**. Parent `fcdb0ac`.

Existing FreeCAD/OCCT/openNURBS circle, curve, surface, trim/seam and shell converters
are reused. Safe Rust now supplies general borrowed curve/surface basis validation,
explicit policies and checked count/byte preflight. The old generated spline gate
delegates to that validator under its original limits. C++ extracts native values
and checks storage envelopes; it retains native geometry/accessor ownership.

Rust also owns schema1 representation/topology observations, operation/options
policy and conversion-report/parameter-map data. The detailed policy coexists with
the old coarse capability API and currently supports inspection and existing V5
working exchange. Permission alone does not assert geometric preservation.

`geometryDescriptor(document, object, subelement)` returns owned Unicode JSON on
the GUI thread. Existing guards are verified before/after capture. Basis arrays
are independent and bounded; stamp identities are exact decimal strings. Native
support facts and restricted domains have separate typed scopes. A half-circle
has closed/periodic supporting geometry and an open topological edge. A partial
cylinder has periodic supporting U while restricted direction facts stay unknown.
Read/rejection/copy tests add no document properties, objects or Undo entries.

## Exact fixtures and corrections

The rational R10 circle retains non-unit weights and sampled radial/circumference
invariants. A 20×20 face keeps its R3 hole, hole exclusion and area 400−9π. The
uncapped R4/H7 cylinder remains a face; ordered seam uses share one edge identity,
and two connected circular free boundaries are measured. Native periodic export
keeps source/sibling basis unchanged; no converter ownership repair was needed.

Area oracles use independently known values and a declared absolute 1e−7 mm²
threshold. Adaptive 1e−14/1e−16 results converge by 1.14e−13; residuals are 5.68e−14
for the hole face and 1.40e−8 for the cylinder. Earlier coarse integration failed;
its tiny returned relative estimate is retained as a diagnostic, not a certificate.
Cold bounds use geometry-only optimal bounds because default bounds can include
mesh approximations. Numeric OCCT tolerance still exists.

The sphere diagnostic preserved the failing raw BRep. Actual display events did
not change it. Explicit meshing of the source changed only its face Checked flag
0101000→0111000; native geometry/tolerances/connectivity remained equal. Samples
now use explicit deep geometry copies without copied triangulation. Exact source
hash/radius/stamp/guard predicates remain intact. Two originally chosen tolerances
both mapped to 0.1 rad angular deflection and produced 8000 facets. Fine 0.01 derives
0.055 rad and refines to 26716 facets. These counts belong to independent samples;
they are not direct viewport-cache measurements. Failed runs remain in evidence.

Both display modes also export/import the sphere through the existing working
CAD 3DM route. Exactly one native CAD solid returns, with no mesh fallback and
unchanged source. Default FreeCAD Shape.Volume returned 113.04787279841815 mm³
against 36π = 113.09733552923255 mm³; local source confirms nonadaptive integration.
It is retained as a diagnostic. A separate native adaptive 1e−14/1e−16 oracle reads
each exact imported BRep, checks its hash and evaluates 36π under the original
absolute 1e−5 mm³ target, including pair convergence. Residual is 9.01e−9 mm³;
pair difference is 2.84e−14 mm³. No tolerance was enlarged.

Worker regression initially lacked the host-location executable and failure
fixture. The harness now tests the application's actual worker path and requires
its bytes to match the isolated build. Temporary deployment restores the previous
host state; serial/parallel geometry, missing/nonzero rollback and process cleanup
are verified. No worker production path was changed.

## Verification

- Matching SDK configure/build exit 0; module SHA256 `b80631e87580fb0c33efa63fb65151edbab1ac2b22d1a75bbcb05c2c35df660c`.
- Full Rust 480 passed; Clippy exit 0 with existing warnings.
- Python tooling 49 passed, 1 skipped; native basis adapter exit 0.
- Native exact fixtures 59 passed; Geometry/TrimInteriorMapping/MultiShell 3 ctest suites passed.
- Two exact-input sphere oracle runs passed 64 checks each (59 repeated plus 5 sphere checks).
- Application 504 assertions in 8 exit 0 processes, including cold reopen.
- Complete numbered fixture evidence recorded only for RCORE-01.T02, RCORE-01.T03, RCORE-01.T04.

| Suite | Assertions | Directory in build/merge_replay-1 |
| --- | ---: | --- |
| kernel-warm | 142 | `1bf94f2737bf48abb8b9040071ba4103` |
| kernel-cold | 79 | `975f60237386484981f2ce57e5e6d1b1` |
| companion | 142 | `1efa76209858406d966eefef052da9b4` |
| spline | 41 | `a0cd97193c0543cfba1f221ecc53a79f` |
| circle-context | 16 | `72a0dbe8aa7449cbb0368c64dbc3a892` |
| circle | 42 | `b9f56d2addd1448db8e6bfa74557cf6e` |
| modeling-exchange | 32 | `a177316dbdeb4ab49784a1596c5392b3` |
| modeling-workers | 10 | `0e58059770b644e6aa199cbae1fa288e` |

The SDK's obsolete openNURBS cache path was repaired to an existing checkout at
the same pinned revision eb92af3ba1806b0a34a99aba0d3bda83e3d46083. No dependency
upgrade or solver replacement was performed. Native gates test rational weights,
periodic/expanded conventions, malformed counts/pointers/address spans and
selected raw curve/surface/revolution entry points. They do not certify every
foreign allocation or every rational consumer.

## Remaining scope

- All P0 remains in progress. The shared registry currently gates inspection and supported working V5 exchange only; Boolean/edit/other command families do not become universally routed or enabled.
- A descriptor owns observations, not a geometry certificate or mutation permission. Supporting closed/periodic facts are separate from edge/face restriction domains; edge closure is native topological endpoint identity. Face restriction closure/periodicity, manifold, orientation, full trim graph and attributes remain unknown.
- World placement and native support location are copied separately. Coordinates and local_to_world composition remain unknown; no general once-only nested/mirrored/nonuniform transform certificate.
- Curve/surface count/value gates cover selected native converter branches. Archive parsing, GetNurbForm, native copies, proxy/member consumers, downstream pcurves and aggregate allocator/worker memory are not all prevalidated. Literal full T07 remains open.
- Generated spline limits remain degree 1..11, poles 256 and coordinates ±1e9. Native direct basis policy allows degree 25, poles/knots 1e6, logical 64 MiB and coordinates ±1e150; the read-only companion has its own 8192-pole/16384-knot budget with explicit uninspected output.
- Circle locus deviation is sampled, not a certified global maximum. ConversionReport/ParameterMap are owned typed contracts with unknown facts until evidenced; no general conversion mapping/query result or dependent History propagation is implemented here.
- Periodic source/sibling export, source-read immutability and cold reopen are selected proofs. All borrowed/subobject mutation ownership, delayed solver cancellation/UAF and complete T09/T11 remain open.
- Release panic=abort and native allocator/exceptions remain separate lifetime boundaries. Rust C ABI pointer preconditions require live initialized readable allocations; scalar pointer checks cannot prove foreign allocation validity.
- Rhino5 startup still crashes before scripts; no new Rhino runtime semantic parity is claimed.

[Hash-bound evidence](2026-10-10-rhino-core-kernel-contract-evidence.json) contains successful and excluded
runs, source/review hashes, native linkage and audit/catalog synchronization.
