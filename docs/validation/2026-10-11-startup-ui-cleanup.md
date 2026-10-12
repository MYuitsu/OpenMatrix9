# OpenMatrix9 startup UI cleanup — 2026-10-11

User scope: remove the redundant Import/Export toolbar row and close the empty
Tasks dock at startup. Standard text menus and the other host toolbars remain.

The development plugin removes its dedicated 3DM toolbar; Qt owns dock visibility
and restoration. The installed FreeCAD 1.1.4 release keeps its existing native
plugin and uses its mandatory Python startup macro to hide the 3DM toolbar and
idle Tasks dock after activation/document creation, including queued startup
layout changes. This is a native Qt/bootstrap exception to Rust-first: no portable
model logic, geometry or command state is added to Python or C++.

## Verified installed-host slice

- Official FreeCAD 1.1.4, existing packaged plugin: six checks passed, actual host
  exit 0, with a fresh isolated profile. Startup creates a document, hides 3DM and
  Tasks, and keeps the standard menu and Import3dm/Export3dm actions accessible.
- `build/build-om9-plugin.ps1`: plugin-only compilation/linking succeeded against
  the shared SDK at `build/relWithDebInfo`, using the existing plugin cache.
- Public source audit: 2,423 files, 511 icon bindings, 510 SVG files; zero errors.
- The tested startup macro was installed in the user's existing launcher, with
  original macro/manifest backups and an updated launcher macro checksum.
  Installed source SHA256: `0138b551b601f12e66c4f4ee52f420b65deb0a7f4d6214228d6cf25b47d4afb4`.

Evidence: [stock runtime report](2026-10-11-startup-ui-cleanup-stock.json).
Runtime screenshot: `H:/FreeCAD-src/build/shared-host-tests/stock-startup-cleanup-fresh/stock-startup-clean.png`.
Installed macro: `C:/Users/nguye/AppData/Roaming/FreeCAD/v1-1/OM9-launcher/Start-OM9.FCMacro`.
Backup: `H:/FreeCAD-src/build/om9-startup-deployment-backup-20261011`.

The current user session stays open; the installed macro applies on its next
OpenMatrix9 shortcut launch. No host build/runtime copy or public release upload
was performed. Full interface/menu feature contracts remain partially implemented.
The existing visible-but-disabled command catalog is unchanged.

The user explicitly approved saving these defaults on 2026-10-11: "ghi vào quy tắt".
The approved startup rules are recorded in
[openmatrix9-workspace-contract](../../skills/openmatrix9-workspace-contract/SKILL.md),
with the UI skill routing to that contract. Repository, packaged-source and
installed skill copies carry the same approved rule. This authorization does not
change the runtime validation scope or the lifecycle limits below.

## Additional checks and scope limit

The installed-host probe was repeated with the 3DM toolbar explicitly shown and
Tasks moved to the right and shown before startup. All six assertions passed;
actual host exit 0. Across both isolated startup cases, 12/12 assertions passed.

The development plugin was linked into an isolated plugin output using the same
existing cache and shared SDK, then the previous cache output selection was
restored. No host/runtime was copied. Its broader native probe passed 12/15
assertions: startup cleanup, menu availability, active Task usability, document
geometry and Undo state passed. Three assertions about idle Tasks and prior
hidden state after repeated workbench switches failed. That development-host
cycle behavior is not certified by this startup slice and is not deployed into
the existing official FreeCAD 1.1.4 native plugin.

Reports: [explicitly visible startup fixture](2026-10-11-startup-ui-cleanup-stock-visible-before.json),
[development lifecycle limits](2026-10-11-startup-ui-cleanup-development-lifecycle.json).
