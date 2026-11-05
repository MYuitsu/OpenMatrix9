# STL linear tolerance dialog

P3a partial delivery: both the menu and typed command open the same STL quality dialog, initialized from the effective document tolerance. A valid positive finite requested linear tolerance is passed to the existing profile export path; invalid values cannot proceed to the native file picker. Cancel and Escape discard the request. The dialog states that final geometric deviation is not yet certified.

Windows, default features including OCCT, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: two `stl_quality` headless egui tests passed, including Escape through the full UI with document and selection preserved. The shared menu/command invocation test passed. The `stl_export` integration suite passed 18 tests. These tests do not exercise native file-picker interaction, text entry on a desktop, actual presentation, or performance. The preexisting OM9 session was not touched.

The dialog currently changes only linear tolerance; the mesher's angular criterion remains fixed at 0.3 rad. Export and the native picker remain synchronous, and source/selection are not captured at initial invocation. Accuracy, full error bounds, immutable background capture, cancellation during export, memory ownership and the other gates in the plan remain unverified or unimplemented.
