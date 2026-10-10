# OM9-CURVE-001 — Polyline

Status: PARTIAL. Original public command: `Polyline`.

Reference: `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-001-polyline.md`. The user selected these specs as sufficient implementation references; no PDF reread is required.

## Spec-derived behavior

Create one straight segment or a chain of straight segments forming an open or closed curve. Start from Curve > Polyline; pick points and finish with Enter. The option cues include Close, PersistentClose, Line/Arc mode, Length, Direction, Center and Helpers. Cue names do not establish unspecified defaults.

## Implemented contract and host decisions

- Menu and CMD start the same Rust session. Viewport picks and coordinates share the same point validator. The public command remains `Polyline`; icon identity remains `CurvePolylinePolyline`.
- Straight Line mode, Enter finish, Close after three distinct points, PersistentClose on final commit, and next-segment Length are supported. `Length` prompts for a numeric value; `Length=5mm` is also accepted. Direction picking is rejected until the requested length is supplied.
- Coordinates accept x,y or x,y,z in internal millimetres; explicit mm/cm/in suffixes and relative `r` coordinates are host choices. Unsuffixed numbers use mm; units do not change with the host display-unit schema.
- Host limits: at least two points for open output, three for Close; adjacent segments at least 1e-7 mm; finite coordinates bounded by 1e9 mm; at most 4096 input points. Invalid input retains the active command.
- Mouse picking uses the native viewer's per-view construction-plane projection and DPI conversion (2026-10-05 core workspace update). Looking Down/Perspective use XY, Side View YZ, Through Finger XZ; these defaults are host choices. Typed coordinates use the active plane, including relative offsets; F4 supplies its origin. Parallel rays preserve the command. Gem/surface-aligned planes and snapping remain pending.
- Commit creates one native `Part::Feature` wire via the host Part/OpenCascade bindings, in one document transaction. Esc discards pending input. Green connected transient segments and square accepted-point handles are shown in all native document views, with a hover segment constrained by the same Rust Ortho/Length logic. No document objects or Undo entries are created until Enter commits. Enter removes handles; Esc removes the unfinished preview. Previews are unpickable and excluded from model fit bounds. Successful-only history is updated after commit.
- Active-document change/deletion and workbench deactivation cancel pending input. Source selection is not used for this initial creation slice. Undo/Redo and FCStd persistence use host transactions and native shapes.

## Remaining behavior

Arc mode, Direction/Center arc constraints, Helpers and AutoClose/Alt are not implemented. PersistentClose currently closes the committed output; continuously closed dynamic preview is absent. Unsupported options report an error rather than approximating an arc as straight segments. This feature is not a full Matrix9 equivalent.

## Evidence

`rust/tests/curve_session.rs` and `tests/curve_smoke.FCMacro` cover semantic state, actual Qt mouse/keyboard events, open/closed wire topology, Length, invalid input, Esc, native Undo/Redo and FCStd reload. Current runtime results and limitations are recorded in the Curve validation report.

Latest user correction (2026-10-05): successive mouse clicks must display the picked chain before Enter, as shown in the three packaged `skills/openmatrix9-ui/assets/matrix9-polyline-*.png` references. Command uses a single selectable history/prompt/input document; only the input suffix is editable. Native evidence and limits are in `docs/validation/2026-10-05-polyline-console.md`. No PDF reread was required. Reinstall the Curve application event filter when starting, then prioritize CoreMouse, matching the existing native point-tool precedence.

2026-10-05 runtime update: parenthesized supported-option prompts, initial-letter CMD inputs and native option clicks use the shared handler. U/Undo removes the last accepted point before commit without a document Undo entry. See `docs/validation/2026-10-05-grid-command-options.md` for native geometry/input evidence, final module and remaining Arc/Helpers gaps. Skill persistence of this contract is pending user consent.
