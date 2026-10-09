# OM9-HISTORY-001 — verified Surface dependency slice

Native SurfaceHistory implements the shared graph contract for Sweep1, Sweep2
and Loft: ordered parents, source and parent-placement updates, two generations,
direct child Shape edit detachment while descendants remain linked, and native
Undo restoring parents. Invalid/deleted inputs clear stale geometry; cycles and
foreign-document parents are rejected. Per-object UpdateHistory suspends/resumes
existing links. FCStd preserves type, JSON settings, inputs and parent frames;
fresh-process restore and subsequent source edits rebuild without workbench
activation.

This validates the Surface graph slice of OM9-HISTORY-001. It does not establish
global Record/Update/Lock/BrokenHistoryWarning, Gem/Builder Settings/Cutters,
Join/Split/Trim/topological renaming or all-command History compatibility.
Direct detachment is the exact shared spec contract; it is not attributed to
an unverified Matrix9 Addendum chapter. Sweep-specific automatic updates are
documented in Matrix8 Book1 PDF183/186.

[Options and host bounds](surface-advanced-options.md),
[Sweep1 History](OM9-SURFACE-002.md), [Sweep2 History](OM9-SURFACE-004.md),
[native validation](../validation/2026-10-09-surface-constraints-history.md).
