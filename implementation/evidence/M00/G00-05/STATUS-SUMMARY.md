# G00-05 — Associative history & stable topology

Trạng thái: **IN_PROGRESS** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 0/2 · L4 có bằng chứng nghiệm thu: 5/8 · L4 có artefact: 3/8

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-05-01` Naming and reference architecture audit | WIP | 2 | — |
| `OM9-M00-L3-05-02` Regeneration and persistence stress proof | WIP | 3 | — |

## Sự thật đã kiểm

- QymCAD naming.rs documents four deterministic rules (split pieces after the parent face, seams after a neighbour, unnamed by position, edges by the face pair) with a per-document NameMint
- QymCAD refs.rs defines Ref/RefError and drivers.rs DriverRef; OM9 MVP uses explicit ObjectId/EntityId instead of query-based refs
- history_ops and history_replay pass (run during this closeout)

## Còn chặn

- L4-05-01-04 split/merge topology baseline is not recorded
- L4-05-02-02 conflict UI and command rollback need the real UI (M04)
- L4-05-02-03 feature-id persistence and cache are not tested

## Việc tiếp theo

- add the split/merge baseline and the persistence test when M04 starts
