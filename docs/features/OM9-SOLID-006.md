# OM9-SOLID-006 — Extract Surface, partial standalone implementation

The M03 reference describes selecting surfaces and separating them from a
polysurface; its right-click variant sets Copy=Yes. The current standalone
numeric-index path accepts one preselected multi-face BRep: the default
extracts one face and replaces the source geometry with the remainder in one
transaction. `ExtractSrf <index> Copy=Yes` retains the source. The output
uses the current layer and inherits source placement; Undo restores the
original document. Invalid inputs fail without modifying the document.

Interactive Copy and OutputLayer Current/Input work in the session. Right-click
starts Copy=Yes, including the tested preselected-object case. Multiple root
BRep faces on one or more objects can be selected in the viewport and
submitted with Enter; Undo removes the last pending face and Cancel does not
commit. With Copy=No, selecting all faces replaces the source with separate
faces. Overlapping root BReps offer a face-candidate chooser; stale sessions,
sources and views are refused. Ctrl + primary click removes a selected face,
including via the overlapping-root chooser. Instance faces, multiple hits within one root,
general disconnected remainder topology, F6, SDK/History and actual
desktop/human UI acceptance remain open.
OutputLayer Current is current OM9 behavior and is not evidence
of a recovered Matrix default. See
`docs/validation/2026-10-28-solid-extract-source-map.md` and
`docs/validation/2026-10-28-solid-audit.md`.
