# Approved launcher contract in the workflow skill

The user explicitly approved saving the startup/update contract on 2026-10-12.
Only the OpenMatrix9 workflow skill's launcher section was added to repository
source and the installed C:/Users/nguye/.codex/skills/openmatrix9-workflow copy.
Existing unrelated edits and installed guidance outside that section were preserved.

The contract specifies one check when opening OM9 through its launcher/icon,
automatic official-package download/verification/installation after Update,
waiting for the user to save/close before replacing an in-use plugin, reopening
the same host/profile, no forced closing, and rollback of plugin and cached
launcher on install failure. Later/Cancel and offline behavior are covered.
It distinguishes install/restart dispatch from confirmed workbench startup and
source/CI evidence from a published release ZIP.

Read-only RED retrieval/application baseline: an isolated agent reading the old
installed skill found no normative direction for query frequency, Update action,
unsaved work, rollback, or installed/startup claims. Those five decisions were
underspecified despite generic evidence guidance.

Read-only GREEN pressure test: a fresh isolated agent used the new section for
all five scenarios. It rejected a legacy 24-hour timer and browser download,
waited for user-controlled save/close despite deadline pressure, preserved files
on hash failure and required rollback on installation failure, and refused to
claim startup or public r2 publication from an installed result/source alone.
The initial cache detail was clarified to include the launcher/cache in AppData.
Implementation choices such as staging cleanup remain outside this business rule.

Both skill folders passed quick_validate.py. The newly added section is identical
in source and installed copies; pre-existing differences outside it are retained.
This update changes reusable guidance, not product code, native binaries, or
release assets. Runtime evidence remains in the
[startup updater receipt](2026-10-12-startup-update.json).
