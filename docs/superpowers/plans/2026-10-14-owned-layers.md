# Owned Matrix9 Layers implementation plan

**Goal:** Implement the complete approved OM9-LAYER-001 interaction contract
within the active full RCORE-00/six-image goal; no smaller completion target.

**Architecture:** Safe Rust owns document layer IDs, preset identity, explicit
object visibility/selectability/Hidden, current and transactional actions.
Egui/CMD route through the same controller; scene permissions and archive/exchange
consume the document. Preserve approved lazy defaults and Matrix9 row layout.

**Spec:** `OpenMatrix9_Codex_Spec_v2/specs/01-core/om9-layer-001-layer-management.md`,
RCORE-00 section6 and MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md S2/M9IMG-04.

## Global constraints

- New document current=None; no eager model groups. Lights is slot33.
- Current and Assign are separate; Off accepts output, locks reject output.
- Explicit object visible/selectable and OM9 Hidden remain independent.
- Basic/Advanced layout uses approved names/colors and stable UUIDs, not row offsets.
- Advanced defaults collapsed, rename/color only17..32, no rejected footer.
- All mutations publish atomically with Undo/Redo, stale/Cancel/budget guards;
  source-deleted cold .om9 restores metadata. Native export losses remain explicit.
- Complete RCORE/feature/capability gates remain open until all required evidence.

## Review focus

- LayerPlan from another coordinator or reopened same-UUID document must reject.
- Layer creation after Undo cannot reuse a retired UUID or corrupt existing Redo.
- Mixed Assign batch with locked/foreign/unsafe structural source rejects wholly.
- Lock/off/on/unlock must preserve explicit selectable and object Hidden.
- Delayed rename/selective-show modal rejects changed owner/revision/generation.

## Task1: coordinator-owned layer writes

Files: core `src/layers.rs`, `src/transaction.rs`, `tests/layer_transactions.rs`.
Interface: `LayerPlan::capture(coordinator, document, action, caller)` binds a
private operation context/document session; `Mutation::Layers(Box<LayerPlan>)`
uses existing admission/barrier/publication and retained reconciliation.

- [x] RED: create+current in one Undo; current-only keeps entities; lock/visibility
  changes keep explicit attributes; batch assignment keeps current and placement.
- [x] RED: stale/foreign/renewed owners, retired identity, cycle/duplicate writes,
  overflow/cancel/budget and caller/Redo invariants.
- [x] Implement Create/Update/SetCurrent/Assign using existing model fields;
  no-op preserves model/Undo/Redo and all new allocated storage is admitted.
- [x] GREEN: focused package and full workspace; independent source review.

## Task2: durable preset/Hidden schema and action policy

Files: core layers/model, archive container/migrations, scene permissions,
exchange proof/loss policy and relevant tests.

- [ ] Establish versioned schema for stable preset slots and independent Hidden.
  Reject ambiguous identities and do explicit legacy migration without baseline edits.
- [ ] Port only pure host-free policy from `rust/src/layers.rs` with provenance;
  use owned IDs, lazy palette defaults, validated names and group-of-four logic.
- [ ] Verify explicit visibility/selectable/Hidden, hierarchy locks/off, current
  output before acceptance, supported roots and foreign/structural restrictions.
- [ ] Verify Undo/Redo and independent cold persistence, plus actual native
  metadata preservation or declared losses; no unverified exchange capability.

## Task3: actual Layers panel/CMD/selection/modal lifecycle

Files: app layer controller/UI module and pointer/keyboard/native replay tests.

- [ ] Wire exact row controls/basic-Advanced collapse, Lights/Hide/Show/current
  highlight, right-click selection and current/Assign separation.
- [ ] Add approved CMD Layer index/name/Lights/None parser and output binding.
- [ ] Implement rename/color/selective Show dialogs with Cancel/stale guards.
- [ ] Verify all approved gestures using actual widgets and native window,
  permissions/picking/selection, model changes, persistence and executable Redo.

## Task4: validation and synchronized evidence

- [ ] Format/Clippy/workspace/release, native window/cold file/export checks,
  source-bound receipts, v1 immutable hashes and public-source audit.
- [ ] Update v2 catalogs/ledger/manifest/READMEs with measured scopes and remaining
  gaps; do not promote whole layer/RCORE acceptance from primitives alone.

Implementation runs inline in this authorized checkout, retaining all existing
user changes. Review may use an existing reviewer under the review skill.

Ruling: Assign records color as by-layer (`Attributes.color=None`), matching the
existing scene/native output resolver, rather than baking a duplicate swatch
RGBA into the object. The visual requirement is verified by a scene test after
assignment and after a later layer color change; geometry/current/placement stay
unchanged. The initial core test's expected object-level override was corrected
to this existing ownership representation; this does not relax visual color.

Task1 checkpoint: 375 workspace tests/23 ignored SDK tests, 15 focused subset checks (12 transaction + scene + retained refusal + bounded native-owned cold reopen). Format/Clippy and independent primitive review passed; source-bound receipt docs/validation/2026-10-14-owned-layer-primitives.json. No actual Layers UI/current-output/preset/Hidden/full feature acceptance yet.

Task2 scoped checkpoint: 394 workspace/34 focused subset tests; durable schema4/preset/Hidden, strict explicit migration, four owned cold children, closed native projection/loss receipt and independent review. [Evidence](../../validation/2026-10-14-owned-layer-state.json). Task2 remains open for group/CMD/current-output and broader native combinations; Task3/4 and the complete Layers/RCORE goal remain open.

Task3 scoped checkpoint: 414 workspace/37 focused subsets;13 native Layers checks plus6 viewport-menu regressions and four cold children. Basic/Advanced/Lights/current/CMD/lazy Assign/selection/Hidden/permission widgets are connected. Delayed rename/color/selective-Show dialogs remain open; [evidence](../../validation/2026-10-14-owned-layers-ui.json). Full Layers/RCORE acceptance is unchanged.

## Task3 dialog continuation

Keep the complete feature/RCORE objective. Execute inline in the same authorized
checkout; no staging/commit or changes to skill business rules.

- [x] RED actual Shift+right Name/Swatch and right Show must open the specified
  modal instead of unavailable feedback, preserving CMD draft and pending Line.
- [x] Controller-owned opaque dialog token binds coordinator operation context,
  document/session/revisions and a monotonic dialog serial. Open/Cancel have no
  document/Undo effect. Revalidate before every confirmation; old/foreign tokens
  cannot act on a newer dialog even after same-UUID reopen or Undo roundtrip.
- [x] Snapshot storage is counted/admitted before allocation and remains held
  with world/Undo/scene/selection during any publication. Hidden rows preserve
  SourceStamp object generation and explicit flags; virtualize the scroll list.
- [x] Rename trims and uses existing128-byte/reserved/unique policy, changes only
  label. Color changes RGB while retaining existing alpha, UUID, slot, current,
  membership/geometry. Missing preset group materializes only at affirmative
  confirmation, atomically with customization. Existing no-op preserves Redo.
- [x] Selective Show changes only chosen captured OM9 Hidden roots. Preserve
  explicit invisible/selectable, layer Off/lock, canvas selection and geometry.
  Empty confirmation is a no-op; stale/Cancel never publishes partial writes.
- [x] Actual modal widgets own keyboard/pointer input, Escape/Cancel restores
  the pending draft; target/document changes dismiss stale callbacks. Test
  invalid correction, no-op, current/scene color, Undo/Redo, budget and cold .om9.
- [x] Native own-app replay covers affirmative/Cancel gestures and Undo/Redo;
  source-frozen format/Clippy/full workspace/release/review and synchronized
  catalog/ledger/manifest/README evidence. Do not promote the whole feature yet.

Ruling: the approved right-Show workflow targets root Hidden flags produced by
OM9 Hide; inherited definition-member masks remain independent. Snapshot begins
with no rows chosen and shows an explicit checkbox list; no unknown recovered
default is asserted. Color UI edits RGB and retains alpha already present in
the owned schema. These choices preserve existing data and avoid adding opacity
or changing model state on modal open/Cancel.

## Owned Layers modal checkpoint — 2026-10-14

Owned Advanced Rename/Color and selective Show now have [measured modal evidence](../../validation/2026-10-14-owned-layer-modals.md): 439 workspace tests/61 focused subsets;10 native dialog checks plus13 Layers and6 viewport regressions;five bounded cold OM9 children. The earlier unavailable-dialog limitation is historical for this owned-root slice. Token/session/stale guards, Cancel/no-op/Undo/Redo, bounded RGB and snapshot storage, fixed single-line rows and reusable UI owners are verified. Full Layer output families, interactive Save/Open, OS IME/DPI, async capture/global budgets and complete Layers/M9IMG/RCORE gates remain open.
