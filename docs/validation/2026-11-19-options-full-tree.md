# Options tree and detail pages — 2026-11-19

The modal opens through the RhinoOptions icon, the Options and RhinoOptions commands, and File Properties or DocumentProperties. UndoSettings opens General. The two reference branches share a scrollable tree and individual detail panels. Options and the viewport title menu share the 34-mode catalog. The reference illustrates the tree. The OM9 dialog uses its own styles.

- Profile settings are wheel Zoom Scale, single-click viewport maximize, and separate grid line and axis visibility for Wireframe and Shaded. The default zoom scale of 0.9 produces a span factor of 0.81 for a single notch. The GPU overlay and displayed-frame comparisons include grid visibility.
- The settings use the existing owners for grid snap, Ortho, Planar, Project, object snaps, Smart Track, Gumball, and current dimension defaults. The dimension defaults apply to new annotations in the session. The Units page reads the current document's units and tolerances. General uses the previous language and Undo controls.
- Unsupported fields are shown as unavailable. The original values of named style presets remain unverified. Only Wireframe and Shaded are selectable in the renderer. Clayoo, RhinoScript, Layout units, and the other unconnected pages supply navigation and field information.

Verification is scoped to this checkout and the normal Rust application library. The workspace test build encountered incompatible copies of the add-on ABI in Gem Tools and Surfaces. An isolated test manifest references the same application and integration test with the normal default features. It excludes dev dependencies unrelated to this Options test and reuses the existing target. Captured output is in `build/standalone-validation/ui-artifacts/2026-11-19-options-full-tree/options-ui.log`.

Four Windows egui tests passed. The Options tests submit the CMD command, select Grid, exercise draft discard through Esc and Cancel, and apply Grid Snap through OK with the drawing unchanged. The viewport test checks the written preference and toggles maximize by clicking the Perspective title once. Both existing navigation tests passed, including the default 0.81 zoom factor in one direction and its reciprocal in the other. The public-source audit reported zero errors.

Desktop input and DPI review, original preset parity, GPU pixel readback for the grid controls, and the unconnected settings remain open.

## Options typography

On 2026-11-20 the dialog received its own text sizes, semibold headings and field labels, higher contrast for labels and statuses, and wider spacing between the field and status columns. The rebuilt default-feature Windows executable opened Options from Info & Settings and showed both View and Appearance with the new style. Both Options egui tests passed again, and the public source audit returned zero errors. The desktop check used one Windows display configuration.
