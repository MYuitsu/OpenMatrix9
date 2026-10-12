# Execution ledger — plan: docs/superpowers/plans/2026-10-03-matrix9-menu.md

Spec: docs/superpowers/specs/2026-10-03-matrix9-menu-design.md

- Ruling: work on codex/matrix9-menu in the requested nested OpenMatrix9 checkout. The app worktree tool addresses the parent FreeCAD repository; a branch here keeps module changes reviewable and preserves untracked user references. Cost if wrong: checkout isolation is weaker than a separate worktree; stage explicit paths only.
- Pre-flight Task1→3: catalog order must be stable for command indices/CString lifetimes; use one immutable catalog, mutable state separately.
- Pre-flight Task2→4: icons.ini uses relative atlas paths and validated rectangles; missing resources retain visible disabled/fallback entries.
- Pre-flight Task3→4: native host checks command availability and confirms success; Rust history accepts acknowledged executions only.
- Pre-flight Task4→5: runtime proof must load the actual compiled module, not just render a standalone Qt widget.
- Baseline: cargo test failed with seven missing-module/type/helper errors; repairing the baseline is part of the authorized Task1.
- Task1: Rust parser/state implemented. Five tests passed (real18/11 catalog, BOM/CRLF/Unicode, malformed input diagnostics, independent section state, successful-only bounded history). Clippy passed. Task1 does not implement the native ABI/sidebar yet.
- Task2: exporter tests passed4/4; source hashes preserved. PNG atlas CRC/structure and rectangles validated.15 explicit mappings resolved,419 unresolved names retained as fallbacks. Distribution INI trailing spaces normalized; original ref unchanged.
- Task3: complete in e93ec91. Seven Rust tests passed; immutable CString catalog and bounded indices linked through the actual native module. Only verified FreeCAD IDs dispatch; unsupported aliases disabled.
- Task4: native sidebar/build complete. QtTest 6 passed (four cases plus lifecycle hooks). Standalone SDK mode builds module against matching existing FreeCAD headers/import libraries; existing E: cache untouched. Ruling: C++23 required by current FreeCAD headers; parent option patch retained separately. Cost if wrong: SDK mismatch requires rebuilding against the correct SDK, not altering Rust ABI silently.
- Runtime regression: native-menu execution changed Rust history but not visible buttons. Observed failing FreeCAD and Qt tests; history snapshot synchronization fixes both.
- Final independent review: two Important findings. Save result inference used recompute state and FileName; pending-recompute smoke failed before fix, passed after switching to scoped signalFinishSaveDocument observation. Closing MAIN MENU lost Reset; Qt recovery test failed before an always-accessible sidebar context action, then passed. Failed Save As to an existing unwritable file has no dedicated interactive test; scoped completion signal prevents acknowledging absent save completion.
- Visual correction: removed redundant TopIcon text toolbar because sidebar already owns the eleven quick positions; titles now left-aligned at reference height. No jewelry commands activated by appearance alone.
- User steering: use OpenMatrix9_named_icon_crops. Inspected PNG and source manifests; added 178 explicit reviewed bindings, preserving generated-image provenance and original bytes. Rejected Interp Curve→ContinueInterpCurve suffix candidate; bound to InterpolatePoints instead. T-Splines excluded from automatic Clayoo/SubD mappings. Exporter expanded to six tests; totals now 188 resolved/246 missing.
- Task5: native FreeCAD smoke passes 23 checks at scale 1/1.25/1.5/2 with isolated profiles; New/Open/Save/cancel/history/switching/recovery/scroll reachability verified. SDK module build and loaded runtime are evidence; full parent build and live Sky input into FreeCAD remain unverified. Screenshot delivered from actual Qt dock grab; OpenGL viewport capture artifact is outside this menu slice.
- Integration ruling: existing user authorization includes push. Commit explicit implementation paths and push codex/matrix9-menu; keep branch/workspace without merging or staging raw references/user deletions.

## Icon correction 2026-10-04

- User reported too many missing icons. Located installed UserInterface/ButtonIcons.bin and SliderIcons.bin; previous exporter had ignored these original resources. Verified archive structure without running original code. Exact keys cover 433/434 menu names; explicit SolidPtOn alias covers the remaining name.
- Exporter now distributes 434 original menu mappings + 26 auxiliary mappings (460 total), zero missing menu resources. Six Display fallback images replaced using exact named Slider records; ten Snap/five Info controls use original images. Original BMP bytes/hashes/offsets/dimensions preserved; named redraw bindings retained as a fallback.
- Observed failing tests before archive extraction, missing Snap/Display buttons and incorrect partial-archive missing counts; corrections passed. Python 19 passed; QtTest 8 passed; native module built and all 27 runtime checks passed at 1/1.25/1.5/2. Every menu icon was compared against installed source pixels.
- Independent read-only review audited both complete archives and all 460 mappings; no important defects. Ruling: source images prove artwork/name correspondence; they do not prove modeling behavior or every original toolbar index. Keep unsupported commands disabled. Remaining Info text and Layers/Projects presentation remain explicit scope gaps.
- Detailed evidence: docs/validation/2026-10-04-original-icons.md and analysis/matrix90/icon-archives.md. Updated actual native screenshots and runtime reports are distributed with the documentation.

## Modern artwork and sidebar correction 2026-10-04

- User explicitly corrected priority: their OpenMatrix9_named_icon_crops redraws are the product artwork; four supplied Matrix9 images guide compact rows, color and panel structure. This supersedes the original-art distribution ruling above. Keep original references read-only.
- Added 17 reviewed sidebar bindings (195 total; 194 accepted generated images, one original fallback excluded). Removed 461 legacy packaged images. Named redraws win even in optional original-reference mode; normal export uses only redraws and newly authored SVGs. Main coverage187 confirmed/247 category fallback, all434 names render. Generic snap symbols are explicitly fallback, not recovered command artwork.
- Preserved full-color Disabled QIcon pixmaps without enabling unsupported actions. Rebuilt DISPLAY/SNAPS/INFO/LAYERS/PROJECTS control structure; Rust catalog/state/ABI unchanged. MAIN MENU uses three selector rows with white text and colored dots.
- Observed red disabled-color, panel-count and migration tests before fixes. Actual FreeCAD screenshot exposed toolbar wrapping despite offscreen geometry test passing; native no-wrap check observed red, then passed after auxiliary controls changed to23px to allow scrollbar space. No new modeling functionality is claimed.
- Independent review found unmanaged-atlas deletion and generic snap misclassification. Both received observed regression failures then fixes; unlisted files survive export and generic icons retain explicit fallback status.
- Evidence and next scope: docs/validation/2026-10-04-modern-sidebar.md, docs/validation/modern-assets.json and docs/menu-validation.md.

## Workspace RGB +5 artwork 2026-10-04

- Latest explicit user request: recreate workspace menu using the recolored Matrix90 images. This supersedes modern-redraw priority for the current product menu. Reuse the already verified shifted PNGs; do not shift them a second time or modify supplied redraws.
- Added explicit `--shifted-root` export path.492 mappings (491 unique images) use RGB +5 derivatives; all434 MainMenu names are confirmed with zero category fallback.58 auxiliary mappings use ButtonIcons/SliderIcons derivatives;11 chrome keys retain authored symbols.
- Validate source transform metadata, PNG CRC/dimensions/current SHA and original PNG fingerprint, record/path safety. Record sources with relative runtime image paths; installed workspace does not need `ref` or Matrix90. Shifted import outranks both named redraw and optional original modes. Three Snap graphic aliases are explicitly listed, without claiming snapping implementation.
- Observed two importer tests fail before implementation; full Python24 tests then passed. Qt10 tests passed, SDK module built/linked, and real FreeCAD31 checks passed at scale1/1.25/1.5/2. Pixel comparisons cover every menu/quick image, full-color disabled and source hashes. New/Open/Save/cancel/history/lifecycle/restore/scroll behavior remains verified.
- Independent read-only review audited all503 records and492 shifted mappings; no blockers. Source and installed PNG bytes match;214 previously managed unused installed images retired after path checks. Native full-content/panel screenshots refreshed. Evidence: docs/validation/2026-10-04-rgb-workspace.md and rgb-workspace-assets.json.

## Functional workspace 2026-10-04

- User selected workspace selection, deletion, Undo/Redo and camera controls. Added bounded Rust catalog and effect/permission policy, native adapters, compact Qt row and Workspace menu.
- All selects document objects directly rather than context-sensitive Tree SelectAll; None preserves foreign-document selections. Native Delete preserves dependencies/transactions, with foreign/mixed/edit guards. Native file and Undo/Redo task policies remain intact; wrappers do not add transactions.
- Six additional RGB+5 graphics and authored Isometric cube eliminate missing history artwork.510 packaged mappings are byte-verified against source/installation; current audit is workspace-assets.json.
- Independent review findings received native RED regressions, fixes and a final no-findings verdict. Current semantic/native evidence and limits are in docs/validation/2026-10-04-workspace.md.
