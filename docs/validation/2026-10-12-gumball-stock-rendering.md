# Stock FreeCAD native rendering regression — 2026-10-12

The actual [failed Action](https://github.com/MYuitsu/OpenMatrix9/actions/runs/38184943957)
compiled the stock plugin and passed the first seven native Gumball cases, then
the core capture terminated with Windows heap-corruption exit `0xc0000374`.
The same failure was reproduced locally with the identical official host binary.

Qt loaded bundled `opengl32sw.dll`, while native Coin/FreeCAD imported system
`opengl32.dll`. With the viewport context current, the former returned Mesa 3.0
and the latter returned no GL version. The CI extraction now supplies a
byte-identical `opengl32.dll` alias of that bundled renderer, and Qt selects that
same absolute DLL path. Deployment refuses conflicting aliases, reparse points
and hosts outside the explicit disposable extraction root. The installed user
host is unchanged; local confirmation used host links and one private Mesa alias.

A hidden Win32 launch also left the owned Qt test window unexposed. The native
runner now exposes it before fixtures take handle snapshots. Stock captures use
FreeCAD's supported `SavePicture=FramebufferObject` in the current viewer context,
at the requested image dimensions. The development `renderToImage` branch is
unchanged. The compact-handle pixel threshold and every gesture/geometry oracle
are unchanged. The existing core macro passed all 68 checks, then the complete
stock cohort passed 30 native processes and 955 checks, including all DPI variants,
Scale, Object Alignment, transactions and FCStd reopening.

The full stock cohort also reproduced a perspective Ellipsoid numeric Scale
cancellation. FreeCAD's automatic near/far clipping changes the view-volume
height at the near plane without changing camera pose or field of view. Gumball
now observes the native perspective `heightAngle`, camera type, pose and focal
distance; orthographic height is still observed. The regression explicitly
changes clipping during the real editor interaction, then verifies real
perspective zoom and rotation still cancel without changing model parameters or
Undo count. It fails against the previous plugin and passes all 22 parameter
checks against the incrementally rebuilt plugin. The changed source also passes
the development SDK syntax check; no development-host DLL rebuild is claimed.

Stock `Base::Type::getName()` returns `const char*`, while the development SDK
returns `std::string_view`. Four exact native Binder property/owner comparisons
were therefore comparing pointers on stock and rejecting the supported native
input. Explicit `std::string_view` comparisons preserve the same whitelist and
every protection/identity guard on both SDKs. The full associative regression
fails before this repair and passes 127 checks afterwards, including preserved
editor-only ReadOnly flags, immutable/ordinary read-only rejection, whole and
Face/Edge incoming dependencies, geometry, materials and Undo/Redo. Both changed
native sources pass the development SDK syntax checks; only the stock plugin
target was rebuilt for the latest native cohort.

The profile/deployment regression fails against the previous helper and passes
17 checks against this helper, using isolated fake hosts. Public release tooling
passed 4 package, 3 version and 11 source-audit regression tests. The version
counters remain unchanged by these repairs within the same completion task.

This is a local stock-runtime evidence checkpoint. Consult the exact commit's
[GitHub Actions result](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml)
for the final CI status and packaged artifacts.

```json
{
  "local_stock_native_pass": true,
  "processes": 30,
  "checks": 955,
  "suite_sha256": "800c4c9730f85c20582eee1b352cfb4701520e74d03c1acb2e78b7dff0b9ee7a",
  "host_sha256": "7b38c5d5aa2c74f830539c1c5c44cbf6e3e8e157292c718f9ee24ac6d3e12cf6",
  "plugin_sha256": "71f8431d71f6abfb36c45e9fd2a128f1d8f232368e2e5c1faf5dd3a6ff58b987",
  "renderer_sha256": "38870631e23fe7f7c1f5a1981d4ffa70bbd147caff31d4c1c4f1ef8cf5ff4860",
  "actual_failed_action": "https://github.com/MYuitsu/OpenMatrix9/actions/runs/38184943957",
  "actual_stock_action_authority": "Exact-commit GitHub Actions result and artifacts; this report records local evidence only.",
  "clip_plane_scale_regression": {
    "red_reproduced": true,
    "green_checks": 22,
    "red_report_sha256": "e5707bf4baacb8e98624978f08cf86856a3da73f23660448a632720155e78d8b",
    "green_report_sha256": "7f00aa0a887f576e952376be8cc05de2e73f701c57381d827af7581303573931",
    "dev_syntax_sha256": "f6cf06ccac5b7ca19292074062be33159c007ae508e3271fef46c21902f31318"
  },
  "native_support_regression": {
    "red_reproduced": true,
    "green_checks": 127,
    "red_report_sha256": "ec0182b3503b3125506d8da3f672b0124091fa16f561be0b3b4f38e3379c23cd",
    "green_report_sha256": "d2156d0eb745615a1401c416887f5b5a8211d50c5aa9ef5e902716cbc1e4d13f",
    "dev_syntax_sha256": "c7430701b52a29cbacc68cc93752d7e97d3f1b9fc7b324f6b6fb44d1bea527a5"
  },
  "source_sha256": {
    "Gui/GumballOverlay.cpp": "80122e097feaa18977dddeeda59228b69cb7d8ab18a13904febc0525f17a4a3a",
    "Gui/GumballAssociativeScale.cpp": "d7e11f4dde465688167c0a0f1bc8192b01cead1bceeaa244de35499b4d6ed0cf",
    "tests/gumball_parametric_smoke.FCMacro": "5b9e6324f68724c76bc6cebba5ab4a455c91fbe73853a1507c4e4a94f54e4d6f",
    "tools/ci/gui-profile.ps1": "3d138ddafa0b51e195fecea0ef39a7fdadc302fc4ac723d034c4ada32950c459",
    "tools/ci/build-windows.ps1": "fed326d3b9adb9defb553280706bfaa0d2b08bdee0a6771cede9630abb445fda",
    "tools/ci/run-native-case.ps1": "2a2cba60d19786eb2d9944ed9a73ebe89b378a57b60029288331bf1b6a0557cf",
    "tests/gumball_capture.py": "ff871421649fca60f51877a7cd1f304c0582599abeebc3076983d3240c08039c",
    "tools/tests/ci-gui-profile-tests.ps1": "7d36ba56b856b079e7ed368af67867c88060284898156ac50d8fbbb268ce09d9"
  }
}
```
