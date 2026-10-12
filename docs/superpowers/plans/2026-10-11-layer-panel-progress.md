# Layer panel execution record — 2026-10-11

[Plan](2026-10-11-layer-panel-completion.md), feature OM9-LAYER-001.

## Rulings

- User explicitly omitted footer combo / C A L V C; no placeholder or invented actions.
- Current layer and assignment are separate. Existing color defaults retained; no claim of commercial palette parity.
- Root output integration includes Edit and Rebuild because their created geometry must follow current-layer semantics. Explode retains source ownership and placement.
- Rust owns independent policy. Native exception: FreeCAD group/property/transaction/selection APIs, Qt dialogs and Unicode native label lookup. Modal identity reuses the existing Rust object-session engine.
- Native Selectable stores explicit permission masked only by lock. Native visibility and OM9 guards suppress hidden picks separately, so normal tree Show does not erase selection permission.
- Rebuild ignores only a validated plain root OM9 owner in dependency checks. Real dependents remain protected.
- Shared plugin cache was being reconfigured by concurrent chats. Build reused its current configuration and copied only plugin outputs to the isolated layer runtime; no host clone/root rebuild and no messages to other chats without authorization.

## Execution evidence

- Rust missing helpers first failed RED; layers + layer_panel then passed 11/11. Rustfmt check and isolated Layers Clippy clean. Whole-crate Clippy issues in unrelated WIP were left untouched.
- Native RED: previous panel controls missing; then object names fixed. Independent review additionally reproduced default Rebuild rejecting layer ownership and Tree Show losing Selectable.
- One repair pass: Rust permission split, validated ownership exemption, corrected Rebuild UI. Repair review found no remaining Important issue.
- Edit fixture distinguishes the visible options dialog from hidden Join dialogs awaiting DeferredDelete; diagnostics preserved.
- Current matching-host GREEN artifacts and final documentation checks are recorded in the implementation validation report. No commit/push requested.

## Scope retained

Art/material, general import hierarchy, all semantic builder families and session-time output-layer binding require separate implementations/evidence. Raw FreeCAD commands outside the OM9 dispatcher are outside the selection-guard claim. Legacy standalone Qt test target was not run. Official FreeCAD1.1.4 launcher uses a different ABI and was not overwritten by the dev plugin.

Business-rule update approved by the user: “có lưu trong skill mới”. Saved as `openmatrix9-layers-contract` in repository and installed Codex skills; source/installed bytes and reference application verified. Native implementation evidence remains unchanged.
