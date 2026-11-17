# Add-on speed audit: Curve, Surface and Solid menus (2026-11-13)

Measured on Linux x86-64 in a cloud container, release build. Windows times differ, but the ratios between commands do not.

## How

**Through the controller (`crates/om9-app/tests/addon_bench.rs`, ignored benches).**
- A whole command with its one commit, and one pointer move.
- Each is run in a document pre-filled with 0, 5,000 and 20,000 objects.
- The same steps are run with OM9's own built-in commands for comparison.

**Every add-on step (scratch timing hook, not committed).**
- Each crate's `Interactive::step` was wrapped with a timer, and the Curve, Curve tools, Surface and Solid test suites were run through it:
  - 124 commands;
  - about 5,800 steps;
  - after every step that leaves a command waiting, three pointer moves (hover previews) were timed from that state.
- The inputs are those of the tests, which check the geometry against exact formulas.

**Kernel (OCCT 7.6 standalone harness, Linux).**
- `face_nurbs` on plates with k × k drilled holes.
- Sewing all their faces back, pairwise and in one pass.
- Checked against the Windows CI (OCCT 8).

## Results

### Through the controller

| Document | Built-in, whole command | Add-on, whole command | Add-on, pointer move |
|---|---|---|---|
| empty | 0.13 ms | 0.14 ms | 0.02 ms |
| 5,000 objects | 50 ms | 51 ms | 0.02 ms |
| 20,000 objects | 269 ms | 280 ms | 0.03 ms |

Offset, Loft, Sweep1 and Revolve behave the same way:
- the add-on takes the built-in's time (× 1.0–1.1);
- a pointer move does not depend on the document size (Revolve: 0.5–0.75 ms).

### Every step

116 of the 124 commands answer a pointer move in under 16 ms (one frame at 60 Hz) and finish in under 0.1 s. Most answer in 0.001–2 ms.

Pointer moves that lagged (median per move), before → after this change:

| Command | Before | After: first move after a change | After: next moves |
|---|---|---|---|
| Offset Curve on Surface ("Side to offset") | 709 ms (worst 779) | 88 ms (worst) | 0.04 ms |
| Refit to Tolerance (curve) | 66 ms (worst 140) | 70 ms | 0.003 ms |
| Unroll | 41 ms (worst 113) | 43 ms | 0.016 ms |
| Rebuild (curve) | 31 ms | 35 ms | 0.006 ms |
| Offset Curve Normal to Surface | 50 ms (worst) | 8 ms (worst) | 0.02 ms |
| Variable Fillet | 10 ms (worst 30) | 11 ms | 0.01 ms |
| Profile Sweep | 8 ms | 9 ms | 0.007 ms |
| Patch | 3.4 ms | 5.3 ms | 0.017 ms |

Offset Curve on Surface's click also went from 690 ms to 0.5 ms: it now reuses the previewed offset.

How:
- **Steady previews** (`om9_addon_geom::memo::steady_hover`). The commands listed in `STEADY_PREVIEW` never read the pointer of a `Hover` input; this was checked in their code. A pointer move with the same command, state, settings, plane, pick and selected geometry returns the previous move's reply.
- **Warm-started closest points** (`Srf::closest_st_near`). Points along a curve, and a walk across a surface, search from the last answer instead of sampling the whole surface (33 × 33) every time. The results are as close or closer: within 1e-4 mm of the whole-surface search, which stops at about 1e-5 mm.
- **Kept results.** Each side's offset on the surface is kept. For the normal offset, the curve's points on the surface and the normals there are kept, so a new distance only scales them.

Still above 0.1 s, once per command (not per pointer move):

| Command | Time |
|---|---|
| Project | 0.4–1.1 s |
| Shell (add-on mesh, build without the kernel only) | 0.3 s |
| Loft with Refit | 0.13 s |
| Helix (long) | 0.12 s |

### Add-on interface (trimmed surfaces and B-Reps)

| Item | Time |
|---|---|
| `face_nurbs`, 127 faces (cached per shape) | 65 ms |
| `face_nurbs`, 70 faces | 27 ms |
| Sending a 127-face solid to a `brep_data` command | about 2 ms per step (191 KiB JSON, written and read) |
| Running one Rhino trim loop along its surface | 0.5 ms per step (bicubic), 0.04 ms (plane) |
| Building one trimmed face for sewing | about 11 ms |

Sewing add-on faces back into a body:

| Faces | Pairwise (`sew_all`) | One pass (`sew_many`) |
|---|---|---|
| 10 | 54 ms | 7 ms |
| 31 | 1.7 s | 48 ms |
| 70 | 23 s | 121 ms |
| 127 | 153 s | 354 ms |

`sew_faces` now uses `sew_many`. A plate with 6 × 6 holes, read face by face and sewn back (42 faces), takes 1.16 s in total on the Windows CI, most of it building the 42 faces. `sew_all`, which Join uses, is unchanged.

## For OM9's core (not add-on code)

Committing one new object takes about 50 ms in a 5,000-object document and 270 ms in a 20,000-object document; Undo takes 170–250 ms. Built-in commands take the same time. Callgrind on one commit in the 20,000-object document (inclusive shares):

| Share | Where |
|---|---|
| 77 % | `Document::storage_usage_internal`, called from `TransactionCoordinator::validate_retained_owners*`: the memory use of the whole document is measured again at every commit |
| 43 % | `Geometry::validate` via `Document::validate_internal` / `validate_entity`: every object is validated again |
| 33 % | `budget::Usage::add` (summing the above) |

The incremental scene update itself is small.

Keeping the document's usage up to date per changed object, and validating only the objects a transaction touches, would make commit and Undo depend on the change rather than on the document size.
