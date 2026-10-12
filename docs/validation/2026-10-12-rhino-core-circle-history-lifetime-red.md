# RCORE-01 — Circle History owner lifetime failure, 2026-10-12T02:47:19+07:00

**Status: in_progress. Full RCORE-01 and Task2/3 remain open.**

Production source, Runtime59 plugin, selected App19/GUI and the shared SDK remain
unchanged. No Rust or plugin/core build is claimed. This fixture is a single
isolated native lifetime diagnostic with an independently owned process/profile.

The recorded Circle and existing GeometryResult are confirmed before the probe.
After editing Source from(3,4,5) to(6,8,10), a callback on the nonempty replay
GeometryResult executes once and calls App.closeDocument on the owning document.
It persists `phase=after_close` and `document_alive_after_callback=false`.

Native recompute does not return within the runner's300-second limit. The runner
kills only its exact Start-Process handle, PID56600, and records
`has_exited=true`, `accepted=false`. The tool session terminates with runner
exit1. The launch receipt retains a null natural host exit; a crash code is not
invented. No fixture-completion verdict exists. This is **not** a green suite or
a completed RED assertion count. The one passing assertion proves setup only.

The callback's durable owner-closed observation directly contradicts the required
replay-owner lifetime invariant. The absence of return strengthens the need for
protection before reading native metadata/source pointers, through rollback and
journal destruction. It does not identify the exact native failure mechanism.
User FreeCAD processes, shared binaries, models and prior artifacts are untouched.

The [proposed native savepoint design](../superpowers/specs/2026-10-12-circle-history-native-savepoint-design.md)
preserves all three caller states, stages a strong native journal merge, rejects
observer/structural/history mutation before it changes state, and keeps native
error cleanup inside the lifetime barrier. It is proposed, not implemented or
verified. A native App target build is justified only by the concrete missing
private journal/lifetime API; reuse the single SDK/object cache and isolated App
outputs, with the matching plugin/native import and selected test host.

The prior semantic RED355/432 across48 cases remains valid and unchanged. This
lifetime witness adds a separate contradicted requirement rather than promoting
those passing assertions into broader acceptance. All original adapters, C/T
fixtures, other consumers, conversion/maps, budgets/sharing and persistence gates
remain required.

Next: Review the proposed native replay-savepoint design, then implement the checked native property journal and lifetime/history/observer barriers before Circle reads live sources. Preserve all caller states; use the unchanged semantic432 RED and require the close callback to complete normally with owner alive and paired typed rejection. Continue every original RCORE-01 requirement.

[Machine evidence](2026-10-12-rhino-core-circle-history-lifetime-red.json).
