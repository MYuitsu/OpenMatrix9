# 3DM exchange: old debug adapter and current source

The layout next to the running debug application contains a native exchange
executable dated 2026-10-21, SHA-256
`4ea46bc769361a000f411e1945963214cfa2f5dc64c4b859974310359bad0560`.
The authored trimmed-face block export was retried explicitly with this binary
and failed during packet preflight with `missing/unknown record fields`.

The existing P1 native build directory was rebuilt from the current
`native/native_exchange.cpp` (SHA-256
`56c7e79984b526ff9c1c7f28d1245d64cb284da7e895c30a8ad3b6174cd214f0`)
using MSBuild Release and two workers. The rebuilt executable SHA-256 is
`8bdd11bcb529b9e55b7eeed5e4e80f5741cb2a4d1e4e1c0bd3040347d4817a6c`.
Using this explicit binary, the same block fixture passed with both editable
and opaque import modes, and the eight-test trimmed-surface 3DM batch passed
(seven export/reimport fixtures and one safe-refusal fixture).

Named batch `3dm-adapter-current-source` had three executions: one fixture with
the old debug binary failed before export, then the same fixture and the full
eight-test batch passed against the rebuilt current-source binary. This isolates
the reproduced mismatch to the old binary versus the rebuilt implementation;
it does not identify the exact historical field change or certify all protocol
packets. This is headless evidence, not a desktop acceptance test.

The running application layout was not overwritten: its executable and hash
are separate files and application commands load that layout while the user is
working. Two active OM9 windows must first be saved and closed before safely
replacing the pair and verifying the actual desktop layout. No new build tree
was created. Native peak, third-party drawings and independent Rhino remain
unverified.

## Existing debug layout updated

After confirming no `om9.exe` or native exchange process and no connected
live window, the already-built current-source P1 Release adapter was copied
to `L:/om9-target/debug` and its sibling SHA-256 file updated. Both the source
and resulting executable hashed to
`8bdd11bcb529b9e55b7eeed5e4e80f5741cb2a4d1e4e1c0bd3040347d4817a6c`.
`cargo build -p om9-app --bin om9 -j 2` succeeded reusing the existing target;
its executable hash was
`3ba8ae674bceb7547eb8c9ec948114499e3b1581ff0718ca2cbb8027842e567a`.

Batch `3dm-debug-layout-update`: two executions; nine distinct fixtures passed
in the full adapter suite (eight export/reimport, one safe refusal), and the
plane-with-hole fixture was repeated using `ThreeDmHost::from_directory` to
verify sibling executable/digest admission before export and reimport. This
uses the real debug directory through the directory loader; it does not launch
the actual desktop executable or test third-party drawings/native peak.

The updated `om9.exe` was also started in a private hidden window and reached
an empty idle document. Through its live command channel a 2 × 3 × 4 Box was
created, selected, and `ExtractWireframe` produced 12 curves; one Undo returned
to the Box and one Redo restored 13 objects. The private process was closed
afterwards. One initial live command connection closed and the same command
succeeded on retry. This verifies that executable/command path, not physical
mouse, visible rendering, save/reopen, or desktop 3DM import/export.

### Visible desktop attempt

Computer-use window inventory showed no OpenMatrix9 window. An attempt to
launch the existing executable through the Windows automation runtime failed
before launch with `GetCursorPos failed: Access is denied (0x80070005)`; a
subsequent window inventory showed no OpenMatrix9 window. No physical click or
visible image was tested. Separately, the previously prepared Rhino 5 batch
points into `H:/FreeCAD-src`, which was absent when checked, including its
recorded script path. No Rhino fixture was executed in this attempt.

### Actual executable 3DM command-line smoke

Batch `3dm-debug-executable-box`: two authored fixtures, two attempts. The
built-in four-object fixture was safely refused during export because its point
cloud has the unsupported runtime-only `hidden=true` attribute; it is not a
successful roundtrip. A 2 × 3 × 4 Box was then created and saved by the
actual app in a private hidden session. With both exchange environment
overrides removed, the actual `om9.exe --export-3dm` and `--import-3dm`
commands succeeded using the sibling layout, and `--inspect` reopened the
output. Each report contained one Box B-rep; before and after, display bounds
were [0,2] × [0,3] × [0,4], with 8 display topology vertices, 12 edges and
6 faces. These are display/topology checks, not independent exact-geometry or
Rhino equivalence proofs. The roundtrip correctly reported geometry-only
export limitations and retained-only/native-default import diagnostics.
One save and one state request initially lost their live connection; after
rechecking the same process, the save succeeded. The private process and only
the three explicitly named temporary files were removed after verification.
Visible desktop operation, Rhino interoperability and native peak remain open.

### Actual executable trimmed-face roundtrip

Named batch `3dm-debug-executable-trimmed-plane`: one authored fixture, one
execution, passed. The integration test saves a translated NURBS plane with an
outer boundary and a rectangular hole to an OM9 archive, then runs the actual
debug `om9.exe --export-3dm` and `--import-3dm` without exchange environment
overrides. The executable uses its admitted sibling native adapter. The output
archive reopens as one editable B-rep: ray queries hit the retained material,
miss the hole and miss outside the trim. The source archive remains unchanged.
The executable SHA-256 was
`a116c9f4c67ec19b1a4f803ddf102798f997559321868717a231fe0f5a15eeea`;
the sibling adapter is the previously recorded current-source adapter. The
test cleans up its unique temporary directory. This is a command-line test of
one generated plane, not a desktop export, independent Rhino validation, a
general trim proof, Undo/Redo test, or native peak measurement.
