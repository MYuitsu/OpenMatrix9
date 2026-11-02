> **Current target:** standalone Rust OM9; [M03 geometry owner](<../../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-001 - Line Polyline Point Creation/l3.json>) and [menu adapter](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-001 - Curve · PolyLine - menu binding/l3.json>). Legacy FreeCAD evidence below remains historical.

## Standalone update — 2026-10-25

`M`/`Mode` toggles Line/Arc; `M=L` and `M=A` select explicitly. Keyboard initials and inline option clicks use the same `Controller::submit` handler. Polyline Mode takes precedence over the global Mid snap shortcut. Supported option initials are underlined by the UI without inserting combining characters in the transcript. P/PersistentClose and H/Helpers toggle their existing flags, with the live prompt and history refreshed immediately. These actions preserve nonempty drafts, paste, IME and unrelated editor focus.

Successive Line and Arc segments form one canonical analytic `Curve::Polycurve`; tessellation is display-only. Arc defaults to the preceding segment's end tangent. When no preceding segment supplies a tangent, the session asks for Direction. An explicitly picked Direction persists until replaced; Center selects the next arc's center. Degenerate/collinear arcs and inconsistent Center radii fail without losing accepted points. Center currently constructs the minor arc and rejects antipodal endpoints; major/semicircle Center selection is not certified. Numeric Length applies to the next Line segment only. Coordinates use the existing unit/CPlane parser.

Enter commits the complete chain in one transaction. U/Undo removes a pending segment, document Undo/Redo restores the entire committed curve, and Cancel/Esc leaves no document object. Menu dispatch and CMD are checked against the same geometry oracle. Mixed line/arc display and its allocation admission use matching sample counts; gaps are rejected rather than connected by an invented edge.

The pre-existing PersistentClose flag still restarts a session after commit; continuously closed preview is unfinished. Helpers currently changes the flag/prompt; dynamic tracking is unfinished. Exact mixed-Polycurve 3DM export, complete M03-UI acceptance and macOS are outside this fix. [Detailed evidence](../validation/2026-10-25-command-polyline-shortcuts/README.md).

Standalone parameter-interface follow-up (2026-10-25): `PolylineOptionInterface` now supplies every existing Polyline parameter to the prompt, keyboard, pointer and typed-name paths from one current-value snapshot. Availability, C ambiguity, Length input, Direction/Center values and invalid/stale-stage activation are tested in [the interface report](../validation/2026-10-25-polyline-parameter-interface/README.md). Geometry and existing flag lifecycle semantics are preserved.

## Historical FreeCAD implementation

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
