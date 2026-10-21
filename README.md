Development main integration (2026-10-12): selected local native SDK proof, 1218 Rust tests and 1066 native checks. [Evidence and host requirements](docs/validation/2026-10-12-main-command-integration.md).

# OpenMatrix9

Windows standalone release (2026-10-21): [v0.3.6-202610211407](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.3.6-202610211407) is published as a pre-release. Both Windows builds and tagged publication passed; the downloaded ZIP checksum and all 432 payload hashes match the tagged commit. The build uses pinned OCCT/openNURBS, MSVC x64, `--locked` and `-j 2`. [Build instructions](tools/ci/README.md) · [Verification](docs/validation/2026-10-21-standalone-release-published.json). CI provenance is build-only; UI/geometry/performance gates remain separate.

Viewport navigation runtime (2026-10-21): direct drag, analytic wheel zoom, one nonblocking window scheduler, adaptive resolution/MSAA, retained GPU occurrences, per-pass camera snapshots and generation-checked full-quality publication after GPU completion are implemented. Automated regressions pass; presentation, physical DPI/RDP and full performance acceptance remain open. Preview conservatively retains LOD0. [Approved plan](implementation/design/M01/VIEWPORT-NAVIGATION-IMPLEMENTATION-PLAN.md) · [Evidence and limitations](implementation/evidence/M01/2026-10-21-navigation-implementation.md).

Navigation forensics review (historical analysis checkpoint, 2026-10-21): `nav-pointer-125.jsonl` was the latest navigation capture in that inspected checkout; 326 camera records were reanalyzed. Per-pane timing defaults, incomplete frame telemetry and wheel interpolation need correction before accepting the proposed navigation refactor. [Review and proposed architecture](implementation/design/M01/VIEWPORT-NAVIGATION-FORENSICS-REVIEW-2026-10-21.md); [measured log summary](implementation/evidence/M01/2026-10-21-navigation-forensics-review.json). Analysis only; no new runtime or UI acceptance.

Slim sidebar scrollbar (2026-10-20): always-visible 4pt solid bar with a reserved 2pt gap, no hover overlay, and retained wheel/drag offset. Rightmost controls stay inside the reduced content viewport; default dock remains 360pt. 11 Rust tests, native narrow-dock and maximize/restore checks pass. [Evidence](docs/validation/2026-10-20-slim-sidebar-scrollbar.md).

Live icon resize fix (2026-10-20): fixed-column Main Menu and Display/Snaps/Info now follow window size within their existing row capacity. Native Windows maximize/restore 1280x720 -> 2560x1369 -> 1280x720 passed; Main Menu 28->31->28pt, toolbar 23->32->23pt, reference palettes approximately 25->28->25pt. 16 Rust tests pass. Default dock remains 360pt; reference slot counts/order retained. [Evidence](docs/validation/2026-10-20-fullscreen-icon-resize.md).

Icon size audit (2026-10-20): 1280x720, 1366x768, Full HD and QHD at simulated 100/125/150%, default dock 360pt. Actual Main Menu slots at 100% are 28/28/31/31pt (ten-column fit), toolbar 23/23/32/32pt, fixed palettes approximately 28pt. Five Rust tests pass; 22 category pages and 20 toolbar tabs checked per matrix case. Runtime sizing unchanged; physical monitor/DPI acceptance pending. [Measurements](docs/validation/2026-10-20-icon-screen-sizes.md).

Reference sidebar slots (2026-10-20): Display 11 icons (6+5); Snaps 16 command icons plus master/four step controls; Info & Settings 22 icons in two 7+4 rows, with history top right and manipulators bottom right. Fixed rows retain order and gaps across dock widths. 35 targeted Rust tests and native 100/125/150 captures passed (125/150 simulated); unavailable controls remain disabled. [Evidence](implementation/evidence/M01/2026-10-20-fixed-sidebar-palettes.json).

Left dock default (2026-10-20): fresh/factory layout and Reset Menus now use 360 outer logical pixels (previously 300); saved custom widths reload as before. 47 targeted Rust tests, validator self-test and native 1280x720 replay passed; measured outer/inner widths 360/355. [Evidence](implementation/evidence/M01/2026-10-20-left-dock-360.json).

UI font smoothing (2026-10-20): installed Segoe UI/Segoe UI Semibold/Consolas with bundled fallbacks, smooth hinting and one glyph pass instead of synthetic overprint. 28 targeted tests passed; native images reviewed at 100/125/150% (125/150 simulated). Full layout oracle still flags the older 22pt icon expectation against captured 24pt icons. [Evidence](implementation/evidence/M01/2026-10-20-smooth-fonts.json).

Dock text readability (2026-10-20): gold headings, warm cream body text and pale mint secondary labels on green, with clearer disabled labels and one clean layer name per row (category retained in tooltip). 32 focused Rust tests and three native GPU scenarios passed. [Evidence](implementation/evidence/M01/2026-10-20-dock-text-readability.json); formal M01-UI review pending.

Green dock theme (2026-10-20): left/right dock backgrounds and floating menu bodies share the Command dark green (`#01391C`), with green section bars (`#005126`). Group/icon/layer colours retained. 14 focused Rust checks and three native GPU scenarios passed. [Evidence](implementation/evidence/M01/2026-10-20-green-docks.json); formal M01-UI review pending.

Command colours (2026-10-20): user-selected dark green history (`#01391C`), green input (`#005126`) and gold text/caret (`#FDD880`), including selection and option tokens. Eight focused Rust tests and one native GPU replay passed. [Evidence](implementation/evidence/M01/2026-10-20-command-green-gold.json); formal M01-UI review pending.

Menu icon emphasis (2026-10-20): larger glyphs inside the existing slots, brighter card outlines and top highlights; original SVG colours and availability retained. 17 targeted Rust checks and one native GPU replay passed. [Evidence](implementation/evidence/M01/2026-10-20-menu-icon-emphasis.json); formal M01-UI review pending.

Main Menu readability (2026-10-20): larger, brighter category labels with stronger strokes, retaining the compact card height and category colours. 66 targeted Rust checks and one native GPU replay passed; formal M01-UI review remains pending. [Evidence](implementation/evidence/M01/2026-10-20-menu-text-contrast.json).

M02 selection / generic handles (2026-10-19): video-driven picking/Gumball and nested shared/mirrored occurrence paths now include canonical native BRep source and UUID topology mapping. The BRep batch passes 176 Rust tests, nine router tests and native BRep/Gumball replays on GPU 100/150% and painter 100%. Save/Open preserves source, topology IDs and placement; reopened OCCT box volume is 6000 mm³. [Verified scope and limits](implementation/evidence/M02/2026-10-19-brep-selection.md). Planned external M02-UI reviewer acceptance remains pending; full M02 is not declared complete. Rotation angle overdraw was subsequently corrected and checked with 18 targeted Rust tests and native CPU/GPU replay ([rotation fix](implementation/evidence/M02/2026-10-19-rotation-feedback.md)).

**Active direction (2026-10-19):** [M01 CAD Shell, Viewport](<OM9_Milestones/M01 - CAD Shell, Viewport/L1.json>) supplies the foundation contracts; exact features use their `OM9_Milestones` owner. Standalone Rust OM9 owns the document/history/UI/scene, with selected OCCT/openNURBS adapters. Matrix9 milestone references and the approved workspace/layers contracts define UI behavior. Existing FreeCAD support tables/runtime reports below remain historical evidence.

Desktop viewports now use the GPU renderer by default; the CPU scene/shading fallback has been removed. A suitable GPU is required, while OCCT remains optional for the default app. [GPU-only implementation, validation and approved rule](implementation/evidence/M02/2026-10-19-gpu-only.md).

Standalone foundation development (2026-10-14): `crates/` (repository root) owns model/transactions/archive/scene and a native Matrix9 UI slice. The measured checkpoint has 360 workspace Rust tests plus 23 separately executed native integration tests. [Execution evidence and remaining gates](docs/validation/2026-10-13-standalone-foundation-progress.md). Selected retained-current 3DM export verifies original/current records, units, frames, raw spans and dirty conservative bounds in a fresh process before atomic publication with an explicit loss report. [Retained-current receipt](docs/validation/2026-10-14-standalone-retained-current.json). The S6 viewport title menu now has typed labels, scoped Maximize/Wireframe and protected keyboard/dismissal interaction; [native menu evidence](docs/validation/2026-10-14-viewport-title-menu.md). Full Matrix9, native family/remapping, clipboard, archive and performance acceptance remain pending; all full RCORE-00 gates stay open. [Build and launch](docs/standalone-foundation.md); released plugin instructions below retain their legacy scope.


Owned Layers primitive checkpoint: 375 workspace tests pass; 15 focused checks are subsets, including an independent owned-process cold metadata reopen. [Measured scope and pending panel/schema](docs/validation/2026-10-14-owned-layer-primitives.md). Native/UI/exchange receipts above retain their earlier checkpoint.

**Open-source jewelry CAD for FreeCAD.** Rust owns the logic; FreeCAD provides native geometry and UI.

Development Box clarification (2026-10-12): default two opposite base corners then Height, optional four methods; staged wire preview in four views and Top-view height drag. [Rust and native evidence](docs/validation/2026-10-12-box-default-video.md).


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
- [Ledger](docs/openmatrix9-progress.json) · [Native reports](docs/validation/) · [Milestone foundation](<OM9_Milestones/M01 - CAD Shell, Viewport/L1.json>).

<a id="license"></a>
## License

[LGPL-2.1-or-later](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md).
OpenMatrix9 is an independent project using Matrix9 as a functional reference;
it does not claim vendor affiliation, clean-room development or legal clearance.
[Source context](docs/public-source-review.md).

Owned layer state schema4 checkpoint: 394 workspace tests/34 focused subsets; four bounded OM9 cold children; 8 separate SDK export tests/18 jobs. [Measured scope](docs/validation/2026-10-14-owned-layer-state.md). Actual Layers panel/CMD/current-output/modal/selection and full RCORE acceptance remain open. Older receipts below retain historical source checkpoints.

Owned Layers widget checkpoint: 414 workspace tests/37 focused subsets;13 actual native Layers checks and6 viewport-menu regressions, four bounded cold OM9 children. [Measured scope](docs/validation/2026-10-14-owned-layers-ui.md). Advanced dialogs, other command families and full Layers/M9IMG/RCORE gates remain open; earlier SDK/UI receipts retain historical checkpoints.

## Owned Layers modal checkpoint — 2026-10-14

Owned Advanced Rename/Color and selective Show now have [measured modal evidence](docs/validation/2026-10-14-owned-layer-modals.md): 439 workspace tests/61 focused subsets;10 native dialog checks plus13 Layers and6 viewport regressions;five bounded cold OM9 children. The earlier unavailable-dialog limitation is historical for this owned-root slice. Token/session/stale guards, Cancel/no-op/Undo/Redo, bounded RGB and snapshot storage, fixed single-line rows and reusable UI owners are verified. Full Layer output families, interactive Save/Open, OS IME/DPI, async capture/global budgets and complete Layers/M9IMG/RCORE gates remain open.

## Owned File lifecycle service — 2026-10-14

Đã thêm [service vòng đời File](docs/validation/2026-10-14-owned-file-service.md): 458 test workspace;17 ca controller và hai tiến trình OM9 mở lạnh mới. New/Open/Save/Save As có dirty, Save/Discard/Cancel, token từng phản hồi và atomic I/O. Menu/CMD/hộp chọn tệp File chưa bật; .3dm, persistence view/material và nghiệm thu toàn bộ File/RCORE còn tiếp tục. Ba replay native hiện có là hồi quy Layers/viewport, không phải nghiệm thu File UI.

## Owned File UI and native cancellation — 2026-10-14

Đã nối [giao diện File native OM9](docs/validation/2026-10-14-owned-file-ui.md): 475 test workspace,16 ca widget và8 kiểm tra File trong cửa sổ native;3 dialog Windows mở/hủy/đóng sạch. Đã sửa Enter lặp và IME chuyển focus có thể tự xác nhận Save/New. Chọn/lưu/mở thành công bằng OS, xử lý lỗi native, persistence đầy đủ, đóng cửa sổ chưa lưu và `.3dm` còn tiếp tục; toàn goal chưa hoàn tất.

## Selected Win32 File adapter and replay guards — 2026-10-14

Đã thêm [adapter chọn tệp Win32](docs/validation/2026-10-14-owned-file-win32.md): 483 test workspace; HRESULT lỗi được báo rõ, Cancel tách riêng và overwrite do token OM9 xác nhận. Replay Layers chờ focus thật và dừng khi dialog chưa mở;3 dialog Windows hủy/đóng sạch. Chọn/lưu/mở thành công bằng OS, startup từ tệp, budgets, close/async, persistence đầy đủ và `.3dm` vẫn tiếp tục; toàn goal chưa hoàn tất.

## Owned window close — 2026-10-14

Đã nối [luồng đóng cửa sổ khi chưa lưu](docs/validation/2026-10-14-owned-file-close.md) qua Save/Discard/Cancel, kể cả nhánh thu nhỏ cửa sổ. 494 test workspace;2 yêu cầu WM_CLOSE thật và4 dialog Windows hủy sạch trong replay mới. Lưu lỗi hoặc hủy giữ tài liệu/bản nháp/cửa sổ. Goal đầy đủ vẫn đang thực hiện; chọn/lưu/mở thành công qua OS, startup, teardown/budgets/persistence/.3dm và các gate đầy đủ còn mở.

## Native File selection and startup — 2026-10-14

Đã kiểm chứng [chọn/lưu/mở tệp qua Windows thật](docs/validation/2026-10-14-owned-file-selection.md): 498 test workspace,3 đường dẫn được Show/GetResult trả về, SaveAs sang tệp khác chỉ nhận đường dẫn sau Replace và giữ nguyên SHA256 tệp gốc. Tiến trình mới chạy --ui path giữ đúng dữ liệu/đường dẫn/tiêu đề và bốn viewport. Goal đầy đủ tiếp tục: persistence hoàn chỉnh/.3dm, missing-module startup, teardown/budgets/async và các gate đầy đủ còn mở.
