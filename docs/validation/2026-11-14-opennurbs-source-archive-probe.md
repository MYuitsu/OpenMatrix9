# openNURBS source archive mesh policy probe

Batch `opennurbs-brep-source-archive`: two executions of one authored box
fixture passed; the second execution additionally reopened the full archive and
checked that it actually contained both cached mesh kinds. This is a native SDK
serialization probe, not a product import/export or `.om9` v5 test.

The fixture attaches render and analysis meshes, a user string and a registered
authored user-data object to a 2 × 3 × 4 B-rep box. With the pinned SDK's
`EnableSave3dmRenderMeshes` and `EnableSave3dmAnalysisMeshes` disabled for
`ON::brep_object`, the filtered archive reopens as a valid B-rep with eight
expected vertices, 12 edges, six faces and six loops, the user string and the
authored attachment. It contains neither cached mesh. The full archive reopens
with both meshes. The filtered archive equals byte-for-byte a subsequent
archive of the same source after `DestroyMesh(ON::any_mesh)` on the private
fixture. The full archive was 13,905 bytes and the filtered archive 12,893 bytes;
these sizes describe this fixture only.

SDK pin: `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`. Final fixture source
SHA-256: `47d104f9fb5c7eb3178fb6ca000b9f53e38bc7c15760a2a8290388ac23e3e978`;
fixture executable SHA-256:
`ffc51162c374339d88e2fa4eef8398e937b7e01206906e2f32807fa354de97f7`.
Built in the existing `build/native-source-fixture` with MSVC Release and
`--parallel 2`, then executed through the existing native runner with the
executable digest. The final runner reported exit success, 386 ms and peak
committed 34,140,160 bytes under its process limit; this is one process sample,
not a benchmark or a bound for arbitrary B-reps.

The product adapter was deliberately not changed: an owned 3DM export that
relied on the original cached meshes could otherwise lose display for readers
that cannot remesh. A box does not prove trimmed holes, arbitrary surfaces,
unknown third-party user data, all hidden payloads, or independent geometry
equivalence. Before using this policy in v5 or import, verify these cases and
provide an accounted background representation and an export strategy; retain
the original file/provenance where required. C1–C5 remain open.

One previously prepared batch remains, `modeling-public-rhino5-generated-exchange`
(four fixtures, seven objects); packages 2–8 remain open. The total number of
future batches is unknown. Next: test a trimmed hole and external user data with
an independent oracle, then connect the policy to an actual source-only path
with 3DM export and cold-restore coverage.
