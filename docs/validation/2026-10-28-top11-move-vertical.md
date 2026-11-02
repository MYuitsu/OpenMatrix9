# Top 11 Move Vertical — 2026-10-28

The normalized M03 Move reference specifies Vertical relative to the active construction plane. The private recovered `modTopMenu` procedure `Proc_199_7_1A51E60` identifies `TopIconMove` and `! _Move` in its bounded disassembly; it establishes the toolbar command association, not the transform algorithm. `FUN_01a51e60` is present in the Ghidra export; its full behavior and COM calls were not audited. Do not treat recovered strings as portable product source.

Before: the Top 11 Move session had no Vertical option and typed CMD points fell through to a different session, yielding `Missing("active command")`. After: the shared command option interface toggles Vertical; the signed displacement is projected on the active CPlane normal. Typed coordinates use the common coordinate parser and reach the same Move point handler as world picks. The default remains unconstrained.

Validation, Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- Regression before fix: `cargo test -p om9-app -j 2 --test top11_options move_vertical_uses_active_cplane_normal_and_undo` failed because the option was absent.
- After fix: `cargo test -p om9-app -j 2 --test top11_options --test top11` passed 4 and 19 tests. The two new tests cover option activation, a rotated CPlane, signed world CMD points, invalid input, cancellation and Undo.
- `python tools/public_source_audit.py` passed.

Source SHA256: `session.rs` 4bfa937dddbacbd2102251dd3dcd48aff5f914be2daecb0e59912ce4b4b528ec; `session/top11.rs` d2008f2be31a7dd92ee07a4d126cfbb0dd9fd138da2ef54b01c9ee7bfe915228; `session/command_options.rs` 4b9a9a63102f93bcf83117f37c161cd87d45742b05d70744b56ddebb734296a2; `tests/top11_options.rs` f6634391e0e77be2c43dd81488530e05feb6d739864634a609bd29eb931b1a91.

Limits: no real UI click, rendered preview, GPU-present or manual M03-UI signoff in this batch. A live Move preview is not established. Select-after-command, Mirror and Rotate options, and surface/polysurface and multi-target edit paths remain open as detailed in `2026-10-28-top11-review.md`. This batch is one part of the requested Top 11 work.
