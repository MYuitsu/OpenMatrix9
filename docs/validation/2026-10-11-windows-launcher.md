# Windows automatic install/update and Desktop launcher

The user approved a bundled offline launcher that installs or updates OM9 in the
versioned AppData Mod directory, preserves the previous module outside Mod,
creates an OM9 Desktop icon and opens official FreeCAD 1.1.4 into the workbench.
The persistent shortcut uses cached bootstrap files rather than the download folder.
Running FreeCAD instances are never terminated by the installer.

Windows PowerShell is installation, COM shortcut and process bootstrap only.
The FreeCAD macro uses its required Python GUI API to activate the workbench and
create an initial document. No CAD modeling logic moved from Rust. The accepted
stock native module and worker are unchanged; no FreeCAD or native rebuild ran.

Fresh [installer evidence](launcher-0.0.2/installer.json) covers 20 checks,
including initial install, unchanged payload, backup/update, damaged files,
active-host refusal, failed plugin promotion and failed cache promotion rollback.
Independent review found auxiliary files were checked too late; an entry-point
test reproduced plugin replacement from an incomplete bundle. Complete bootstrap
hash preflight and staged cache promotion now preserve the plugin/cache on failure.

Fresh [native launcher evidence](launcher-0.0.2/native.json) covers 11 checks on
actual FreeCAD 1.1.4: first startup, workbench/module discovery, persistent cached
startup, real shortcut target/arguments/icon, ICO decoding and exact binary hash.
Custom FREECAD_USER_HOME points directly to v1-1; FreeCAD does not append it.

Actual user startup passed and its OneDrive Desktop shortcut was created. Windows
Known Folder Desktop supports cloud reparse directories; only a shortcut is written
there, with no recursive operation. Physical-volume reads matched all 541 plugin
payload files and four cached bootstrap files. The requested OM9 GUI stays open.
The existing development FreeCAD instance was left untouched.

The supplemental setup ZIP has SHA256
`714dbeb66ef621b5a0c57a6e39c6f2db990cbf0989c2ec31d22872f7ba659deb`.
It is added alongside the original v0.0.2 artifacts with its own
`Launcher-SHA256SUMS.txt`; the original tag, source ZIP and six assets remain intact.
Launcher source and updated installation documentation are on repository main.
See [summary](launcher-0.0.2/summary.json) and [stock-host limits](2026-10-10-stock-freecad-1.1.4.md).
