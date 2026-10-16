# Matrix9 viewport title menu implementation

The full goal remains RCORE-00 plus the six-image UI specification. This plan
implements its viewport-menu requirements; it does not close the full goal or
enable unverified renderers, native family codecs or menu subcommands.

Sources: MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md S6/M9IMG-09–11,
OM9-VIEWPORT-001, OM9-INFO-010, OM9-DISPLAY-008 and current workspace contracts.

1. Capture RED pointer/behavior regressions for exact menu vocabulary/order,
   target-view radio state, independent modes, focus/draft and small-window
   scrolling. Keep geometry/selection/Undo unchanged by view-only actions.
2. Add a safe Rust typed menu registry with stable item identities, separators,
   submenu markers and explicit unsupported reasons. Only actual accepted
   actions become enabled. Rendered/plugin labels never imply backend support.
3. Bind popup actions to coordinator/document context and a runtime view owner,
   rejecting closed/replaced/foreign/stale views before mutation. Use the same
   maximize/restore reducer as title double-click; keep camera and pending
   command input. Do not force CMD focus while the popup needs keyboard input.
4. Render the complete menu with DPI/work-area bounded vertical scrolling,
   exact labels and actual radio state. Retain unsupported entries visibly and
   honestly. Submenus absent from the source are not populated with guesses.
5. Run focused then workspace checks, warning-strict Clippy/formatting, release
   build and own native framebuffer/event replay. Record supported scope,
   stale/cancel/focus evidence and unimplemented render/submenu operations in
   the live ledger; keep all full RCORE-00 gates open.
