# OM9-L2-03-03 — audit of the current code against the milestone records

Date: 2026-10-19. Authority order (user direction 2026-10-19): **Matrix9/Rhino5 documentation** first, Matrix9 decompiled
reference second, QymCAD Rust reference third (reference only; changed or ignored when it does not fit).

| L3 | Title | Status | Evidence | Notes |
| --- | --- | --- | --- | --- |
| OM9-L3-03-03-001 | Matrix Click Shift Ctrl Select | **PASS** | acceptance_viewport_selection (8), acceptance_selection_set (6), source_crosswalk + evidence JSON | Shift is add-only (fixed 2026-10-19), Ctrl toggles, plain replaces, locked/hidden never enter the set |
| OM9-L3-03-03-002 | Matrix Left Right Drag Selection | **PASS** | om9-scene selection::select_band (window all + no clipped; crossing vertices/segments/face containment), selection_overlay window/crossing, new dock-drag test | Marquee is the white rectangle; a drag starting on the dock starts no marquee |
| OM9-L3-03-03-003 | Matrix Ambiguous Selection Popup | **PASS** | pick_disambiguation (5) with type label + layer swatch and Esc preservation | Rows now show "label [Type]" and a layer colour swatch; Esc closes the chooser without touching the selection; stale ids are filtered by revalidation before choosing |
| OM9-L3-03-03-004 | Matrix Gumball Axis Drag | **PASS** | native selection replay 12/12 checks (move +5 mm, one Undo, CMD Undo/Redo, scale x2, rotate 90 deg, Esc keeps the redo branch); gumball_viewport (3); preview_drag (3) | GPU scene viewports recorded in the replay receipt |
| OM9-L3-03-03-005 | Matrix Gumball Alignment And Relocate | **PASS** | gumball_placement (4): World, CPlane (new), Object, GumballOrigin/Reset | CPlane alignment added 2026-10-19; relocating the gumball never moves the object |
| OM9-L3-03-03-006 | Matrix Gumball Numeric Edit | **PASS** | gumball_numeric (5): exact typed value + one Undo, NaN/empty/negative refused, gizmo stays | Typed numeric entry is the command line (MoveX 12.5) rather than an inline field on the handle; the L4 algorithm allows typed input from M01 |

## Changes made while auditing

- `Shift` click was toggling; it is now add-only (Rhino p33 and the L3-001 oracle). `crates/om9-app/src/session/layers.rs`.
- Ambiguity popup rows now carry the geometry type and a layer colour swatch, and `Esc` closes the chooser without changing the selection. `crates/om9-app/src/ui.rs`.
- Added `GumballCPlane` alignment; `gumball_pivot` takes the axes from the active construction plane. `crates/om9-app/src/session/gumball.rs`, `session.rs`.
- New tests: shift add keeps selection, dock drag starts no marquee, popup type + Esc, CPlane alignment, NaN/empty numeric input.

## Verification commands

```
cargo test -p om9-app --test acceptance_viewport_selection   # 8 passed
cargo test -p om9-app --test acceptance_selection_set        # 6 passed
cargo test -p om9-app --test pick_disambiguation             # 5 passed
cargo test -p om9-app --test gumball_placement               # 4 passed
cargo test -p om9-app --test gumball_numeric                 # 5 passed
cargo test -p om9-app --test selection_overlay               # 8 passed
python tools/validate_m02_selection.py (native replay, 12 checks PASS)
```

## Remaining gates (not signed here)

- **M02-UI human review** is still pending; this audit is not a reviewer signature.
- L3-006 uses typed command-line input rather than an inline numeric field on the handle; the L4 record allows typed input, but a human should confirm the interaction is what Matrix9 shows.
- Native BRep subentity coverage belongs to the separate BRep checkpoint, not to this L2 audit.

Changed-file SHA-256:

```
664ceaf48cc159523396d9b8270cbc7879dbadf5b0c036176eb8f7c84e1986a3  crates/om9-app/src/ui.rs
587b2fbced725dca491bd7f9b7d17e93572f0d70b32c072e1c933dd1b36801eb  crates/om9-app/src/session/layers.rs
f4784ced194a509afd25c79d402b880e3f45c2735e93447824442a53adc39e70  crates/om9-app/src/session/gumball.rs
2e31183233b8bc0c818224fe148641b1c6bb793db55da26fe62d99073b37417d  crates/om9-app/src/session.rs
```
