# OpenMatrix9

**Active direction (2026-10-13):** [specs v2](OpenMatrix9_Codex_Spec_v2/README.md) define standalone Rust OM9, selected QymCAD code/routines and capability-scoped OCCT/openNURBS. [Matrix9 UI contract](OpenMatrix9_Codex_Spec_v2/MATRIX9_UI_CONTRACT.md) preserves the currently described interface/workflow. Existing FreeCAD support tables/runtime reports below are legacy evidence, not standalone completion.


**Open-source jewelry CAD for FreeCAD.** Rust owns the logic; FreeCAD provides native geometry and UI.

[Tiếng Việt](README.vi.md) · [Installation](#installation) · [Releases](https://github.com/MYuitsu/OpenMatrix9/releases)

![OpenMatrix9](Resources/branding/introduction.png)

## Contents

- [Donate](#donate)
- [Detailed installation](#installation)
- [Progress](#progress)
- [Release notes](#release-notes)
- [License](#license)

<a id="donate"></a>
## Donate

Support CAD, 3DM and jewelry tools. Contributions are voluntary. Thank you!

- **PayPal:** send to **`nguyenthaiduy277@gmail.com`** through PayPal.
- **MoMo:** recipient **NGUYEN THAI DUY**; scan the QR code below.

<img src="docs/images/donate/momo-qr.png" alt="MoMo — NGUYEN THAI DUY" width="240">

<a id="installation"></a>
## Installation

### 1. Prepare FreeCAD

The current release targets **Windows x64 and official FreeCAD 1.1.4**.
Using the package requires no FreeCAD source checkout, Rust or Visual Studio.

1. Open the [FreeCAD 1.1.4 release](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4),
   download the **Windows x64** installer and follow its installation steps.
2. Start FreeCAD once. Check **Help → About FreeCAD** for version **1.1.4**.
   Its installation folder may be called `FreeCAD 1.1` despite the patch version.
3. Save your projects and close FreeCAD before installing or replacing the OM9 plugin.

### 2. Download SetupLauncher

Open **[Release 0.3.5-202610121520](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.3.5-202610121520)**, expand **Assets** and download **SetupLauncher**:

| File | Download |
|---|---|
| **Launcher — recommended** | [OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.3.5-202610121520/OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip) |
| SHA256 verification | [SHA256SUMS.txt](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.3.5-202610121520/SHA256SUMS.txt) |

**Source code** downloads are for development; install OM9 using the **SetupLauncher** ZIP above. This release is marked **Pre-release**.

### 3. Install with the launcher

1. Right-click the **SetupLauncher** ZIP → **Extract All**.
   Extract the entire bundle before opening the launcher.
2. Keep **`OpenMatrix9`**, **`launcher-manifest.json`** and all **`Start-OM9`**
   files together in the extracted folder.
3. Double-click **`Start-OM9.cmd`**.
4. The launcher installs OM9 in `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, creates
   **OpenMatrix9** shortcuts on **Desktop** and in **Start Menu**, then opens the workbench.
5. Use the **OpenMatrix9** shortcut afterwards. Once installation succeeds, the
   extracted download folder can be removed; the plugin is installed in AppData.

OM9 uses your user directory; Administrator access is unnecessary for the launcher
or FreeCAD. If the launcher reports that the plugin is in use, save and close
FreeCAD and retry. It leaves your running application open.

### 4. Open and check the installation

1. Use the installed **OpenMatrix9** shortcut, or open FreeCAD 1.1.4 and select
   **OpenMatrix9** from the workbench selector.
2. Create a document with **File → New**.
3. Select **Line**, place two points, then try **Undo** and **Redo**.
4. Check the sidebar, **Command** area, viewports and **Layers** panel.
   Disabled menu entries represent functionality still being developed.
5. Save a `.FCStd` project. Finish or cancel a modeling command before changing documents.

To check which plugin was loaded, open **View → Panels → Python console** and enter:

```python
import OpenMatrix9Gui
print(OpenMatrix9Gui.__file__)
```

For this installation, the path should end in `v1-1\Mod\OpenMatrix9\bin\OpenMatrix9Gui.pyd`.

### 5. Update, restore or uninstall

- **Update:** download the next Release's SetupLauncher, extract the complete
  bundle, save and close FreeCAD, then run `Start-OM9.cmd`. The launcher checks
  for updates in the background; downloading and installing remain explicit actions.
- **Previous installations:** the launcher keeps them in
  `%APPDATA%\FreeCAD\v1-1\OM9-backups`. To restore one, close FreeCAD, move the
  current plugin outside `Mod`, then place the chosen backup in `Mod\OpenMatrix9`
  keeping the folder named `OpenMatrix9`.
- **Launcher logs:** `%APPDATA%\FreeCAD\v1-1\OM9-launcher`.
- **Uninstall:** close FreeCAD and move `OpenMatrix9` outside `Mod`.
  Remove OM9 shortcuts if you no longer intend to use the installed launcher.
  Your saved `.FCStd` projects remain intact.

### 6. Verify downloads and troubleshoot

Download `SHA256SUMS.txt` from the same Release. In PowerShell in Downloads:

```powershell
Get-FileHash -LiteralPath '.\OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath '.\SHA256SUMS.txt'
```

Compare `Hash` with the line for that exact ZIP in `SHA256SUMS.txt`.

| Symptom | Check |
|---|---|
| OpenMatrix9 is missing | FreeCAD 1.1.4, `v1-1\Mod`, `InitGui.py`, then restart. |
| `No module named OpenMatrix9Gui` | Rerun `Start-OM9.cmd` from the fully extracted SetupLauncher bundle. |
| `DLL load failed` | The Windows x64 package for FreeCAD 1.1.4; read the **Report view** message. |
| Missing icons or menus | The `Resources` directory and the complete package contents. |
| `3DM import worker is missing` | `bin\OM9ThreeDmImportWorker.exe`; re-extract it from the same ZIP if absent. |
| Launcher does not install | Keep OpenMatrix9, the manifest and Start-OM9 files together; inspect step 5 logs. |
| Update says files are in use | Save and close FreeCAD using OM9, then rerun the launcher. |

Open **View → Panels → Report view** for errors. Include the FreeCAD version,
ZIP filename and panel message when reporting a problem. Match the plugin to
its host; do not replace dependencies with DLLs from another FreeCAD version.

**For developers:** [build OM9 against the shared SDK](docs/build-windows.md), in Vietnamese.

<a id="progress"></a>
## Progress

**Gumball 0.3.5**: Shift-click or window-select multiple objects to use one shared Gumball in all four views. Move, Scale and Rotate use the selection center; one Undo restores the whole batch. Local validation: **848 Rust tests**, **31 native cases / 1,099 checks**. [Evidence](docs/validation/2026-10-12-gumball-multi-selection.md). GitHub Actions builds and packages only.


**In progress:** Core/History — object lifecycle, rollback and resource budgets.

```mermaid
flowchart LR
    A["Core & History<br/>In development"] --> B["CAD & 3DM<br/>Supported scope available"]
    B --> C["Builders & Gems<br/>Incomplete"]
    classDef active fill:#e8f2ff,stroke:#2563eb,color:#17376b
    classDef available fill:#e6f6ef,stroke:#20845c,color:#14543b
    classDef planned fill:#f3f4f6,stroke:#9ca3af,color:#4b5563
    class A active
    class B available
    class C planned
```

**Workspace snapshot: 2026-10-12.** Counts describe validated supported scope,
rather than full features. Identifiers can repeat across groups. Consult the Release for binary scope.

| Group | Done / Total | Progress |
|---|---:|---|
| [Quick tools](docs/releases/2026-10-12-development.md#quick-tools--thao-tác-nhanh) | **3/11** | ▰▰▰▱▱▱▱▱▱▱ 27.3% |
| [File](docs/releases/2026-10-12-development.md#file) | **5/13** | ▰▰▰▰▱▱▱▱▱▱ 38.5% |
| [View](docs/releases/2026-10-12-development.md#view) | **10/15** | ▰▰▰▰▰▰▰▱▱▱ 66.7% |
| [Utilities](docs/releases/2026-10-12-development.md#utilities) | **0/22** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Measure](docs/releases/2026-10-12-development.md#measure) | **2/17** | ▰▱▱▱▱▱▱▱▱▱ 11.8% |
| [Curve](docs/releases/2026-10-12-development.md#curve) | **7/58** | ▰▱▱▱▱▱▱▱▱▱ 12.1% |
| [Surface](docs/releases/2026-10-12-development.md#surface) | **3/40** | ▰▱▱▱▱▱▱▱▱▱ 7.5% |
| [Solid](docs/releases/2026-10-12-development.md#solid) | **6/30** | ▰▰▱▱▱▱▱▱▱▱ 20.0% |
| [Transform](docs/releases/2026-10-12-development.md#transform) | **0/41** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Clayoo / SubD](docs/releases/2026-10-12-development.md#clayoo--subd) | **0/58** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Emboss](docs/releases/2026-10-12-development.md#emboss) | **0/7** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Builders](docs/releases/2026-10-12-development.md#builder) | **0/16** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Jewelry tools](docs/releases/2026-10-12-development.md#tools) | **1/31** | ▱▱▱▱▱▱▱▱▱▱ 3.2% |
| [Gems](docs/releases/2026-10-12-development.md#gems) | **0/35** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Settings](docs/releases/2026-10-12-development.md#settings--ổ-đá-và-chấu) | **0/12** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Cutters](docs/releases/2026-10-12-development.md#cutters) | **0/11** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Render](docs/releases/2026-10-12-development.md#render) | **0/23** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Core / project / selection](docs/releases/2026-10-12-development.md#core--project--selection) | **5/11** | ▰▰▰▰▰▱▱▱▱▱ 45.5% |
| [Additional display](docs/releases/2026-10-12-development.md#view-và-display-bổ-sung) | **10/22** | ▰▰▰▰▰▱▱▱▱▱ 45.5% |
| [Snaps / constraints](docs/releases/2026-10-12-development.md#snap--ràng-buộc) | **4/17** | ▰▰▱▱▱▱▱▱▱▱ 23.5% |
| [Properties / history / layers](docs/releases/2026-10-12-development.md#properties--history--settings--layers) | **7/26** | ▰▰▰▱▱▱▱▱▱▱ 26.9% |
| [3DM / keyboard](docs/releases/2026-10-12-development.md#3dm-chuyên-biệt--keyboard) | **1/3** | ▰▰▰▱▱▱▱▱▱▱ 33.3% |

<a id="release-notes"></a>
## Release notes

- [Release 0.3.5-202610121520](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.3.5-202610121520): changes, downloads and release-specific limitations.
- [Development checkpoints](docs/releases/2026-10-12-development.md): command tables, history and evidence moved from the README; includes Vietnamese notes.
- [Ledger](docs/openmatrix9-progress.json) · [Native reports](docs/validation/) · [Spec](OpenMatrix9_Codex_Spec_v2/INDEX.md).

<a id="license"></a>
## License

[LGPL-2.1-or-later](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md).
OpenMatrix9 is an independent project using Matrix9 as a functional reference;
it does not claim vendor affiliation, clean-room development or legal clearance.
[Source context](docs/public-source-review.md).

Owned Layers widget checkpoint: 414 workspace tests/37 focused subsets;13 actual native Layers checks and6 viewport-menu regressions, four bounded cold OM9 children. [Measured scope](docs/validation/2026-10-14-owned-layers-ui.md). Advanced dialogs, other command families and full Layers/M9IMG/RCORE gates remain open; earlier SDK/UI receipts retain historical checkpoints.

## Owned Layers modal checkpoint — 2026-10-14

Owned Advanced Rename/Color and selective Show now have [measured modal evidence](docs/validation/2026-10-14-owned-layer-modals.md): 439 workspace tests/61 focused subsets;10 native dialog checks plus13 Layers and6 viewport regressions;five bounded cold OM9 children. The earlier unavailable-dialog limitation is historical for this owned-root slice. Token/session/stale guards, Cancel/no-op/Undo/Redo, bounded RGB and snapshot storage, fixed single-line rows and reusable UI owners are verified. Full Layer output families, interactive Save/Open, OS IME/DPI, async capture/global budgets and complete Layers/M9IMG/RCORE gates remain open.

## Owned File lifecycle service — 2026-10-14

[Controller/I/O evidence](docs/validation/2026-10-14-owned-file-service.md): 458 workspace tests/80 focused subset;17 File service checks and two new bounded cold OM9 children. New/Open/Save/SaveAs ownership, dirty fingerprints, per-response tokens and atomic archive I/O are implemented as Task1 prerequisites. File menu/CMD/native chooser remain disabled/unaccepted; .3dm and full view/material persistence remain required. Existing native Layers/viewport regressions are separate from File UI evidence. Full feature/M01–M12 gates and catalog statuses remain open.

## Owned File UI and native cancellation — 2026-10-14

[Current scoped evidence](docs/validation/2026-10-14-owned-file-ui.md): 475 workspace tests/96 focused subset;16 File widgets;8 native File checks and3 real Windows picker cancellations, distinct from scripted path delivery. Native `.om9` menu/CMD/F6/Standard routes and owned prompts are connected.10+13+6 Layers/viewport regressions and7 cold children are separate. OS successful selection/save/open, native backend diagnostics/single overwrite UX, UI/native peak ownership, native startup, close/async/full persistence/.3dm remain required. All full feature/RCORE/M9IMG gates and catalog statuses remain open.

## Selected Win32 File adapter and replay guards — 2026-10-14

[Current scoped evidence](docs/validation/2026-10-14-owned-file-win32.md): 483 workspace/104 focused subsets; selected authored Win32 adapter exposes HRESULT errors and disables OS overwrite prompt for the existing owned confirmation.4 audit+3 native+1 UI-error checks;8 native File checks/3 real Windows cancels;10+13+6 regressions/7 cold children. Readiness guards prevent missing-modal events from editing CMD; original missed-click cause remains unknown. OS selection/save/open/overwrite/native startup, full budgets/close/async/persistence/.3dm and all full feature/RCORE/Matrix9 gates remain required.

## Owned window close — 2026-10-14

[Scoped close evidence](docs/validation/2026-10-14-owned-file-close.md): 494 workspace/115 focused subset. Owned Save/Discard/Cancel controls window closing in normal UI and minimized logic; stale consent/save failure/chooser Cancel cannot approve shutdown. Native replay11 checks includes2 root WM_CLOSE requests and4 real picker cancels; original File8 and Layers10+13/viewport6 are separate overlapping regressions;7 cold children exit. Successful OS selection/save/open/overwrite/startup, native Save-success/Discard-button shutdown, complete teardown/budgets/async/persistence/.3dm and full gates remain required.

## Native File selection and startup — 2026-10-14

[Scoped native File evidence](docs/validation/2026-10-14-owned-file-selection.md): 498 workspace/119 focused subset;3 real Windows Show/GetResult selections, alternate SaveAs/owned Replace adopts path only after acceptance with original/seed SHA256 preservation. Native actual --ui path clean startup3checks and cold report match. Success5/close11/File8/Layers10+13/viewport6 are separate overlapping replays;7 Job-owned cold children plus2 bounded CLI inspections. Missing-module native startup, native success/Discard shutdown, complete persistence/.3dm/async/budgets/disposal and all full gates remain open.
