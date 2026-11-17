# P1: OCCT topology serialization probe

Date: 2026-11-05. Diagnostic only; this is not a topology identity certificate.
Probe source: `implementation/design/M01/viewport-export-separation-20261102/topology-normalization-probe.cpp`
(SHA-256 `433e4c62d42a9e9c7e3b59b0ef75b7e24680a660c04969656f0898c4aebe6a97`).
Windows x64, MSVC 14.44.35207, project OCCT 8.0.1 SDK/runtime; compiled
as a standalone native probe with output under `L:\om9-target`. It generated a
10 × 20 × 30 box, a radius-5 sphere, and a compound containing two separately
constructed coincident boxes, serialized with the same
`BRepTools::Write(shape, stream, false, false, TopTools_FormatVersion_CURRENT)`
settings used by the snapshot adapter, and read with `BRepTools::Read`.

| Fixture | First whole-source roundtrip byte-equal | Second roundtrip byte-equal | Face raw subshape bytes same in order | Edge raw subshape bytes same in order |
| --- | --- | --- | --- | --- |
| Box | yes | yes | yes, 6/6 | yes, 12/12 |
| Sphere | no | yes | no, 1/1 differs | no, 3/3 differ |
| Coincident boxes | yes | yes | yes, 12/12 | yes, 24/24 |

The sphere's two vertex serializations matched on the first roundtrip. After
normalizing each subshape separately via one write/read/write, all indexed face,
edge and vertex strings matched across the initial roundtrip for all three fixtures;
they were unique within each topology type for the individual box and sphere.
The coincident boxes gave only 6 unique face strings for 12 faces, 12 unique
edge strings for 24 edges, and 8 unique vertex strings for 16 vertices. After the first
whole-shape roundtrip, a second roundtrip also preserved indexed subshape strings
for all three fixtures. The program compares complete strings, not hashes.

Consequences: raw serialized subshape bytes cannot safely be used to match
original sphere topology to topology read from its source. Per-subshape
normalization is a candidate to investigate, not a general proof: this probe
does not resolve duplicate coincident subshapes, reversed orientation, seam
occurrences, trimmed NURBS, imported v4 shapes, version upgrades, or native
memory peaks. Nor does it show that equal geometric subshapes have the same
topological role. No behavior or archive version was changed by this probe.
Any production mapping must reject ambiguous cases, bind source and kernel
version, validate graph/adjacency, and be admitted within the memory budget.
