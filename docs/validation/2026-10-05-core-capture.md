# ViewCaptureToFile validation

Scope: OM9-VIEW-005 only; full01-core completion is not claimed.

Rust original-command-name test failed before mapping and passes after implementation. All33 Rust tests, fmt check, Clippy with warnings denied and release build pass. CoreViewControls and CurveController compile; the separate capture module links and loads in the patched native host.

The first automation attempts timed out after cancel succeeded because programmatic QFileDialog selection/accept did not follow the visible filename editor flow. Diagnostics preserved partial progress and dialog state. The fixture now navigates to the folder, sets fileNameEdit and clicks the real Save button with Qt mouse events. This completes the actual file-dialog flow and leaves the production dialog unchanged.

Native results: 13 checks at100% (`build/capture-isolated-1/241d8a1019fc45bc9250981cac1f5ea0/results.json`), expanded16 checks at150% (`build/capture-isolated-1.5/3c6ffd4441b24a73897429c78c4ce335/results.json`); both native process exit zero. Outputs decode as their selected formats, Unicode filename succeeds, physical viewport dimensions match, cancel/overwrite rejection preserve files, and invalid extension does not write. The expanded test includes newly active viewport capture, read-only task behavior and capture while Line waits for a point.

The fixture disables native Windows dialogs in its process; desktop mouse and that native backend are not proven. Feature contract and limitations: `docs/features/OM9-VIEW-005.md`. The user's existing preview processes were preserved; only stalled test processes were terminated.

Final expanded16 checks also pass at200% DPI with process exit zero: `build/capture-isolated-2/e04a26a7b148463593faff4e210dd8fa/results.json`. This includes physical pixel-size matching at the higher scale.
