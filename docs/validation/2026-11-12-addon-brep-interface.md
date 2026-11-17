# Add-on interface for trimmed surfaces, B-Reps and subobject picks

Date 2026-11-12. Three stacked branches on the fork. Each is a PR into the one before it:

1. `codex/addon-trimmed-faces`, based on `codex/surfaces`;
2. `codex/addon-brep`;
3. `codex/addon-subpick`.

**Why.** Rhino 5 commands that Matrix 9 runs work on the exact geometry of trimmed surfaces, polysurfaces and solids:
- Untrim;
- surface edges as rails and sections;
- ExtrudeCrv with Solid=Yes;
- edges picked with Ctrl+Shift+click.

Until now add-ons received only curves and untrimmed NURBS surfaces exactly. Everything else came as a display mesh.

## What crosses the interface

| Direction | Before | Now (`brep_data = true` commands) |
|---|---|---|
| Rhino-trimmed surface → add-on | the untrimmed surface | `Geo::Trimmed`: the surface, plus its trim loops run along it as closed polylines (about 256 points per loop) |
| One-face B-Rep → add-on | display mesh | `Geo::Trimmed`: exact NURBS surface and loop edges from the kernel's `face_nurbs`; a face that is its whole surface comes as `Geo::Surface` |
| Polysurface or solid (≤ 256 faces) → add-on | display mesh | `Geo::Brep { faces, solid }`: every face exact; `solid` when it is one closed solid |
| Faces/edges picked with Ctrl+Shift+click → add-on | – | `Selected.sub`: index of each face/edge, plus its exact geometry (face, or the edge's curve from `edge_nurbs`) |
| Add-on → `Geo::Brep { faces, solid }` | – | Faces made by the kernel and sewn (`sew_faces`, 0.01 mm). `solid = true` refuses anything that is not one closed solid. Without the kernel, the display mesh is kept |

Wire compatibility: every new field defaults through serde and is sent only when present. Older add-ons keep working.

For add-on source code:
- the new `Geo` variant is a breaking change for any `match` on `Geo` with no wildcard arm;
- the new `Selected.sub` field is a breaking change for full `Selected { … }` literals. These appear only in tests; they were updated here.

Performance:
- A B-Rep is read once per shape, keyed by its source hash (cache of 32 shapes).
- Solids above 256 faces are never converted.
- The display's face count is checked before any kernel call.

## Kernel changes (C++ adapters)

- `face_nurbs_bridge.cpp` (new, read-only). It returns each face's surface, clamped. Rules:
  - an unbounded surface is cut to the face's UV box;
  - a full-period direction is opened without trimming.

  It also returns the face's loops. Loop rules:
  - edges come in wire order, oriented by the forward face (outer loop counter-clockwise, holes clockwise);
  - every edge ends on its vertices;
  - faces bounded only by their surface's own sides, seam and poles get no loops.

  `om9_occt_edge_nurbs` returns every edge in the display's edge order.
- `creation_bridge.cpp` `om9_occt_trimmed_face` has two changes:
  - loops are chained vertex to vertex in the order given (previously `BRepBuilderAPI_MakeWire` searched by position, which joins a seam point passed twice to the wrong edge);
  - on closed surfaces each p-curve is moved by whole periods so that it continues the previous one.
- `editing.rs` `sew_faces` builds `trimmed_face`s, then calls `sew_all`, then checks for a closed solid.

## Evidence

Windows CI (OCCT 8.0.1, `kernel-tests-windows.yml`):

| Test target | Branch | Result | What it proves |
|---|---|---|---|
| `face_nurbs` | trimmed-faces | 5/5 | box sides whole; drilled plate (hole r 2, winding); cylinder clamped on r 3; sphere and torus on their shapes; cut torus loops chained; malformed text refused |
| `face_nurbs` | subpick | 6/6 | adds: display triangles lie on the face their index names and display segments on their edge's curve (drilled plate, sphere); sphere poles; box edges 4 × 4/6/10 |
| `sew_faces` | brep | 4/4 | six trimmed planes give the box (volume 1000); five give an open shell with 4 naked edges, which is refused as a solid; read→sew round trips of a box (400), a drilled plate (400 − 16π) and a cylinder (45π); extruded circle + caps give the cylinder (12π) |
| `trimmed_face` | brep, subpick | 5/5 | the existing trimmed-face tests pass unchanged after the loop-chaining change |
| `sew_faces`, `extrusion`, `l2_01_10` | subpick | 4/4, 3/3, 74/74 | no regression in the neighbouring face/extrusion/boss tests |

Before CI, a standalone harness on Linux with OCCT 7.6 ran `face_nurbs` → `trimmed_face` → sewing with precise integration. Results:

| Shape | Volume |
|---|---|
| box | 400 |
| drilled plate | 349.735 (= 400 − 16π) |
| cylinder | 141.372 (= 45π) |
| cone | 47.124 (= 15π) |
| torus with a quarter cut away | 592.18 (= ¾ of 2π²·10·4) |

This harness found five bugs, all fixed before CI:
- periodic knots not clamped;
- a torus over two full periods refused by the conversion;
- a single-edge hole oriented wrongly (`BRepTools_WireExplorer::Orientation`);
- loops across seams not closed;
- 1e-7 gaps left by booleans.

Linux (no kernel):
- add-on tests:
  - Loft between the hole edges of two trimmed plates gives the cylinder r 1;
  - Untrim restores the surface, and KeepTrimObjects keeps the loops;
  - ExtrudeCrv Solid caps a ring with its hole: walls on r 2 and r 1, caps at z 0 and z 3;
  - Loft between two picked edges of a polysurface;
- controller tests:
  - Untrim on a Rhino-trimmed plane with a placement removes the trim record and keeps two loops in world position;
  - ExtrudeCrv Solid gives one object (the display prism within 0.1 % of π r² h);
- all add-on crates' tests;
- `cargo check -p om9-app` with `occt-brep` (type check against the kernel API).

## Known limits

- A face whose trim loops pass through a surface pole (for example a sphere drilled across) is refused by `trimmed_face` with an error, never with wrong geometry. Whole spheres and cones go through as whole surfaces and work.
- A solid made of one closed face (a whole sphere or torus sent back with `solid = true`) is refused: sewing one face gives no shell.
- `Geo::Trimmed` does not yet carry the face's `reversed` flag. No current command needs the normal side.
