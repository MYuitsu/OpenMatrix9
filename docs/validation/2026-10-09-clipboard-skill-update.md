# Approved Phase 1 clipboard/worker skill update

The user explicitly answered “có cập nhật skill” after the proposed reusable text and verified implementation results were presented. This authorizes recording the Phase 1 two-way Rhino 5 ↔ OM9 clipboard behavior and bounded 60% worker default. No further approval was requested.

Updated `skills/openmatrix9-feature-port/SKILL.md` discovery description and conditional 3DM reference link. Added the approved rules to `references/rhino5-3dm-contract.md`: current selected geometry, text shortcuts, one Paste Undo transaction, independent editable objects, `max(1, floor(0.60*N))` available-logical-thread budget bounded by tasks/RAM, CAD/display-mesh classification and metadata-only availability. Actual mesh/cloud Copy still serializes required geometry. The acceptance remains scoped to Phase 1, not full openNURBS.

Both changed files were synchronized to `C:/Users/nguye/.codex/skills/openmatrix9-feature-port`. The installed skill's existing progress-synchronization paragraph was preserved in the source copy; the source's existing 3DM/block contract was preserved in the installed copy. UI metadata is unchanged. Original installed files are retained under `H:/FreeCAD-src/build/clipboard-skill-backup-20261009`. No unrelated skills or product code were changed.

Fresh reference tests:

- RED: independent agent `skill_clipboard_baseline`, restricted to the original source skill and linked references, could not retrieve any of the six clipboard/worker/classification answers. It correctly distinguished the existing CAD-preserving block-import rule from missing clipboard rules.
- GREEN: independent agent `skill_clipboard_forward`, restricted to the updated installed skill and linked references, answered all six scenarios correctly, calculated 14 workers from 24 available threads, and rejected a full-openNURBS inference. The supported-type enumeration remains in the existing checkout support matrix rather than duplicated into the skill.
- Source and installed skill each passed `rtk proxy python -X utf8 C:/Users/nguye/.codex/skills/.system/skill-creator/scripts/quick_validate.py <skill-directory>`.
- `rtk proxy python H:/FreeCAD-src/build/check-clipboard-skill.py` passed source/installed byte equivalence for two changed files and unchanged UI metadata, eight local links, and four checkout evidence/reference paths.
- `rtk proxy python H:/FreeCAD-src/build/verify-final-clipboard.py` passed the existing 14 bound product requirements, 48 native suites and 11 Rhino cases. This was an evidence-integrity check; application suites were not rerun for this documentation-only update.

SHA-256 source/installed matches:

| File | SHA-256 |
|---|---|
| `SKILL.md` | `0acc31f7f48cbb159727dac2c219840c0ba5a8f96a231130eb8e87ed022c5799` |
| `references/rhino5-3dm-contract.md` | `dfdfbb16db58999818304ffc924294eb8505c0e13acced10cae3bcbb85b2b838` |
| `agents/openai.yaml` (unchanged) | `976eb696e16aa59c7db3365747bd22a4fc0793206124c1aba3ae09b0eaa192c4` |

Next product action remains Phase 2 classification and bounded CAD snap/curve editing with an actual application gate. Seven full-exchange packages remain open; the total future batch count is unknown. No commit, push or primary installation was performed.
