# Source/topology binding in the remesh proof

The in-process `TopologyAttestation` now records an integrity digest of the
source fingerprint and the UUID/adjacency topology captured with the native
snapshot. After the existing document/display validation, the tracked
source-only lease is checked against that digest both when captured and when
used to rebind a remeshed candidate. The digest binds the topology to the
attestation; by itself it does not prove a native mapping, authenticate a
file, or make a legacy v4 B-rep eligible for display eviction. The ShapeSet
restore path updates the binding only after its existing reference mapping has
rebound the candidate topology.

A native Sphere radius 5 fixture dropped the original B-rep and confirmed its
display was released while the attestation still accepted the original source
fingerprint and topology. It rejected the same source fingerprint with a
different face UUID, and the same topology with a changed source fingerprint.
This checks the source-only binding function; it does not exercise arbitrary
tampering of a live document or independently prove OCCT mapping.

With `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, kernel topology tests with
`3dm-brep` passed 10, with 3 manual probes ignored. The app `topology` filter
passed 51 tests with 2 ignored, including existing remesh, Undo/Redo, pressure,
and source export fixtures. The first kernel run exposed an error variant
change; the existing full B-rep path retains its previous error and the group
passed on rerun. Public-source audit and `git diff --check` passed.

Computing the digest scans topology IDs and adjacency; large-topology latency,
native peak memory and desktop operation were not measured. Sphere production
still does not install this attestation, and document/Undo still own display.
The P1 source/display and history gates remain open.
