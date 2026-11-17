# ExtractWireframe from retained 3DM B-rep payload

## Native source and edge allocation failures

Native-to-source conversion now has a typed admission result for the
ExtractWireframe caller. A failed Rust allocation copying native source bytes
reports a memory refusal; an invalid payload or an address/structural limit is
reported separately. Similarly, fallible Rust allocations for the edge output
are marked distinctly from edge-count and size limits. The caller maps only the
allocation marker to the existing memory-recovery flow and keeps the original
managed-admission error when that is the cause.

An isolated test allocator deliberately failed one real Rust allocation while
copying source and another while allocating the edge list; both were identified
as allocation failures, while an output-count limit and invalid payload stayed
distinct. This is injected allocation failure, not an exhausted operating
system. A separate UI mapping test checks the allocation/structural distinction;
the headless native-wireframe menu recovery fixture checks a managed-memory
refusal, not the injected allocator end to end through the UI.

Two authored 3DM cylinder fixtures passed again after the change. Wireframe
passed 80 tests with six ignored, DupBorder passed 24, and four retained-native
STL regression tests passed. Actual physical OOM, C++ transient allocation,
third-party drawings and desktop remain unverified.

## Authored 3DM input follow-up

Two actual `.3dm` files written by separate project-authored openNURBS fixture
generators were imported with `BrepMode::RetainOpaque`, saved/reopened as
`.om9`, and processed by ExtractWireframe. A quarter-cylinder face produced two
trimmed circular arcs and two side edges; five sampled points on each matched
the expected radius, height and quadrant. A capped cylinder produced two
circles and one seam with the expected radius and height. Both passed Undo/Redo;
the quarter-cylinder fixture also retained Redo before the edit.

The two individual runs and a combined run all passed. Inputs were the existing
`L:/om9-target/dupedge-arc.3dm` (SHA-256
`bacf2611787e0b652ea3fc9f776b627aa6f779537a4b5c048c0c5ecb5a516125`)
and `L:/om9-target/dupedge-cylinder.3dm` (SHA-256
`5f8646471aa87f7ba8fa9f8336806aa87aa759e7297414e70bda562027ae3b0b`).
The compatible P1 Release native exchange executable had SHA-256
`3363dc46cf420db81b44d703efadf5b9885a34834c932de8f28225c033889c9e`.
Their source generators are `crates/om9-app/tests/fixtures/dupedge-3dm-arc-generator.cpp`
and `dupedge-3dm-cylinder-generator.cpp`. These are project-authored synthetic
fixtures, not arbitrary third-party files or independent Rhino acceptance.
Desktop and native transient peak remain unverified.

The earlier sections below describe the state before this follow-up; their
statement that no actual 3DM input was tested is historical.

ExtractWireframe previously accepted editable OCCT B-reps, meshes and surfaces,
but refused a `NativeBrep` retaining an openNURBS B-rep payload. It now converts
that payload to an operation-local canonical OCCT source, extracts edges from
one source topology map and commits all requested outputs together. The
conversion and Rust-owned edge outputs use the existing admission callback;
OCCT/openNURBS transient allocations are not covered by that callback. No
viewport mesh is used as the geometric source.

The scoped native-source batch had five executions: one test with rigid and
affine placements, one invalid-payload test, one mixed NativeBrep/mesh test,
one headless menu-recovery test, and a final full wireframe suite including the
first three tests. Five distinct scenarios passed:

- a box NativeBrep after `.om9` reopen, with translation or affine placement,
  produced the twelve correct distinct edges even after the display vertices
  were shifted; CMD and menu routes and one Undo/Redo were checked;
- a NativeBrep and mesh produced 15 edges in one Undo/Redo transaction;
- a malformed NativeBrep following another valid selected source was refused
  with the document and Redo unchanged;
- simulated managed pressure preserved document and Redo, and a headless
  menu recovery fixture retried after pressure was released.

The full wireframe suite passed 80 tests with four diagnostic probes ignored;
DupBorder passed 24 tests; the targeted menu-recovery test passed. These tests
create the openNURBS payload from an OM9-generated box and save/reopen the
`.om9` document. They do not establish a real external 3DM import, arbitrary
native B-rep topology, native temporary peak, actual OS out-of-memory behavior
or desktop acceptance. An internal allocation failure in the native-to-source
copy has not been independently injected through the UI, so automatic memory
recovery is claimed only for the simulated managed refusal.
