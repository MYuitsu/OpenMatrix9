# Matrix9 S6 viewport-title menu — 2026-10-14

The supplied S6 image is a vocabulary/layout reference. This is a measured
implementation slice of M9IMG-09–11; none of the twelve complete screenshot
acceptance cases or M01–M12 gates is promoted.

Rust owns an ordered typed registry for every visible S6 label, separator and
submenu marker. Maximize/restore and Wireframe are enabled. Shaded, Rendered,
plugin presets and the unimplemented submenu operations are visibly disabled
with a reason. No unseen submenu contents are inferred.

Actions capture coordinator/document, runtime view and layout owner identities,
view state, a shared monotonic layout generation and a Controller-owned action
generation. Accepted actions cannot be replayed after cloned workspace rollback
or a new layout clock with copied view states. Camera/CPlane/mode remain exact
across maximize/restore. Wireframe changes only the captured view; a no-op keeps
its view revision and model Undo/Redo unchanged.

The menu focuses its first enabled item and contains Tab/Shift-Tab/Up/Down.
Command input waits while the menu owns the frame. Escape, stale-view closure
and an outside pointer gesture through release preserve the numeric draft,
pending command, selection and model. A subsequent independent canvas click
still accepts the pending Point. Long-menu scrolling exposes the last entry in
a 1280×500 headless window; the scrollbar is bounded to the available work area.

Validation: 17 focused menu tests (included in the workspace total), 360 workspace
tests/23 separately executed native Rust integration tests/69 suites, formatting
and warning-strict all-target Clippy. Independent review reproduced and verified
the rollback, fresh-clock, keyboard, dismissal-click and stale-Escape regressions.

The release native winit/wgpu window replay passes six checks: popup opening,
keyboard Maximize, keyboard Restore with exact view states, Escape, outside-click
dismissal and reopening. The literal pending Point draft remains `1,2,`; objects
and model Undo remain zero. The 1600×1000 framebuffer was inspected. The owned
process has 84 loaded modules, no child processes and no legacy host dependency.

Receipts: [native UI](2026-10-14-viewport-title-menu/ui-receipt.json),
[workspace](2026-10-13-standalone-workspace.json),
[plan](../superpowers/plans/2026-10-14-viewport-title-menu.md).

Unverified: Shaded and other render backends, all disabled viewport operations,
high DPI/multiple monitors/OS IME/task-modal interaction, GPU geometry caching,
large-scene performance and complete Matrix9/RCORE-00 acceptance. Arbitrary
external public-field mutations restored byte-for-byte are not an observed
mutation lifecycle; the captured fingerprint and supported reducers are scoped
to the checks above. Native CAD/exchange command acceptance remains separate.
