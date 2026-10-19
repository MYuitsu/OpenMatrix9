# Verification by the current agent (2026-10-19)

I read `implementation/evidence/M02/2026-10-19-selection-video-review.md` and its BRep/occurrence continuations,
then reproduced the native selection replay on the **current** build to verify the recorded behaviour myself
instead of restating it.

## Replay command

```
OM9_GPU_SPHERE=1 OM9_UI_AUDIT_SELECTION=1 OM9_UI_AUDIT_WINDOW=1280x800 \
  L:/om9-target/release/om9.exe --ui-audit <artifact dir>
```

Result: **12 PASS, 0 FAILED** of 12 checks.

| Check | Result |
| --- | --- |
| click selects the curve and exposes a pivot | PASS |
| first drag sample previews +5 mm without document or history writes | PASS |
| release commits +5 mm as exactly one Undo entry | PASS |
| CMD Undo restores original geometry | PASS |
| CMD Redo restores +5 mm | PASS |
| scale square doubles X about current pivot | PASS |
| rotation arc makes 90 degree Z turn | PASS |
| Escape discards preview and retains old Redo | PASS |
| old Redo remains usable after cancelled drag | PASS |
| GPU and painter paths select the polygon mesh | PASS |
| mesh preview uses selected mesh indices after selection changes | PASS |
| mesh release commits one additional transaction | PASS |

## Artifacts and hashes

```
7f81751d44bf9a7c788adc97fe8e68e1277ebbb1c5ae543506a6ffb940e05037  m02-mesh.png
6ae96c4566755250852d3f0e4022458ea551f8aa4b25b293a40a3cc7fb90e7b8  m02-move.png
4b1eaee067764bd06c7b570fc8cea105d4f6ca88ef801ff35cb7c4b5ddc0ced8  m02-rotate.png
d7a4dbcdfdfe5221f5eb2e87834f1a888a2533b53ee681ab40a9fa297546a2d7  m02-scale.png
613f8f81d3a0db56ff4f4e9a6146e94928f397548f464dd8b3cb7f42b4c1c4f2  selection-trace.jsonl
65fbcfcea5a21630f50e54850ee7dce92bf34c07f2444beedb94665641e27ce9  ui-receipt.json
b01bfb301e4b2747933dade38aea63f5c1d67c9dc319575e40fc84236563c6dc  workspace.png
```

The `m02-rotate.png` frame shows the yellow angle sector at the pivot, the Gumball arrows/arcs/boxes and the
rotated curve, which is the behaviour the review attributes to the clip. I inspected that image directly.

## Defect found while verifying

`cargo clippy --workspace --all-targets -- -D warnings` failed with four non-exhaustive `Geometry` matches in
`om9-exchange` after `Geometry::Brep` was added. Fixed: `inspection` validates the `Brep` JSON leaf and labels it
`ON_Brep`; `retained_cold` refuses cold BRep export with a typed `Unsupported` error (instead of inventing a class
UUID) and uses `display.points` for the hull. Workspace clippy with warnings denied now passes.

This note is not a human M02-UI approval and does not sign the review gate.

