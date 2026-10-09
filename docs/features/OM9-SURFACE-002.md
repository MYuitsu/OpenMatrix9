# OM9-SURFACE-002 — Sweep 1 History

CMD alias `gvSweepHistory` reuses the native Sweep1 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF183 / printed173.
The manual describes automatic rebuilding after rail/profile edits and Closed
Yes only for a closed rail with at least two profiles.

Native `OpenMatrix9Gui::SurfaceHistory` retains ordered curve subreferences,
options and parent-frame dependencies. Direct child Shape edits detach its
parents while descendants remain linked, according to the shared OM9-HISTORY-001
contract. Undo restores those parents. Invalid/deleted sources clear the result
and report an error. Source geometry is preserved; Preview, one-transaction
commit, Cancel, Undo/Redo and FCStd restore share Sweep1 behavior.

Closed defaults Yes only within the supported closed-rail/two-profile case.
Reverse/Flip is per input; the remaining geometric bounds match
[Sweep1](OM9-SURFACE-001.md). Full Matrix History/global controls and dedicated
History menu/F6 placement remain unverified. This feature remains partial.

[Shared advanced contract](surface-advanced-options.md) and
[native validation](../validation/2026-10-09-surface-constraints-history.md).
