# G00-06 — License / redistribution compliance

Trạng thái: **CLOSED** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 2/2 · L4 có bằng chứng nghiệm thu: 8/8 · L4 có artefact: 5/8

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-06-01` Software license and reuse strategy | DONE | 3 | project owner |
| `OM9-M00-L3-06-02` Assets, trademarks and release notices | DONE | 2 | project owner |

## Sự thật đã kiểm

- QymCAD LICENSE is AGPL-3.0; it stays reference material only (not linked, not shipped)
- OCCT 8.0.1 ships LGPL-2.1 plus the OCCT exception; the project links dynamically and publishes the modified TKG3d through the patch and build script
- cargo metadata over 345 packages shows no GPL/AGPL dependency in the Rust tree; the six LGPL-2.1-or-later entries are the workspace members themselves
- design/M00/G00-06/THIRD-PARTY-NOTICES-M00.md records the OCCT modified component, the openNURBS notice and the QymCAD reference-only status

## Còn chặn

- L4-06-01-04 product license decision needs the owner
- L4-06-02-04 legal readiness signature needs the owner

## Việc tiếp theo

- the owner signs the license decision and legal readiness
