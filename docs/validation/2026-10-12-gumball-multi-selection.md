# Gumball whole-object multi-selection — 2026-10-12

Version **0.3.5-202610121520** repairs disappearing handles after Shift/window selection
in Object Alignment. The released 0.3.4 binary selected all three whole owners
in all four views, but four of 32 probe checks failed because its single-owner
Object-frame guard detached the overlays. CPlane/World remained functional.

Safe Rust now resolves multiple whole canonical roots to the active construction
basis while retaining the requested Object preference. The native adapter uses
the existing union-bounds center and immutable ownership/transaction pipeline.
It chooses no arbitrary first owner. Single-owner frames and ambiguous or mixed
component rejection remain unchanged. This is an explicit OM9 policy following
the user's shared-selection request. [Rhino 5 Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
documents multiple selections and their nonpersistent relocated widget; it does
not prove this basis algorithm for every Rhino representation.

The independent native regression passed **143 checks**: actual Shift and
left-to-right window pointers in CPlane/World/Object from all four views; literal
common center and construction bases; per-overlay Scale extents; twelve native
Move/Scale/Rotate results checked against analytic vertex transforms; one Undo
per batch; Undo/Redo of every owner; retained primitive types and selection;
Ctrl relocation without model edits; deselect pivot reset; Esc cancellation;
mixed whole/face and ambiguous component rejection without model/Undo edits.

The first new fixture completed its 124 geometry/UI checks but blocked on an
unsaved disposable test document during exit. It was stopped only after its
report completed. The final fixture explicitly closes its own document; all
final cohort processes exited zero. The old geometry-only fixture expected
whole multi-owner rejection; that obsolete oracle now checks literal shared
center/basis and two independent staged occurrences. Component rejection
oracles were retained and extended. Reviewer recheck found no Critical or
Important issues in the code or adjusted tests.

Final checks on the same compiled plugin: **31 native processes / 1,099 checks**
and **848 Rust tests**. Native suite SHA256 `cbd769475183d867877862d3dde4c9efa9d654a7eb82d953ed609dca193bc96b`;
plugin SHA256 `c013417b56abe9cc2613d88047161660fbb7957cba5aace3578889440aa1af33`;
stock host SHA256 `7b38c5d5aa2c74f830539c1c5c44cbf6e3e8e157292c718f9ee24ac6d3e12cf6`.
[Case hashes and source fingerprints](2026-10-12-gumball-multi-selection.json).
Reports and four-view screenshots are retained under
`C:/Users/nguye/.codex/tmp/gumball-selection-native-verified-20261012`.

Only the standalone OM9 plugin target was rebuilt using the checkout's existing
plugin cache and prepared matching stock SDK. No FreeCAD core rebuild or host
copy was made. Current main's README/installation cleanup was fast-forwarded;
those changes touched documentation only. Prior main/CAD/layer evidence remains
historical rather than being counted as newly rerun by this task.

At the user's request GitHub Actions only builds/packages, with
`validation_kind: build_only` and `tests_executed: false`. Its exact-source ZIPs
and public release digests are verified separately. Existing stock-host limits,
unsupported dependency graphs, Clayoo/SubD, SmartTargets and full Rhino/P0 parity
remain outside this fix. The existing Rust dead-code warning is not a test failure.

Version increments **cc from 4 to 5** for this completed bugfix task; aa/bb remain
0/3, with the build timestamp in Asia/Saigon.
