# G00-04 — 3DM / openNURBS interoperability

Trạng thái: **IN_PROGRESS** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 0/3 · L4 có bằng chứng nghiệm thu: 9/12 · L4 có artefact: 5/12

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-04-01` 3DM capability gap and dependency candidate | TODO | 0 | — |
| `OM9-M00-L3-04-02` NURBS / trimmed BRep / units proof | WIP | 2 | — |
| `OM9-M00-L3-04-03` Import security and transparent limitations | WIP | 3 | — |

## Sự thật đã kiểm

- QymCAD's Format enum has ten variants and no 3DM (QymCAD/crates/qymcad-io/src/formats.rs:21-33)
- the openNURBS pin eb92af3… writes a version-5 3DM fixture (9 528 B: point, line, rational arc, mesh, cloud) and an independent read returns the identical JSON, including rational weights, domain and mesh topology
- a 512-byte garbage archive is rejected with exit 1 and a message, no crash
- the pin is the only available 3DM decoder: OCCT has no 3DM reader and QymCAD lists none

## Còn chặn

- L4-04-02-03 NURBS/trim 3DM -> OCCT TopoDS conversion is not implemented
- L4-04-02-04 block/nested-instance transforms are not tested
- L4-04-02-05 inch->mm, tolerance and UV trims are not tested

## Việc tiếp theo

- implement the NURBS/trim conversion in M03, then the instance and unit tests
