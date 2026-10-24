# Public plugin SDK, first developer slice

User direction, 2026-10-21: independently authored OM9 SDK and AI-assisted Rhino
plugin ports; Rust source shared across Windows/macOS, separately compiled
native libraries collected in one `.om9p`. No Rhino runtime or bridge.

## Ownership and reusable APIs

The [API inventory](../../../sdk/API-INVENTORY.md) identifies actual reusable
functions, future service candidates and native fixture probes that must stay
internal. This slice exports document/object/layer/selection inspection,
point/line/polyline/circle geometry, add/delete and affine object transforms.
It also exports the units owner's services: display unit and millimetres per
unit, the document's own tolerance policy (`None` when it states none), typed
length/angle parsing and the command prompt's coordinate grammar. Those calls
dispatch to `om9_core::units` and `om9_core::coords::parse_local`; the plugin
host adds no second parser, unit table or default tolerance. Advanced
NURBS/surface/BRep/UI algorithms are not implicitly exposed.

The document owner's read/write attributes are exported too: a filtered paged
object query, one attribute record read, and one staged attribute/label edit.
The host evaluates staged records with `Document::permissions_for_attributes`
(the same layer-chain rule the application uses) and emits `Mutation::Attributes`
or `Mutation::Label`, so Undo, permissions and revisions stay with the document
owner. One command stages at most one kind of source write per object, matching
the application's typed property-edit model; mixing transform, attributes and
rename is refused with `conflict` before commit. The transaction owner now admits
output into a layer proposed by the same request: the insert validates against the
proposed layer record and its captured parent chain, and the read/write overlap
guard excludes only layers this request creates (a layer whose flags change still
cannot receive output, because its captured permission would be stale).

Curve construction is exported over the owner builders: `add_arc` uses `curve_build::arc_through` (a circle with an increasing angle domain), `add_interpolated_curve` uses `curve_build::interpolate` (odd degree 1..=11, chord/uniform knots), and `add_nurbs_curve` validates a control-point record with `NurbsCurve::validate` and a clamped domain derived from its knots; `add_ellipse` and `add_rectangle` call `curve_build::ellipse`/`rectangle` on the caller-supplied orthonormal frame, `add_polygon` closes a point list through `curve_build::polyline`, and `add_circle_on_frame` builds a framed `Curve::Circle` validated by the curve record. Both stage one `Entity` through the same output path as `add_line`, so layer permissions, budgets and the single transaction/Undo rule stay with the document owner.

Placement writes are exported over the document transaction: `set_object_placement` validates the affine matrix (`Transform::validate`) and stages `Mutation::Placement` under the same one-source-write rule, so Undo restores the previous matrix and a created object keeps its insert placement.
Extension metadata is exported through the document's extension slot record: `object_extensions` returns the stored version/payload, and `set_object_extension`/`clear_object_extension` stage `Mutation::Extension` with core `Extension::new` validation. An object created by the same command is refused, because its inserted record is stamped once.
Project state is exported from the Controller file lifecycle: `project_state` reports the last opened/saved path and the dirty comparison, and `save_project`/`save_project_as` stage a request that the Controller fulfils after the command commits by calling `begin_file_operation(FileKind::Save)`/`SaveAs`, so the atomic save path, fingerprint/receipt and dirty state stay with the file owner. A failed save is a warning event, never a command failure after commit; a plugin cannot open or replace the active document.
Measurement is exported from the analysis owner: `measure_distance` calls `measure::distance_mm` and `measure_angle` calls `measure::angle_degrees`, so the 1e9 coordinate bound, the non-zero direction rule and the `atan2` degree range stay with the command that the UI analysis uses. Both return typed DTOs and stage no mutation.

Curve reading is exported from the geometry owner: `curve_info` reports kind, native domain and NURBS degree/poles/periodic/reversed; `curve_definition` adds NURBS weights/knots/seam and an arc angle domain under a control-data bound; `curve_semantics` runs `rhino_semantics::classify_curve` under an explicit or the document tolerance; and `evaluate_curve` runs `evaluation::evaluate_curve` at native parameters with D1-D3 in object-local millimetres. Samples are bounded and the placement matrix is not applied, so a plugin composes it once. Polycurve segment parameters remain host-internal, so a polycurve is refused rather than guessed.

The selection service is exported through the Controller session: `set_selection`/`add_to_selection`/`remove_from_selection`/`toggle_selection` stage IDs, the host validates each against the staged document permissions, and the Controller validates them again against the scene it is about to publish before committing, so a bad or locked root refuses the whole command instead of committing and then failing. Selection is applied after a successful command, creates no transaction or Undo entry, and `selected_objects` reads the staged set. Subobject refs, pick queries and drag transactions are not part of this slice.

The command feedback service is exported through the host command event log: `Host::progress`/`warning` stage bounded one-line reports and the Controller emits them as `Progress`/`Warn` events for the plugin command, so the Command line and its observers read plugin feedback through the same surface as built-in commands. Reports are not document state; a command that returns an error publishes nothing, reports included. Interactive prompts and input sessions are not part of this slice.

The typed parameter service is exported from the same core rule: `ParameterDomain`/`ParameterSpec`/`ParameterValue` mirror `request::OptionDomain` and `request::Value`; `validate_parameter` checks one typed value and `apply_parameters` runs `CommandSchema::apply` (defaults, required/unknown names, bounds, step, count integer, text size). The unit is the wire tag, so a bare number is refused and a length can never be read as an angle.

The layer service is exported through the same route: `layer_create`,
`layer_update`, `set_current_layer` and `assign_objects_to_layer` build
`LayerAction` plans and let the document owner validate name grammar, uniqueness,
the layer budget and permission reads. The host evaluates staged layer records
through `Document::layer_permissions_with` / `permissions_for_attributes_with`, so
a command reads back the layer flags it staged. The application's anti-bypass
rule is preserved: an insert into a layer whose flags change in the same command,
and an assignment off such a source layer, are refused rather than evaluated on
stale permissions. Groups, blocks/instances, change notification and
snapshot/restore are still not exposed.

`om9-sdk` is a separate crate with only serde dependencies. Its public DTOs do
not contain core model, egui, wgpu, OCCT or openNURBS types. The library exports
one versioned C entry point. A copied UTF-8 JSON request/reply protocol crosses
that boundary; allocations are freed on their originating side. Invocation
contexts and callbacks cannot be retained or moved to another thread.

The host library stages additions/deletions/placement changes in owned maps.
Repeated transforms compose before publication. A failed host call poisons the
whole command even if a plugin ignores the error. Successful output becomes
one ordinary `TransactionRequest`; the Controller prepares a scene and reserves
its memory before committing. SDK code never receives document mutation or
Undo-journal ownership. M01 owners remain `01-02` (command/transactions),
`01-03` (document/identity), `01-06` (derived scene), `01-08` (units).

The new `prepared_remaining_limits` query verifies coordinator authority and
epoch, then reserves candidate/preparation ownership before external scene
allocation. Existing mutation permission, stale stamp and Undo checks remain
the acceptance owner. Geometry and translation coordinates are mm;
`DocumentInfo.unit` reports display formatting only.

## Native exception and lifetime

Dynamic library loading and C callback marshalling require a small unsafe Rust
boundary in `om9-plugin-host` and `om9-sdk::abi`. No new C++ adapter or Python
production logic is introduced. `LoadedPlugin` is thread-bound; its library
outlives all calls and is unloaded before its unique extraction directory is
removed. Native plugins execute with application privileges. C ABI checks,
checked JSON and Rust unwind handling do not certify foreign code memory-safe
or contain process aborts, invalid pointers or native crashes.

Packages validate bounded ZIP entries, paths, duplicate names, versions,
declared target architecture and measured SHA256 before native loading. A hash
establishes payload integrity, not publisher identity. Inspection never loads
code. No implicit scanning/autoloading, private native dependency resolver,
asset-access API, interactive point picker, SDK panel API or asynchronous
callback service is present in this slice.

## Rendering integration

The SDK example exposed an existing display gap for line `Polycurve` geometry.
Scene admission now supports connected, unreversed direct-line polylines using
exact corners, including the closing point. Whole-object/reference picking
uses exact line projection, including perspective and near-plane clipping.
No fabricated global native curve parameter or subobject binding is returned.
Nonlinear/reversed/disconnected polycurves retain explicit diagnostics.

## Developer deliverables and limits

- `sdk-bundle` creates a standalone SDK source ZIP with normalized independent
  manifests, example, C header and migration skill. It contains no core/app or
  private source. The example must build after extraction outside this repo.
- `pack` collects independently built target libraries; it performs no
  cross-compilation and refuses existing outputs. `inspect` checks bytes only.
- Live CMD `PluginLoad`/`PluginRun` and CLI `--plugin-run`/`--plugin-run-on` use
  the same Controller service. Dynamic menu registration is future work.
- The sample implements `CreateProfile`, `MoveSelection`, `DeleteSelection`,
  `UnitsReport`, `AttributesDemo`, `LayerDemo`, `ParameterDemo`, `FeedbackDemo`, `SelectionDemo` and `CurveDemo`. Programmatic cancellation after
  staged geometry must leave the document, scene and previous Redo intact. This is
  not an interactive Escape-loop claim. `UnitsReport` is read-only: it stages no
  mutation and an empty Undo/Redo, and it reports the document's stated tolerance
  or `null`. `AttributesDemo` edits one object's colour/lock and renames another
  in one transaction. `LayerDemo` creates a current layer, adds a point into it, assigns an object to it and
  reads the staged layer list back in the same command. `ParameterDemo` declares length/count/choice parameters, applies their typed defaults and validates supplied values through the document option domain. `FeedbackDemo` reports progress and a warning and can fail on request, so the test covers both the committed and refused event sequences. `SelectionDemo` sets or extends the selection, including a refused unknown root. `CurveDemo` reads a curve's domain, definition, closedness/planarity and samples a line, circle and refused polycurve; `CurveBuildDemo` creates an arc, an interpolated spline and a control-point NURBS in one command; `MeasureDemo` returns a 5 mm distance and a 90° angle; `CurveShapeDemo` creates an ellipse, a rectangle and a polygon; `ProjectDemo` reads project state and can request a save.
- The units adapter is scoped to M01 `OM9-L2-01-08` L3-001/002/003/007. Tolerance
  is read, not written; builder parameter values/range/serialization
  (L3-004/005/006), CPlane frame resolution and interactive numeric input remain
  host-owned or open.
- SDK ABI v1 is experimental. Current native adapters/release tooling remain
  Windows-specific. Portable source and structural Mac package tests do not
  establish macOS runtime support.

Fresh commands, results and limitations are recorded in
[the SDK validation record](../../../docs/validation/2026-10-21-plugin-sdk.md),
[the units owner record](../../../docs/validation/2026-10-21-m01-08-units-sdk.md)
and [the document-attribute record](../../../docs/validation/2026-10-21-m01-03-attributes-sdk.md).

## Current service surface (round 44, 2026-10-21)

The owner records under `docs/validation/2026-10-21-*` are the measured source of truth.
This section only indexes what the SDK now exposes so the design record stays navigable.

- Units/coordinates: `document_info`, `document_units`, `document_tolerance`, `parse_length`,
  `parse_angle`, `parse_coordinates` (typed, unit-safe).
- Objects: add point/line/polyline/circle/framed-circle/arc/ellipse/rectangle/polygon/mesh,
  interpolated and control-point NURBS; query paged/filter; `object_info`;
  attributes read/write; rename; placement read/write; geometry read; bounds; extension slots
  read/write/clear; delete.
- Layers: list, single `layer_info`, create/update, set/clear current, order, assign objects.
- Curves: `curve_info`/`curve_definition`/`curve_semantics`, native-parameter evaluation with
  D0-D3, segment list and per-segment evaluation.
- Measurement: distance, angle, sampled length, circle radius, mesh area, object summary,
  measurement label through the event log.
- Selection: read, set/add/remove/toggle by persistent root IDs.
- Project: state (path/dirty), save, Save As (host dialog).
- History read: `last_change` covering plugin commands plus host Undo/Redo.
- Records: groups read/create/delete; block definitions read.
- Parameters: declaration, typed validation and application.

Still host-only or open: document open/create/replace, group membership edit, instance graph
and insertion, layer delete/reorder, snapshot/restore, subobject/pick services, and macOS
execution. None of the open items is reported as done anywhere.