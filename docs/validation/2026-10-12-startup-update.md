# Startup-only automatic update installation

Approved behavior: check GitHub only when OM9 opens through its shortcut or
Start-OM9.cmd. No periodic monitoring, scheduled task or background check loop.
Click Update to download, verify and install; save and close a running OM9 when
asked, then it installs and reopens the same FreeCAD host/profile. Later offers
the update at the next start; Cancel preserves the installed version.

PowerShell remains the Windows process/installation bootstrap. Modeling and
portable core logic stay Rust. This does not require rebuilding FreeCAD or the
unchanged native plugin. Timestamp versions use the existing main-branch version
parser/comparator; unknown versions, foreign endpoints, drafts and downgrades
are excluded.

The selected GitHub SetupLauncher archive and same-release SHA256SUMS.txt are
downloaded over HTTPS with bounded connection/read timeouts and approved GitHub
redirect hosts. SHA256, GitHub asset digest when present, selected version/revision,
archive paths/size/link checks, and every payload/bootstrap manifest hash are
verified before installing. Existing transactional plugin/cache promotion and
rollback are reused. No browser download is invoked. A fresh isolated profile
does not wait for an unrelated FreeCAD process. Existing installations retain
the conservative host/module process guard; the real stock host path was checked
against the user's running process without changing or closing it.

[Hosted CI run 38222547986](https://github.com/MYuitsu/OpenMatrix9/actions/runs/38222547986)
passed for source `8c9232461b75f206108f8d9fbcab4252f8490df1`: 27 installer,
29 version/shortcut/updater and 24 auto-update checks; eight package and 11 public
source audit tests also passed. Auto-update tests exercise real Windows Update,
Cancel, process waiting and installation with transport fixtures. A separate live
test passed six checks through public GitHub download/verification/installation
and actual official FreeCAD 1.1.4 reopening. The downloaded hosted-CI ZIP passes
21 further real installer, workbench, icon and Start Menu checks on desktop.

GitHub provenance was verified cryptographically for the exact repository,
launcher-windows.yml workflow, branch, source/signer commit and hosted runner.
The ZIP SHA256 is
`ff0cfe46b9fcd1f14c096dc462789e1f988b2a3869207d893e12f7f491ff7ebb`.
See the [verification receipt](2026-10-12-startup-update.json).

The launcher-only CI packages the bootstrap from public source and reuses the
exact matching public `0.3.4-202610120606` native payload. Its original build-only
validation is preserved; launcher checks do not certify all Gumball/CAD workflows.
The native source and ABI are unchanged by this feature. `installed` means the
atomic install completed and restart was dispatched; start-*.json supplies
application-level startup confirmation.

The full Python tooling suite still has 37 historical failures and five errors
among 113 tests, covering icon export/reference fixtures and missing Rhino/
openNURBS evidence. Every failed/error test is named in the receipt. This is not
an all-project green claim. The new release/launcher gates pass.

Local verification encountered full H: during a download and extraction; it
failed before changing an installed plugin. Only owned download scratch was
removed, preserving reports and user drawings, then tests ran on C:. MSIX Local
AppData aliases required matching the selected user_data and actual loaded/native
hash rather than a literal path prefix. The Known Folder junction fixture passes
outside virtualized Local AppData; the standard CI runner also passes it.

Published release ZIPs/tags remain unchanged. Install r2 once to enable this flow;
future release builds inherit the updater. The verified package was extracted
under Downloads/OpenMatrix9-startup-update-38222547986 for first installation.
No feature catalog completion was changed. At runtime verification the reusable
skill had not been updated; the user subsequently approved recording the contract.
See [skill update verification](2026-10-12-startup-update-skill.md).
