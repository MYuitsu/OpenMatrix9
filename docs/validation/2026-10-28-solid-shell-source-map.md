# Solid Shell: bounded registration and kernel scope

The recovered `modMillC_Defaults.bas` procedure `Proc_198_35_1A2F840`
registers `SolidShell` near address `01A353EE` with external `_Shell` near
`01A353D6`. A Ghidra definition exists at `FUN_01a2f840`. This registration
establishes the menu-to-command association only; it does not expose the
external command's solver, default thickness or selection implementation.
No recovered code or artwork is reproduced here.

The normalized Shell reference calls for selecting faces of a closed solid,
leaving at least one face, and offsetting inward. The standalone OCCT bridge
now accepts multiple distinct face indices and the Rust adapter requires one
closed solid, positive thickness and a retained face. A two-opposite-face box
fixture measured an inward wall result of 360 mm³; index order, invalid input,
duplicate faces, all faces and an open shell were checked. Rust owns validation
and document state; the native exception is limited to the OCCT operation and
snapshot. App menu/CMD still expose one numeric face only; viewport selection,
multi-face command state, large native allocations and actual desktop UI
remain open. This is not full Shell parity.

The following application checkpoint adds staged root-BRep face selection,
positive Thickness input and one atomic replacement. An egui simulation picked
two adjacent faces in separate views; controller tests measured two opposite
openings, invalid/all-face retry, Cancel and Undo/Redo. The historical numeric
single-face form remains available. Instances, complex topology, original
default thickness, actual desktop/human review and native peak memory remain
unverified.
