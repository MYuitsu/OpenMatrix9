# Common topology snapshot review

Fresh reviewer: `topology_snapshot_review`, read-only review of the uncommitted
Rust/native adapter and fixtures against RCORE-01 and the topology snapshot plan.
The author reproduced and fixed both substantive findings before acceptance.

1. Repeated same native face/location in a compound doubled edge incidence,
   falsely reporting a closed surface and removing free boundaries. The authored
   real-host RED at `H:/FreeCAD-src/build/topology-snapshot-repeated-red-20261010`
   fails the open-boundary assertion. Native extraction now captures each unique
   face/location once, matching FreeCAD's Faces convention, and separately retains
   every face occurrence and orientation. The final host covers identical,
   opposed-orientation and located shared-storage occurrences.
2. The Rust shared-edge consistency check accepted opposite edge orientation
   with the same directed endpoints. The closed-loop malformed seam fixture
   `opposite_use_orientation_requires_opposite_directed_endpoints` returned Ok
   before the fix. It now rejects: equal orientations require equal directed
   endpoints, opposite orientations require reversed endpoints; self-loops work.

Ruling: unique native faces supply the recorded-face incidence graph; occurrence
metadata preserves repeated compound references. It is an observation of topology,
not a multiplicity-based area/volume computation or full manifold certificate.
Cost if this convention is misused: a downstream quantity consumer could ignore
occurrence multiplicity; the exposed occurrence array and scope must be respected.

The reviewer found no additional defect in checked spans, graph bounds or atomic
reject output under the documented initialized/live/nonoverlap FFI contract.
Native pcurve selection uses forward-face traversal and oriented edge occurrences,
as in the existing accepted BRep exporter. Runtime tests check opposed periodic
UV sides rather than only assigning two different pcurve IDs.

No native pointer accessibility, raw decoder allocation, full vertex-link
manifoldness, solid orientation or Rhino parity claim follows from this review.
No deferred minor findings were reported. No additional review pass substituted
for the RED/GREEN regressions or the final full Rust and real-host controls.
