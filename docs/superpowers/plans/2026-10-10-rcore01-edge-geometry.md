# RCORE-01 edge/vertex geometry implementation plan

> For agentic workers: use superpowers:executing-plans inline, with TDD and a
> fresh read-only review after integration. The original RCORE-01 contract and
> completion audit remain authoritative; this plan does not reduce their scope.

Goal: retain the missing 3D edge/vertex/tolerance data associated with native
face/pcurve topology, including loose edges/vertices and standalone Part roots.
Architecture: extend the existing guarded owned snapshot and native frame design.
Rust owns bounded numeric rows and validates IDs/endpoints/domains/tolerances/
locations. C++ reads native OCCT storage and reuses Rust-owned support serializers.
Python runs independent real-host fixtures only. Tech: Rust, OCCT/Qt, shared FreeCAD.
Spec: OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md,
and docs/reviews/2026-10-10-rcore01-closure-audit.md.

The user-authorized full chapter and accepted owned snapshot/frame design already
require these data. This is a bounded extension of that existing interface, not a
new backend, kernel, editor or ownership architecture. Do not ask again to choose
the approved Rust/native split; preserve every original requirement still open.

## Global constraints

- Reuse H:/FreeCAD-src/build/relWithDebInfo; only OpenMatrix9Gui plugin target,
  existing openmatrix9-history-cage cache and reusable-history-runtime output.
- Rust owns portable data/validation. Native handles stay synchronous and guarded
  on GUI thread; no source mutation, Curve3D reconstruction or implicit healing.
- Existing topology use/face IDs remain stable. Native TShape+location determines
  unique local edge/vertex identity; repeated located identities remain distinct.
- 16384 edge+vertex rows;8MiB retained/serialized numeric payload. Finite coordinates/
  tolerances bounded1e150; tolerances nonnegative; location uses existing checked
  affine Frame. Shared support payload accounting remains distinct from global/
  native/Qt allocation and peak staging. Original allocation gates remain required.
- Counts/envelopes precede staging/slices, validation precedes Rust cloning, JSON
  finishes before output copy. Live initialized immutable ABI storage remains a
  caller obligation; no whole-program memory-safety claim.

## Data and files

rust/src/edge_geometry.rs/.ffi and rust/tests/edge_geometry.rs: repr(C) Edge192
bytes (edge/start/end IDs, domain/tolerance, present/degenerate/SameParameter/
SameRange flags, location16 doubles), Vertex168 bytes (ID/XYZ/tolerance/location),
Input32 bytes (two ptr/count pairs). Owned private vectors; IDs ordered/consecutive,
endpoints resolve to vertices. Missing nondegenerate Curve3D rejects. Degenerate
null curves have explicit absent domain, not invented zero-length geometry.
JSON edges_3d/vertices includes native_location and geometry_coordinates native_local;
native parameter flags are observations, with null correspondence certificate.

Gui/CoreEdgeSnapshot.cpp/.h reads BRep_Tool Curve without applying location,
forward canonical endpoints, vertex Pnt with occurrence location removed from a
detached TopoDS view, native tolerances and flags. The existing CoreSupportSnapshot
reader gains general3D NURBS/trim-unwrapping support, reusing checkedBasis/count gates.
CoreTopologySnapshot collects all unique edges/vertices, retains face tolerance,
and joins new owned rows with existing pcurve edge IDs. CoreGeometryContract reframes
edge/vertex locations exactly once, exposes standalone Part topology and point
payload, and reuses copied edge support at the standalone curve root.

## Tasks

- [x] Rust RED→GREEN: owned rows/empty distinctions, IDs/endpoints, missing versus
  degenerate curves, domain/tolerance/coordinate/transform flags, counts and ABI
  alignment/null/reserved/output canaries/serialized limits; full Rust suite.
- [x] Native RED→GREEN: holes/seams/poles/cavities, loose/mixed topology, standalone
  point/curve/wire, rational and polynomial3D support, restrictions and reversal,
  direct/group/link frames and source preservation. Independent native UV→surface
  and3D curve samples use explicit native flags/domain, without global parameter
  map certification. No false finite domain for degenerate absent curves.
- [x] Matching plugin build and all relevant previous instance/discrete/world/
  support/topology/descriptor/import fixtures; fresh read-only review and fixes,
  exact evidence, audit/spec/status/README/ledger/lookup/MANIFEST synchronization.

## Review focus

- Degenerate pole edges may have no Curve3D; nondegenerate missing curves must
  reject without an implicit bake/reconstruction or source mutation.
- Curve location and vertex location may already include source Placement;
  shared reframe must remove embedding once under nested mirrors/scales.
- SameParameter/SameRange flags do not certify actual global correspondence.
- NURBS full domain must stay separate from edge and pcurve restrictions;
  reversed edge use does not reverse/duplicate the supporting basis.
- Loose mixed-compound edges/vertices and repeated located topology must not be
  dropped or create fictitious face adjacency; payload/row gates stay bounded.

Remaining full chapter obligations, including manifold/solid/cavity certificates,
typed conversion/operation reports, arrays/reference contexts, attributes, numeric
owners/raw decode and all mutation/worker/lifecycle gates, remain in the audit.

Accepted evidence:632 full Rust tests;921 new+1016 regression=1937 host
checks;all12 matching processes exit0. See docs/validation/2026-10-10-rhino-core-edge-geometry.md.
Original RCORE-01 remains in progress; full original requirements stay in the audit.
