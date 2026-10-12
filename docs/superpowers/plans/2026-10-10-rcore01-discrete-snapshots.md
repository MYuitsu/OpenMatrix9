# RCORE-01 mesh/point-cloud snapshot plan

Continue the user-authorized full RCORE-01 inline using executing-plans/TDD;
fresh read-only review after integration. Original chapter/audit remain the
completion authority. This is an extension of the existing owned descriptor,
not a new kernel or replacement for the full goal.

Intent: reuse FreeCAD's native mesh/point kernels and existing native frame,
source attribute, occurrence graph and numeric policies. Direct mesh/cloud
objects and group/link definitions need owned geometry rather than the current
uninspected payload or unsupported leaf. Rust is the portable geometry owner;
C++ reads necessary native storage synchronously; Python only real-host fixtures.

## Design and interfaces

Rust `discrete_snapshot.rs/.ffi`: mesh representation8, cloud9; owned raw-local
XYZ points, mesh triangle vertex indices and local bounds. Empty initialized
native arrays are known empty, not unknown. MAX_POINTS/MAX_FACES100000 each;
8MiB checked retained bytes and serialized payload. Reuse finite coordinate
policy1e150 and existing bounded atomic JSON writer. Triangles must have in-range
distinct indices; no manifold/solid/normal/global quality claim is inferred.
Owned model clones only after validating all values and indices.

FFI initialized live buffers:24-byte double XYZ and24-byte u64 triangle triples.
Input40 bytes: representation/reserved u32, point pointer/count, triangle
pointer/count. Count gate before borrow or C++ staging; null accepted only with
zero count; alignment/overflow/address/output envelopes before slices. No native
handle or allocator transfer. Numeric malformed input and short output are atomic.

Native `CoreDiscreteSnapshot.cpp/.h`: validate exact builtin properties using
the loaded Base::Type registry before a native static cast. Mesh also registers
and checks its exact kernel type. Points does not register PointKernel; its exact
builtin PropertyPointKernel owns the concrete PointKernel returned by getComplexData.
This native ownership contract, not an absent runtime type registration, permits
the cloud cast. Read
MeshObject.getKernel().GetPoints/GetFacets and PointKernel.getBasicPoints inline
through existing module-owned property data. Use the common virtual kernel
getTransform through Data::ComplexGeoData. Avoid unbounded generic getters;
the shared Mesh STL reader dependency is deferred with delay loading as below.
Count gate precedes staging; copy raw initialized arrays to the Rust ABI.
Source object/owner remains live and is revalidated on GUI thread afterwards.

Raw kernel coordinates are native_local. Add native_discrete_location and teach
the existing Rust-backed reframe path to compose it: parent/path/inverse(feature
placement)/kernel transform. The native property synchronizes kernel placement,
but do not assume local points already embed it. Definition reader permits8/9
leaves and keeps their precise classifications distinct from CAD/solid.

Other native flags/segments, property channels/normals/colors/intensities,
retained Rhino f64 source attributes and mesh/cloud subelement mappings remain
explicitly uninspected unless independently copied. Basic native float32 preview
data cannot certify full retained archive preservation; do not close C05/C06.
Arrays/reference contexts and all original topology/report/gate gaps remain open.

## Tasks

1. [x] Rust model/FFI RED→GREEN: owned mesh/cloud and empty/bounds distinction;
   malformed coordinates/index/type/counts; layout/query/short/misaligned buffers.
2. [x] Native RED→GREEN: directly owned geometry, rotated nested parents,
   mirrored/nonuniform links, groups with shared mesh/cloud definitions; source
   kernels/placements/history untouched, replacement/close lifetime; explicit
   unsupported selectors and malformed prepared native data reject atomically.
3. [x] Build only OpenMatrix9Gui on shared SDK; full Rust, new native fixture and
   prior instance/world/support/topology/descriptor/import fixtures on matching
   plugin. Fresh review findings reproduced/fixed; exact fingerprints/results,
   original audit, chapter/status/README/ledger/lookup/MANIFEST synchronized.

Review focus: exact native type/cast obligations; raw-vs-placed storage; transformed
kernel frame not counted twice; no implicit precision/attribute/manifold claims;
counts gate before native vector staging and all FFI slice creation; negative,
overflow/degenerate indices; empty arrays; remaining subelement/instance owners.

Ruling: preserve the accepted ownership/frame design and extend its missing
native owner adapters inline under the active full-goal authorization. Reading
exact builtin module storage through inline APIs and delayed Mesh imports avoids loading extra DLLs when
the plugin starts; unsafe casts remain a narrow native obligation verified by
type/property checks, not a memory-safety certificate.

Build ruling: MSVC exports std::vector<MeshPoint/MeshFacet> storage readers from
Mesh.pyd; the first build proved six unresolved begin/end/size symbols. Import
the existing shared Mesh.lib and delay-load Mesh.pyd, with system delayimp.
This resolves the native STL ABI without overriding export macros or inventing
a container layout. Actual mesh owners already require their native module;
prior non-mesh fixtures must still load the plugin cold. No Points library is
needed for its inline float32 storage reader. The cost is a deferred dependency
on the matching shared Mesh SDK module, not a second host/runtime build.

Accepted evidence:628 full Rust tests;99 new+6 cold+911 regression=1016 host
checks;all11 matching processes exit0. See docs/validation/2026-10-10-rhino-core-discrete-snapshots.md.
Original RCORE-01 remains in progress; unknown native channel/archive/reference
and full original gate requirements are retained in the completion audit.
