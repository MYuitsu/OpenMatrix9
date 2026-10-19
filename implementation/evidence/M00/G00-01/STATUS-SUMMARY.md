# G00-01 — QymCAD GitHub & source reuse

Trạng thái: **IN_PROGRESS** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 0/3 · L4 có bằng chứng nghiệm thu: 2/12 · L4 có artefact: 12/12

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-01-01` Repository provenance & pinned checkout | TODO | 0 | — |
| `OM9-M00-L3-01-02` Workspace map and dependency audit | TODO | 0 | — |
| `OM9-M00-L3-01-03` REUSE ADAPT ADD architecture decision | TODO | 0 | — |

## Sự thật đã kiểm

- the plan pin and the on-disk QymCAD checkout are both 746a17f127897e921a57118b6caad08968602bf5; the re-pin (3015 files, 19 642 SHA references) is complete
- the M00 execution protocol tools now exist: validate_m00_v013.py passes (7 L2 / 19 L3 / 73 L4, DAG and evidence schema consistent), next_m00_task.py reports the blocked task and exits 2, validate_m00_execution.py validates the 12 existing records and its --self-test rejects 5 kinds of forged evidence (6/6)

## Còn chặn

- two L4 need a human decision (L4-01-01-01 architecture, L4-01-03-03 reuse model)
- the ten dependent L4 stay blocked until those two are signed

## Việc tiếp theo

- the owner signs the two approval L4, or authorises the three missing protocol tools
