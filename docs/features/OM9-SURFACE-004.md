# OM9-SURFACE-004 — Sweep 2 History

CMD alias `gvSweep2History` reuses the native Sweep2 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF186 / printed176.
The manual describes automatic rebuilding after edits and Closed/Flip/
Maintain Height options. Native output preserves two ordered rails, sections,
all component subreferences, options and parent frames.

Closed defaults Yes for closed rails with at least two profiles; open-rail
closure is unsupported. Maintain Height defaults Yes with one profile; the
multiple-profile host strategy cannot implement it. Reverse/Flip is per input.
Other geometric bounds match [Sweep2](OM9-SURFACE-003.md).

The native dependency graph handles source/parent changes, suspend/resume,
invalid/deleted inputs, direct child edit detachment, native Undo and fresh-process
FCStd reconstruction. These lifecycle rules share the bounded
[advanced contract](surface-advanced-options.md). Dedicated History menu/F6
placement, global Record/Update controls and full Matrix parity remain unverified.
This feature remains partial. See [native validation](../validation/2026-10-09-surface-constraints-history.md).
