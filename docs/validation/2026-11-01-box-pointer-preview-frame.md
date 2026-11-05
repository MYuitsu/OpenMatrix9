# Box pointer preview frame — 2026-11-01

The user reported slow green preview while drawing a Box. UI inspection found
that `viewport_ui` painted the active command preview before updating the hover
position from the current pointer event. The painted preview could therefore
lag a pointer event and remain stale until another frame. The pointer projection
and snap calculation now occur before command-preview painting. When the hover
changes during an active command, a repaint also updates viewports that were
already painted in that UI frame. Selection, document data and command geometry
are unchanged.

With default OCCT features, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`:

- `cargo test -p om9-app --lib box_preview_uses_the_current_pointer`: one
  simulated egui test passed. Looking Down and Perspective pointer-move events
  produce the four green Box base-preview edges for the current hover in the
  same UI frame. This is not a desktop input or frame-time benchmark.
- `cargo test -p om9-app --lib box_base_preview_tests`: three passed.
- `cargo test -p om9-app --test command_options_interface`: 74 passed.
- `cargo build -p om9-app --bin om9` succeeded for the debug executable,
  SHA-256 `e31909a6a6d9ecc40d13067ee5a9071df8c42d117ed957876684bc065a97efb3`.

At inspection the user's running instance was `L:/om9-target/release/om9.exe`,
PID 31636, displaying `manual-test.om9 *`. It was left running and untouched.
The new debug executable was not launched over that unsaved session. No desktop
pointer capture, release rebuild, latency benchmark or proof that all perceived
slowness is resolved is claimed.
