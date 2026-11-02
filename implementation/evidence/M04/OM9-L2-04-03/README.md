# M04 L2-04-03 jewelry ring builder family

`OM9-L2-04-03` (06-febe-jewelry-ring-builder-family). The audit found no ring builder code
(QymCAD has no such module), so the family is implemented in portable Rust for the sections and
the OCCT kernel adapter for the B-Rep body, reusing the OM9 builder session, producer registry,
style library and history journal.

## What is implemented

- `om9_core::ring_builder` — one surface of revolution around the ring axis with per-kind
  section modulation. `RingParams::validate` refuses the impossible sets, `quad_mesh` publishes
  the sampled sections, `build` returns a watertight mesh solid, `stone_frames` publishes the
  Eternity placement ladder, and `inspect_measures` / `inspect_geometry` is the one final
  closed/positive-volume gate for both representations.
- `om9_kernel::creation::sewed_solid` — the OCCT adapter: one planar face per quad corner
  triangle, sewn into exactly one closed shell, checked for free edges and a positive volume,
  then published as an owned B-Rep snapshot. `RingFamilyBrepProducer` registers it for the four
  commands, so the product build commits a B-Rep ring body while the preview keeps the same
  sections.
- Four builder modes — `BuilderKind::EternityBand`, `SignetRing`, `RaisedBand`, `AwardRing` —
  wired into the panel tabs, the command line (`EternityBand`, `Signet Ring`, …), the style
  targets and both producer registries (core analytic and OCCT).
- Presets — `BuilderStyle::snapshot` records the ring fields; `om9.builder_style.v1`
  roundtrips them and `apply` restores the same body.
- History — reopening the recorded feature and editing the diameter updates the same owner
  id through `Mutation::HistoryUpdate`; undo/redo restores the previous volume.
- Final validation — a closed ring is accepted with a positive volume in both representations;
  a deliberately open mesh, an open quad set (missing face) and a wall below the minimum are
  refused with typed errors.
- Viewport handles — `diameter`, `width`, `wall` on every mode plus `stone`, `raised`,
  `shoulder`, `top_width`, `top_length` where the mode owns them; each grip drags the same
  `BuilderSession` field the panel edits, through `adjust_field`, so the two paths cannot drift.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
| `OM9-L3-04-03-001` | Eternity Band Builder | automated_verified_manual_gate_open |
| `OM9-L3-04-03-002` | Signet Ring Builder | automated_verified_manual_gate_open |
| `OM9-L3-04-03-003` | Raised Band Builder | automated_verified_manual_gate_open |
| `OM9-L3-04-03-004` | Award Ring Builder | automated_verified_manual_gate_open |
| `OM9-L3-04-03-005` | Ring Builder Style Presets | automated_verified_manual_gate_open |
| `OM9-L3-04-03-006` | Ring Builder History Updates | automated_verified_manual_gate_open |
| `OM9-L3-04-03-007` | Ring Builder Final B-Rep Validation | automated_verified_manual_gate_open |

## L4 packages

Each L3 directory under `implementation/evidence/M04/<L3>/` carries its own L4 artefacts:
`source_crosswalk.json`, `api.md` + `positive.json` + `negative.json`, and one
`OM9-L4-...-0X.json` receipt per IMPLEMENT / INTEGRATION / REGRESSION / ACCEPTANCE /
ADVERSARIAL role — **34 receipts, 47 planned L4**. A receipt names the command that ran,
its exit code and the hash of its output, and keeps the M04-UI gate open.

## Verification

- `cargo test -p om9-core --lib ring_builder::tests`
- `cargo test -p om9-core --test ring_family`
- `cargo test -p om9-app --test ring_family_panel`
- `cargo test -p om9-kernel --test ring_family_brep --features 3dm-brep`
- `cargo test -p om9-core --test builder_session --test builder_style`
- `cargo test -p om9-app --test builder_panel --test builder_styles`
- `cargo clippy -p om9-core --lib --no-deps -- -D warnings`
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: no human has reviewed the four builders on screen and no pointer
  frame records them.
- A smooth NURBS B-Rep: the committed body is a closed solid of planar triangles sewn from
  the sampled sections (chord error about 5 um), not a NURBS surface.
- Gem solids: the Eternity builder publishes placement frames only.
