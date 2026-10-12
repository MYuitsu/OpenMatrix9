# Circle History result review — 2026-10-12

Scope: the new `ReplayCircleHistory` result path, preserving the existing Circle
solver and recorded recipes. A fresh read-only reviewer inspected source; it did
not edit, build, run Rust tests or launch native hosts. RCORE-01 remains open.

Six findings were retained and checked by the root agent:

| Finding | Evidence before fix | Resolution under validation |
|---|---|---|
| Empty selected values reject Mesh/PointCloud replay | Runtime58: Cloud failures in result36; Mesh fails sources8 | Reuse native source reader, capture refreshed selected values and discrete indices |
| Full parent type incorrectly admits/rejects a selected subelement | Runtime58: Box.Vertex1 creates successfully but replay clears output | Preserve full parent snapshot; validate selected representation and native subelement identity separately |
| Units recaptured after scalar callbacks can differ from result units | Runtime58: numeric-only scalar units callback leaves success with nonempty output | Freeze document units before replay/setters and use that context for result and property |
| Recipe/scalars/original construction can change inside result callback | Runtime58 callback33: units, radius, fit, approximation, recorded, recipe, construction failures | Native GUI property barrier checks immutable recipe, links, units, construction and expected output values after each setter |
| Initial exemption survives Update=No creation | Runtime58 sources8: first resume leaves stale output | Consume initial publication exemption before testing Update setting |
| Support geometry alone accepts arcs or trimmed splines as whole Circles | Rust trim test: 13 pass / 1 fail, actual Cargo exit101 | Validate complete expected domain and shared endpoint identity; periodic spline full domain must match solver |

The initial Runtime58 result fixture passes 28/36, callback fixture 19/33, and
source/lifecycle fixture 4/8; each native host terminates normally, while the
fixture runner reports failed assertions. Those immutable artifacts remain RED
evidence. Runtime59 source fixes are not accepted until matching build/native
evidence is recorded.

Positive assessment: replay is independent of active command state; input0 and
source refs1..N use consistent indexing; full source ownership and freshness
checks surround publication; analytic center/normal/radius and periodic spline
basis are compared against the existing independent Rust replay; invalid replay
clears Shape/result together; detach retains paired bytes.

Declined judgments remain required work: document-close/removal safety and atomic
callback rollback; source-to-resolved native-plan correspondence; complete
original provenance and output attributes; sharing and aggregate budgets;
global conversion/deviation/maps; Rebuild and remaining consumers/families;
cold persistence, Undo/Redo acceptance for the new path; every original
RCORE-01 C01–C12/T01–T11 requirement. Producer inventories are candidate inputs,
not exact compiled-input certificates. The review does not certify full Rhino
History parity or completion of RCORE-01.

Root verification 2026-10-12T02:02:17+07:00: Runtime59 build actual exit0; all six fixes validated in named evidence. Primary113/113 and regression317/317,14 normal host exits,full Rust916/0/0. Four recorded FCStd archives cold reopened and replayed; real Undo/Redo verified for Center/Around/cloud/deformable. Root examined native artifact hashes and copied fixtures. The missing-helper context launch was excluded after exact owned process termination and passed with its helper on serial retry. Declined judgments above remain open; no new blanket review claim.
