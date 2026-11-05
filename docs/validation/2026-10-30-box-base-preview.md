# Box base preview — 2026-10-30

The user requested that, between the first and second points of a Box, the
preview show only the rectangular base. Previously the Corners/Diagonal/Center
preview derived a provisional height from the base width and drew 12 box edges.
It now draws four edges in the captured construction plane. After the second
point, the existing height stage still previews 12 edges and only the height
commit creates the solid. Cube and other construction modes are not changed.

With default OCCT features, `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`:

- Before the change, a focused controller regression failed: 12 preview edges
  were returned where four were expected.
- `cargo test -p om9-app --lib box_base_preview_tests`: three passed. The tests
  cover four signed quadrants, world-point and typed coordinate entry, a rotated
  construction plane, Corners/Diagonal/Center, no document transaction before height, cancellation,
  height preview, commit and Undo.
- Cargo compiled and linked the application but could not replace
  `L:/om9-target/debug/om9.exe` because the user's open process holds it. The
  freshly linked `debug/deps/om9.exe` was copied to
  `L:/om9-target/debug/om9-box-preview.exe` (SHA-256
  `fe418d17bdfa340b41e3a587cb2db13850146c09c62eb05d660342dc627b0129`).
  Its `--create-fixture` invocation succeeded. The existing process was not
  closed at that time. After the previous OM9 process exited, `cargo build -j 2
  -p om9-app --bin om9` succeeded and the standard executable was launched as
  PID 12764 with title `OpenMatrix9 — Untitled`; SHA-256 was
  `f950519a5cff3c31003e5384fadb0bccbd6be93564de998a6f3e6a195967d6ce`.
  UI pointer events and desktop preview remain unverified.

This is the user's requested OM9 interaction, not independent proof of Matrix's
original implementation. The newly launched executable contains the change.
