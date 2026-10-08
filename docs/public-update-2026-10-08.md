# Public source update — 2026-10-08

This branch adds the verified development source for 3DM preservation exchange,
reference/version checks, native field editing, CAD Wireframe and faster import.
It preserves the public authored icon set, branding and donation instructions.
It also includes Curve/Spline, Rebuild, Edit, Surface and Solid implementation,
native command integration, Rust tests, FreeCAD macros and authored feature guides
from the owner's current checkout. The original working trees remain intact.
Full openNURBS exchange remains incomplete:7 packages remain open;the total
number of future test batches is not yet enumerated.

The prior development runtime passed12 targeted FreeCAD reports/151 checks and
7 native suites for this performance slice. Paired measurements of a privately
supplied ring gave median import25.85→13.71 seconds and first display29.23→16.82
seconds. These measurements are file-specific. No Rhino5 renderer equivalence
or completion of every type/property is inferred from them.

Private source archives,decoded geometry dumps,compiled libraries and original
vendor references remain outside this source tree. Authored validation reports
describe their historical evidence;links to raw reports may require the owner's
local validation archive. Absence of those files is not a successful test.

Build using docs/build-windows.md and cmake/StandaloneSDK.cmake. The native
OM9ThreeDmImportWorker executable is required beside FreeCAD.exe and is built
with OpenMatrix9Gui. Preservation uses up to4 isolated processes for sources
of at most16 MiB with at least8 objects per group. Small/large sources and
Geometry only are serial. OM9_3DM_WORKERS=1 selects the serial oracle.

Configure synthetic native tests with tests/native/CMakeLists.txt. Optional
OM9_RING_FIXTURE and OM9_UNMESHED_FIXTURE accept caller-owned ring files.
Tests depending on absent private/frozen Rhino output are explicitly disabled.
Legacy-reader tests require independently configured upstream SDK2013 tools.
FreeCAD macros requiring local fixtures or installed Rhino5 should be run only
after supplying their inputs;they are not an automatic public acceptance suite.

The former PropertyFileIncluded core-copy dependency is superseded by plugin-owned
chunked payload storage. FreeCAD's source changes have been reverted to the original
baseline. A matching SDK/ABI build is still required; an arbitrary installed
FreeCAD is not certified. See the stock-core storage validation report for current
copy, Undo/Redo, migration and FCStd evidence.

Pre-push checks on this publication source: Rust75 passed; Python29 passed and
1 optional integration test skipped; native module/worker build and link passed.
Actual FreeCAD serial/parallel ring comparison9 checks and final queued Qt
Import/Export8 checks passed with process exit0. Publication review findings
were addressed. The source audit retains510 authored SVG bindings with0 errors.
[Check summary](public-update-checks-2026-10-08.json).

The combined publication also passed107 Rust tests and10 actual FreeCAD suites
(3498 checks), including Curve26, Spline/Rebuild40, Surface96, Solid69, Edit115,
Wireframe25, 3DM menus18, native queued Import/Export8, included-file copy8 and
the authored icon/F6 suite3093. The latter first caught an old T-Splines display
label; restoring SubD and rerunning the suite passed. Public feature guides,
skills and build instructions are included. Strict Rust formatting/Clippy remain
nonpassing and are recorded, without suppressions. Those combined checks used the
earlier core build; the subsequent plugin storage validation uses a fresh build
from the original FreeCAD baseline.
[Combined source evidence](all-code-checks-2026-10-08.json).

The subsequent plugin storage update passed12 FreeCAD suites/382 checks on
freshly rebuilt baseline core, 107 Rust tests, 37 Python tests (one optional
integration skipped) and three native Merge/Hatch suites. Both caller-owned
regression rings and native legacy FCStd exchange are covered. Source audit
reports510 SVG bindings and zero errors. No new Rhino5 application run is claimed.
[Current storage evidence](plugin-storage-checks-2026-10-08.json).
