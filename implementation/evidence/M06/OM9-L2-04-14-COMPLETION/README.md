# OM9-L2-04-14 — 01-febe-head-and-bezel-brep-settings

Head Builder (basket, half bezel, four edit levels, seat cut into the stone), Bezel Builder
(Bezel, Under Bezel, Gallery Rail, Boolean finger rail groove), Scallop Bezel cutters, the
profile library with user Bezier sections, the girdle seat frames and the rebuild/validity
report. Automated Rust evidence only — the M06-UI manual review is open.

## Commands

| run | exit | tests | seconds | log |
| --- | --- | --- | --- | --- |
| `cargo test -p om9-core --lib settings::head -j 2` | 0 | 12 | 0.34 | `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log` |
| `cargo test -p om9-core --lib settings::bezel -j 2` | 0 | 10 | 0.21 | `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log` |
| `cargo test -p om9-app --test head_commands -j 2` | 0 | 9 | 1.16 | `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log` |
| `cargo test -p om9-core --lib -j 2` | 0 | 171 | 0.37 | `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log` |
| `cargo check -p om9-app -j 2` | 0 | 0 | 1.85 | `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log` |

## Per L3

| L3 | title | feature | design | fixtures |
| --- | --- | --- | --- | --- |
| OM9-L3-04-14-001 | Head Builder Basket | `BuilderHead` | [api](../../../design/M06/OM9-L3-04-14-001/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-001/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-001/negative.json) |
| OM9-L3-04-14-002 | Head Builder Half Bezel Mode | `BuilderHead` | [api](../../../design/M06/OM9-L3-04-14-002/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-002/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-002/negative.json) |
| OM9-L3-04-14-003 | Head Builder Prong Rail Level Edit | `BuilderHead` | [api](../../../design/M06/OM9-L3-04-14-003/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-003/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-003/negative.json) |
| OM9-L3-04-14-004 | Bezel Builder Full | `BuilderBezel` | [api](../../../design/M06/OM9-L3-04-14-004/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-004/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-004/negative.json) |
| OM9-L3-04-14-005 | Bezel Builder Under Bezel And Gallery | `BuilderBezel` | [api](../../../design/M06/OM9-L3-04-14-005/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-005/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-005/negative.json) |
| OM9-L3-04-14-006 | Scallop Bezel Builder | `BuilderBezelCutter` | [api](../../../design/M06/OM9-L3-04-14-006/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-006/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-006/negative.json) |
| OM9-L3-04-14-007 | Head Bezel Profile Browser | `command options only` | [api](../../../design/M06/OM9-L3-04-14-007/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-007/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-007/negative.json) |
| OM9-L3-04-14-008 | Head Bezel Girdle Seat Attachment | `command options only` | [api](../../../design/M06/OM9-L3-04-14-008/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-008/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-008/negative.json) |
| OM9-L3-04-14-009 | Head Bezel Geometry Validity And Update | `command options only` | [api](../../../design/M06/OM9-L3-04-14-009/api.md) | [positive](../../../fixtures/M06/OM9-L3-04-14-009/positive.json) / [negative](../../../fixtures/M06/OM9-L3-04-14-009/negative.json) |

## Gates

- `M06-UI` manual review: **not signed**; the dashboard keeps the nine L3 at WIP.
- Occlusion/cancel/undo: the app tests commit through the host transaction and Undo restores.
- `Boolean=On` is a section-plane boolean: the finger rail is a ring on the same girdle, so the
  groove is the rail section swept along the outline (exact for the ring rail; a non-ring target
  would need the 3-D CSG owner). The Finger Rail body is committed in the groove it cut.
- Profiles: `Profile=Bezier:x,y;x,y;...` samples a closed section and `ProfileSave=<name>` keeps it
  in the session library, so the built-in library stays read-only; the Bezier control-point
  editing UI still belongs to the UI owner.
- Scallop cutters never enter the settings metal mass; the mass report adds the cutter volume
  and the net metal (an upper bound on the removed metal, not a measured cut).
- The seat frames are published by this owner; Enter now applies the seat cut to the gem body
  through the transaction, so Undo restores both the stone and the head.
