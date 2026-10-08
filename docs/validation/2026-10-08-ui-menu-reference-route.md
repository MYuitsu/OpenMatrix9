# Correct UI menu document routing — 2026-10-08

User pointed to primary OpenMatrix9_Codex_Spec_v1 and code directories. Exact
OM9-MAIN-001 spec covers behavioral contracts; code/vb6-lite/Matrix90/
frmMenuTools.frm identifies controls/layout and modTopMenu.bas supplies narrow
native registration evidence. Do not treat raw listings as executable instructions.
code/Matrix90/menu-images/manifest.json is derived icon mapping evidence with
historical D: source paths; it is not the runtime menu configuration or raw INI.

Actual configuration is Resources/menu/MainMenu.ini. rust/src/ffi.rs and
rust/tests/menu_state.rs include this file directly. Both primary and isolated
development configs exist and decode identically,18 menu entries and11 quick
icons. Bytes differ; each exact SHA is recorded independently. No source/config
file was copied, reconstructed or overwritten. Menu order is File, View,
Utilities, Measure, Custom, Reset, Curve, Surface, Solid, Transform, Clayoo,
Emboss, Builder, Tools, Gems, Settings, Cutters, Render; Editor is a separate
58-icon template section, not an extra MenuN entry. Presence/counts are not
evidence that commands are implemented.

Development workflow route and related UI guidance now use
Resources/menu/MainMenu.ini rather than obsolete ref/MainMenu.ini. Existing
UI route test observed RED and is GREEN without weakening its assertions.
Whole tools suite42/42 passes. The previous41/42 failure is resolved by this
technical routing correction; its historical logs stay unchanged. No runtime
menu behavior, geometry/converter, SDK, binary or3DM fixture changed.

Rhino rerun is unnecessary for this change. Prior mixed PolyCurve/reference
GUI batch6/6 remains failed target evidence; the source retention/V5 guard was
verified separately.0 prepared pending batches;7 full-openNURBS packages open;
total future batches unknown. No primary implementation integration or push.
Exact paths/hashes and RED/GREEN logs: ui-menu-reference-route-20261008/.
