# SDK API inventory

Audit scope: standalone Rust OM9, 2026-10-21. The exposed surface below is the initial SDK contract (SDK/ABI version 1, developer release v0). The remaining tables identify implementation candidates; they are not promises of exported API or equivalent Rhino behavior. Source anchors identify the audited definitions; tests named here are coverage pointers, not fresh execution receipts. Use the root live ledger for measured results.

## Exposed v0

The public boundary is [`om9-sdk`](../crates/om9-sdk/src/lib.rs), with [`abi.rs`](../crates/om9-sdk/src/abi.rs) defining the entry point, host callback table, versions, JSON transport and message limit. The SDK has serde dependencies and no dependency on internal OM9 model/UI/kernel crates.

| Public interface | Contract |
|---|---|
| `PluginDescriptor`, `CommandDescriptor`, `Plugin`, `export_plugin!` | Describe an identified plugin and commands, then execute through `om9_plugin_entry_v1`. |
| `Host::capabilities` | Discover services actually exposed by this host. |
| `Host::document_info` | Opaque document ID, name, revision, current layer and display-unit symbol. Geometry coordinates remain canonical mm. |
| `Host::document_units` | Display-unit symbol, millimetres per display unit and the document's own tolerance policy; `None` when the document states none, never an invented default. |
| `Host::parse_length`, `parse_angle`, `parse_coordinates` | The host's typed numeric parsers: bare number uses the document display unit, a trailing token carries its own unit, results are canonical mm/degrees (coordinates are pre-CPlane). Non-finite, malformed or unsupported text is a typed error; one field is bounded to 256 bytes. |
| `Host::progress`, `warning` | One-line progress/warning reports surfaced in the host command event log (same log as the Command line and observers); at most 32 messages of 512 bytes per command, no Undo entry, and none survive a command that returns an error. |
| `Host::validate_parameter`, `apply_parameters` | Typed builder/command parameter contract: kind-tagged wire values (length mm, angle degrees, ratio, count, choice, boolean, text), bounds in canonical units, optional step grid, integer count rule, typed defaults, required/unknown-name and text-size checks. A bare number is refused. One call is bounded to 128 declarations and 128 values. |
| `Host::objects` | Object IDs, labels, geometry kinds, layer IDs, visibility/selectability/editability, placement, generation and geometry/attributes revisions. |
| `Host::query_objects` | Filtered, ordered, paged object query (`layer`, exact geometry kind, label substring, selection only, hidden inclusion), total match count, page capped at 1024. Includes this command's staged additions and attribute edits. |
| `Host::object_attributes` | Stored and effective attribute values for one object (layer, colour, visible/hidden/selectable/locked, effective flags, revisions). |
| `Host::set_object_attributes` | One staged attribute edit per object (`color`, `visible`, `hidden`, `selectable`, `locked`); the object's layer must be editable, and the edit becomes one source write in the command's transaction. |
| `Host::rename_object` | One staged label edit; the label grammar and the one-source-write-per-object rule apply. |
| `Host::selected_objects` | Selected root object IDs, including the command's own staged selection. |
| `Host::set_selection`, `add_to_selection`, `remove_from_selection`, `toggle_selection` | Replace, extend, remove or toggle the selection; joining needs a canonical, selectable root with pick geometry validated against the scene that will be published, so one bad ID refuses the whole command. Leaving the set never needs selectability. Selection creates no transaction or Undo entry. |
| `Host::layers` | Layer IDs, name, visible/locked/selectable/current flags and colour; inspection plus the command's own staged layer records. |
| `Host::layer_create`, `layer_update`, `set_current_layer` | Create a layer (optional make-current), rename/recolour/reflag one layer, or select/clear the current layer. One layer write per command per layer; the document owns name grammar, uniqueness and the layer budget. A make-current layer can receive newly added objects in the same command. |
| `Host::assign_objects_to_layer` | Move objects to an existing or this-command-created layer; the object colour is cleared so the layer colour applies, and the source layer must be editable. |
| `Host::set_object_placement` | Write one object's affine placement matrix (row-major, local->world) through the document transaction; finite affine/invertible matrix required, one source write per object per command, created objects carry placement through their insert instead. 
| `Host::object_extensions`, `set_object_extension`, `clear_object_extension` | Read/write one named extension slot (verbatim JSON-object payload with a positive version) through the document extension mutation; bounded, one source write per object per command, and objects created in the same command must be tagged in the next command. 
| `Host::last_change` | Read the change batch recorded for the last committed change (plugin command `edit`, or host `undo`/`redo`) with real revision range and entity/layer entries. Other host edits are not in the feed; compare `document_info().revision` for those. 
| `Host::project_state`, `save_project`, `save_project_as` | Read the host project path and dirty state; request one atomic save after the command commits (refused before commit when untitled), or start the host Save As flow so the user chooses the destination. A save failure becomes a warning with the document still dirty; opening/creating/replacing the active document stays interactive and host-only. 
| `Host::object_info` | Read one object's `ObjectInfo` record (id, label, kind, placement, flags, revisions) without paging a query.
| `Host::object_placement` | Read one object's local affine placement (row-major 4x4, staged value included); parent/instance composition is not applied.
| `Host::clear_current_layer` | Clear the current-layer pointer; clearing an already-clear layer stages no mutation.
| `Host::block_definitions` | Read block definitions: id, label, member count and revision. Instance insertion/definition editing is not exposed.
| `Host::delete_group` | Delete one group record (member objects stay) as one transaction mutation, undone by one Undo.
| `Host::create_group` | Create one group from existing member ids (name bounded, members must exist); one `Mutation::GroupCreate`, undone by one Undo.
| `Host::groups` | Read document groups: id, name, member ids and revision. Group creation/editing is not exposed.
| `Host::measure_mesh_area` | Sum of fan-triangulated face areas in object-local square millimetres; a non-mesh is `unsupported`.
| `Host::set_document_tolerance` | Write the tolerance policy (absolute mm, relative, angular rad) through the document mutation; invalid values are `tolerance`, one Undo restores the previous state.
| `Host::layer_info` | Read one layer's record (name, effective visible/selectable/locked, current, colour) with the same mapping as `layers`.
| `Host::layer_order` | Layer order (bottom to top) as ids; a layer created by this command is appended.
| `Host::report_measurement` | One bounded measurement label as a `Progress` event line (≤512 bytes, ≤32 per command); empty/oversized text is a `message` refusal and a failed command discards it.
| `Host::object_summary` | Total and by-geometry-kind object counts, including objects this command already added and excluding ones it deleted.
| `Host::measure_curve_radius` | Native circle/arc radius in object-local mm; any other curve kind is `unsupported` (no guessed radius).
| `Host::measure_curve_length` | Sampled curve length (33-point sweep, object-local mm); a non-curve is `unsupported`; the value is an approximation, not an exact arc length.
| `Host::object_bounds` | Object-local bounding box/size/center from the point position, the owner's 33-sampled curve points, or mesh vertices; other geometry kinds are `unsupported` and the placement is not applied.
| `Host::measure_distance`, `measure_angle` | Distance between two world points in millimetres and the undirected angle between two directed lines in degrees, reusing the analysis owner's admission bounds (`1e9` coordinate, non-zero direction); measuring stages no mutation. |
| `Host::object_provenance` | Retained import source proof for one object: source name, lowercase-hex SHA-256, archive version, import mode, record kind, original uuid/index/family and disposition; an incomplete chain is `null`, never a guessed native import. |
| `Host::is_object_pickable` | Effective pickability (`effective_visible && effective_selectable`) from the object's layer chain and attributes; `locked` blocks editing, not picking; reading stages no mutation. |
| `Host::add_arc`, `add_interpolated_curve`, `add_nurbs_curve`, `add_ellipse`, `add_rectangle`, `add_polygon`, `add_circle_on_frame`, `add_mesh` | Build an arc through three points (increasing angle domain), an interpolated NURBS curve (odd degree 1..=11, chord/uniform knots), a control-point NURBS record (one weight per pole, poles+degree+1 knots, clamped domain), a rational-quadratic ellipse, a closed four-segment rectangle on an orthonormal frame, a closed polygon through three or more points, a full circle on an arbitrary orthonormal frame, or a polygon mesh from vertices/faces, as one transaction output on the current layer validated by the document geometry rule; one Undo removes it. |
| `Host::curve_segments`, `evaluate_curve_segment` | List a polycurve's segments (kind/domain/angle domain) and evaluate one by index with the same local-mm, D0-D3 and 1024-sample rules; a nested polycurve or an out-of-range index is refused.
| `Host::curve_definition`, `curve_semantics` | NURBS poles/weights/knots/degree/domain/periodic/reversed/seam, circle arc angle domain and polycurve segment count (bounded control data); closedness/planarity and sampled endpoints under an explicit or the document tolerance, with a missing tolerance or a degenerate curve refused. |
| `Host::curve_info`, `evaluate_curve` | Line/circle/NURBS identity (domain, degree, poles, periodic, reversed) and native-parameter evaluation with D1..D3 in object-local mm, up to 1024 samples; polycurve needs an explicit segment parameter and is refused. Placement is not applied. |
| `Host::object_geometry` | Only supported point, line, polyline and circle DTOs; local millimetre coordinates before object placement. |
| `Host::add_object`, `add_point`, `add_line`, `add_polyline`, `add_circle` | Request validated geometry publication under host ownership. |
| `Host::delete_object`, `transform_object` | Request guarded mutation by opaque ID. Transform uses a finite affine row-major matrix, translation column 3. |
| `SdkError`, `Result`, `Value` | Typed error code/message and JSON command arguments/results. |

The host owns atomic command publication and Undo. A plugin must propagate a failed host call; no partial command is considered successful. Error/Cancel paths preserve the source document and existing Redo. Do not expose `TransactionCoordinator` itself across the native ABI. The host call is synchronous, invocation-scoped and thread-bound; no retained callbacks, shared allocator ownership or mutable document pointers.

One command stages at most one source write per object: transform, attribute edit and rename are different edits and must be separate commands, mirroring the application's typed property-edit model. Mixing them is refused with the `conflict` error code before commit. Attribute reads and queries merge the command's own staged records, so a plugin reads back what it has staged in the same invocation.

## Candidate future: portable geometry

These are implemented internal Rust algorithms/data. Adding a public service still requires an owned DTO, capability/version, validation and behavior tests at the plugin boundary.

| Candidate | Definition anchors | Existing test coverage pointers |
|---|---|---|
| Typed units/parameters | `crates/om9-core/src/units.rs:120` `LengthMm`, `:173` `AngleDeg`, `:235` `AngleRad`; `coords.rs:16` `WorldPoint`, `:31` `LocalPoint`, `:56` `resolve`, `:67` `parse_local`; `request.rs:273` `OptionDomain::check` (typed dimensions, bounds, step, count integer rule); `command_schema.rs:63` `CommandSchema::apply` | Core units, coordinates and command-schema suites. Length/angle/coordinate parsing, the document tolerance policy and the typed parameter contract are exposed; `WorldPoint`/`LocalPoint` frame resolution stays host-internal with the view/CPlane owner. |
| Object attributes, layers | `model.rs:61` `Entity`, `:344` `Layer`, `:392` `Permissions`, `:615` `permissions_for_entity`, `:620` `permissions_for_attributes`, `:634` `display_attributes`, `:581` `layer_permissions`, `layer_permissions_with`, `permissions_for_attributes_with`; `layers.rs:21` `LayerAction`, `:42` `LayerPlan`, `:512` `check_layer_reads_with`; transaction `Mutation::{Attributes, Label, Layers, Insert}` | Object attribute read/write, rename and the layer service (create/update/current/assignment) are exposed with revisions. Output into a layer proposed by the same request is admitted against that record and its parent chain; an insert into a layer whose flags change in the same request is still refused by the permission-read rule. Groups, blocks/instances, change notification and snapshot/restore are not exposed. |
| Transform values | `geometry.rs:119` `Transform`; `:126` identity, `:134` translation, `:141` scale, `:162` compose, `:175` point | Core geometry/transaction and scene coordinate-frame suites. |
| NURBS/mesh values | `geometry.rs:189` `NurbsCurve`, `:296` `NurbsSurface`, `:341` `Curve`, `:439` `Mesh`, `:612` `Geometry` | Core geometry, Rhino semantic and curve/surface suites. Public v0 only exposes the simple subset above. |
| Interpolation and primitives | `curve_build.rs:135` interpolate, `:151` interpolate_any, `:234` circle_through, `:281` polyline, `:310` rectangle, `:378` arc_through, `:424` ellipse | `crates/om9-core/tests/curve_build.rs` |
| NURBS editing | `curve_build.rs:491` rebuild, `:590` insert_knot, `:655` elevate_degree, `:724` fit_points | `crates/om9-core/tests/curve_build.rs` |
| Curve evaluation/offset | `evaluation.rs:51` evaluate_curve; `offset_curve.rs:363` OffsetCurve, `:390` derivatives, `:456` point, `:461` curvature | Core evaluation and offset suites. Line/circle/NURBS evaluation with D1-D3, NURBS control data read, closed/planar classification and arc/interpolated-curve construction are exposed; offset, control-point editing and polycurve segment parameters remain host/internal candidates. |
| Surface query/creation | `surface_kernel.rs:95` evaluate, `:133` derivatives, `:181` normal, `:196` isocurve, `:263` refit, `:341` plane, `:386` cylinder, `:472` loft | `crates/om9-core/tests/surface_kernel.rs` |
| Surface sweep | `surface_kernel.rs:541` sample_curve, `:613` path_frames, `:682` sweep1, `:904` sweep2 | Surface-kernel and W8/W9/W10 suites; check each option's supported scope. |
| Measure | `measure.rs:26` angle_degrees, `:56` distance, `:71` distance_mm | Core measurement suites. |

Paths abbreviated after the first row are relative to `crates/om9-core/src/`. These functions validate owned inputs against existing limits. Do not make the SDK depend on private application modules simply to reuse their types.

## Candidate future: host geometry, exchange and viewport services

| Candidate | Source anchor | Boundary required |
|---|---|---|
| Exact Box/Sphere | `crates/om9-kernel/src/brep.rs:189` box_brep, `:196` sphere_brep | Host calls complete native operation, validates Rust-owned result and publishes it in the same transaction. |
| Canonical BRep reload/volume | `brep.rs:204` read_brep, `:211` source_volume | Do not expose OCCT source/pointers as editable public state; volume comes from canonical geometry, not display mesh. |
| Editable BRep 3DM conversion | `brep.rs:265` from_3dm, `:314` to_3dm | Requires matching OCCT/openNURBS runtime; errors and budgets remain explicit. |
| Import/export service | `crates/om9-exchange/src/imported.rs:58` BrepBridge, `:68` BrepExport, `:402` prepare_import; `staged.rs:133` prepare, `:329` publish | Keep staging, provenance, permissions and publication in host; no plugin-owned parallel document database. |
| STL export | `crates/om9-exchange/src/stl.rs:98` StlOptions, `:159` write_binary_stl | Validate units, winding and geometry policy. |
| Camera/CPlane values | `crates/om9-scene/src/camera.rs:62` Camera, `:83` CPlane, `:254` project, `:278` ray | Return copied portable values; request camera changes from host. |
| Picking | `crates/om9-scene/src/scene.rs:140` PickHit, `:153` revalidate, `:934` pick, `:946` pick_kind, `:970` pick_all | Return entity/subelement identity with revision and world hit. Host revalidates against current scene and permissions. |
| Preview overlay primitives | `crates/om9-render/src/overlay.rs:438` LineBatch::line, `:450` point | SDK-owned drawing DTOs consumed by host. Keep `wgpu::Device`, `Queue`, buffers and render passes private. |

`crates/om9-kernel/tests/brep_3dm.rs:9` is an ignored-by-default integration test requiring a supplied native payload and runtime. Native test presence alone does not establish a tested platform. `om9-3dm` is a mesh archive reader, not the complete document exchange API; prefer host exchange services for plugin-facing 3DM operations.

## Internal fixture only

Do not mechanically export every `pub fn` as an SDK capability.

| Existing public symbol | Actual scope |
|---|---|
| `om9-kernel/src/lib.rs:588 topology(kind)` | Counts/validity for hardcoded box, sphere or cylinder fixtures. |
| `lib.rs:628 transform_box` | Transforms a fixed 10×10×4 box, not an arbitrary input BRep. |
| `lib.rs:669 curve_point_and_derivative` | Fixed radius-5 quarter-circle. |
| `lib.rs:694 surface_point_and_normal` | Fixed radius-5 sphere. |
| `lib.rs:719 line_face_hits` | Fixed line through a box. |
| `lib.rs:257 drilled_volume`, `:410 fillet_volume`, `:438 offset_volume` | Proof measurements; no general editable Boolean/fillet/offset output contract. |
| `om9-core/src/brep_kernel.rs:296 KernelBackend` | Generic adapter contract; `UnlinkedBackend:317` refuses calls. Fixture backend in `tests/brep_kernel.rs:338` is not a linked product kernel. |
| Lifecycle/memory/create-destroy probes | Diagnostic and validation mechanisms; do not expose native process or kernel ownership to plugins. |

Rows in this table abbreviate paths relative to `crates/`. General Boolean, trim, join, fillet, topology naming, arbitrary native intersections, interactive input, SDK panels and display conduits remain capability gaps until implemented and validated as public services. Never substitute a mesh or bounding box and label it an exact solid operation.

## Platform boundaries

Current concrete platform dependencies must be ported before claiming full macOS host support:

| Source anchor | Dependency |
|---|---|
| `crates/om9-kernel/build.rs:33`, `:73`, `:129` | Windows OCCT/openNURBS `.lib` lookup and Windows native libraries. |
| `crates/om9-kernel/src/bridge.cpp:47`, `:184` | `windows.h` and `GetProcessMemoryInfo`. |
| `crates/om9-3dm/build.rs:22`, `:43` | `opennurbsStatic.lib` and Windows system libraries. |
| `crates/om9-native-owner/src/lib.rs:199` | Non-Windows native-process containment explicitly refused. |
| `crates/om9-native-owner/src/lib.rs:18`, `crates/om9-app/src/native_picker.rs` | Windows-only dialog implementation; non-Windows fallback branches. |
| `tools/ci/package_standalone.py:161`, `.github/workflows/release-windows.yml` | Windows x64 product package and release pipeline. |

Native `.dll` and `.dylib` files must be built for their actual OS/CPU target. `.om9p` collects them under target triples. SDK source portability, package validation, a Windows build and a macOS runtime pass are separate claims. Existing egui/wgpu portability does not certify these native adapters.

## Architecture ownership

M01 assigns document/geometry identity to `OM9-L2-01-03`, kernel lifecycle/operations to `OM9-L2-01-05`, units to `OM9-L2-01-08`, rendering of derived data to `OM9-L2-01-06`, and persistence/exchange policy to `OM9-L2-01-12`. Public SDK services adapt those owners rather than duplicate them. The relevant source contracts are rooted in `OM9_Milestones/M01 - CAD Shell, Viewport/L1.json`; imported proposed signatures and planning statuses are not runtime evidence.
