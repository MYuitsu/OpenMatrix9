# Left MAIN MENU command surface

The user requested hiding the horizontal tabs, icon palette and view-tab strip
above the canvas on 2026-11-17, keeping the left MAIN MENU shown in their second
screenshot. `MatrixApp::show` no longer draws that panel. The region planner
reserves zero height for its retained serialized identity, recovering 67 logical
points for the canvas. Older toolbar state migrates to hidden/zero without a
spurious size warning. The top text menu and Command area retain their functions.

The left category palettes and the eleven Top11 icons above DISPLAY keep their
existing dispatchers. File controls are exercised through the left File palette.
This is a layout change: canonical geometry, producers, picking and CAD tolerances
are unaffected; an extra geometry preview or worker is not applicable.

## Evidence

- A real debug window was opened and observed with the horizontal strip absent,
  the left MAIN MENU/Top11 present, and all four viewports directly below Command.
  The launch receipt records PID 62052 and executable SHA256
  `B4C89AE5C5D83FC84A57B4F93A34272321009F480A79784A5C11ED26ABB2C426`.
  This observation predates the final legacy-toolbar-warning cleanup; it is not
  a human milestone sign-off.
- Focused headless suites: `chrome_actions` 30, `file_widgets` 21,
  `workbench_regions` 8, `workbench_regions_sizes` 10 and `left_stack_layers` 8:
  **77 passed**. Tests cover left File actions and unsaved prompts, command draft
  and IME protection, category/display actions, four-view layout, narrow/high-DPI
  sizes, docking and sidebar order.
- Initial failures came from obsolete toolbar dimensions/targets and an implicit
  focus assumption in the migrated CMD fixture. Expectations now use the visible
  left controls, explicit CMD focus and the recovered canvas height.
- The running preview later contained user edits. It was preserved. Integration
  targets were built with `cargo rustc --test ... -j 2` and run with the existing
  debug DLL directory on PATH; this avoids replacing the running `om9.exe`.

Receipts and per-target logs:
`build/standalone-validation/ui-artifacts/left-menu-only-20261117/`.
Existing target: `L:/om9-target`; no additional target directory was created.

Remaining: final warning-cleanup build has not been reopened while the user is
working in the preview. Older opt-in horizontal-toolbar audit scripts are
historical and do not prove the new layout. Ring Rail's complete command and
the 31-command Tools plan remain separate, incomplete work.
