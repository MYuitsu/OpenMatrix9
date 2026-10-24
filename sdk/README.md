# OpenMatrix9 plugin SDK

SDK delivery requirements live inside the existing milestone/L2 owners. See the
[plan integrating SDK work into milestones](../OM9_Milestones/SDK-INTEGRATION-PLAN.md)
for owner-specific checklists, existing-code retrofit work and public plugin acceptance.
New SDK-eligible features ship their shared service and SDK adapter together in the same owner delivery.

OM9 plugins use a public Rust crate, `om9-sdk`, and a versioned C ABI. The host owns the document, permissions, geometry validation, transactions and Undo. A plugin requests changes through SDK services rather than linking against the application, egui, wgpu, OCCT or openNURBS internals.

This is the initial developer SDK, not a Rhino binary compatibility layer. It can support ports of Rhino plugins after their code and dependencies have been adapted. Renaming a `.rhp` to `.om9p` does not convert it.

## Contents

| Component | Location |
|---|---|
| Public Rust types, `Plugin` trait and `Host` services | [`crates/om9-sdk`](../crates/om9-sdk/) |
| Native entry point and ABI contract | [`abi.rs`](../crates/om9-sdk/src/abi.rs) |
| Host integration and package tooling | [`crates/om9-plugin-host`](../crates/om9-plugin-host/) |
| Complete example plugin | [`examples/hello-plugin`](examples/hello-plugin/) |
| Exposed APIs and future candidates | [`API-INVENTORY.md`](API-INVENTORY.md) |
| AI migration workflow | [`openmatrix9-plugin-port`](../skills/openmatrix9-plugin-port/SKILL.md) |

## Separate SDK distribution

Create a source SDK ZIP for developers who do not have the full OM9 checkout:

```powershell
rtk cargo run -p om9-plugin-host --bin om9-plugin -j 2 -- sdk-bundle H:/OpenMatrix9 L:/om9-target/openmatrix9-sdk-0.1.0.zip
```

Extract into a new directory and start with `BUNDLE-README.md`. The SDK crate's
manifest is made independent of the OM9 workspace, and the included example
depends only on that SDK and public serde packages. It can build without OM9
core/application/kernel sources. The ZIP contains sources and the C ABI header;
the OM9 host and package tool are separate. Existing ZIP outputs are not overwritten.

## Write a plugin

Create a Rust library with `crate-type = ["cdylib"]` and depend on `om9-sdk`. Within this checkout, the example uses the path dependency; an external plugin project should use a released SDK distribution or its checked-out SDK path. No public crates.io release is implied.

```rust
use om9_sdk::{CommandDescriptor, Host, Plugin, PluginDescriptor, Result, SDK_VERSION, Value};

#[derive(Default)]
struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "org.example.point".into(),
            name: "Example Point".into(),
            version: "0.1.0".into(),
            sdk_version: SDK_VERSION,
            commands: vec![CommandDescriptor {
                id: "ExamplePoint".into(),
                title: "Example Point".into(),
            }],
        }
    }

    fn execute(&mut self, host: &mut Host<'_>, command: &str, _args: Value) -> Result<Value> {
        if command != "ExamplePoint" {
            return Err(om9_sdk::SdkError::new("unknown_command", command));
        }
        let id = host.add_point("Example point", [10.0, 0.0, 0.0])?;
        Ok(om9_sdk::Value::String(id))
    }
}

om9_sdk::export_plugin!(ExamplePlugin);
```

`export_plugin!` exports `om9_plugin_entry_v1`. The host constructs a fresh plugin instance per request. `Host` is scoped to that synchronous invocation and cannot be retained, sent to a worker or used after the call returns. Store durable data only through an explicit host-supported API; this initial SDK has no persistent plugin-storage service.

The initial services are document information, the document's display unit and stated tolerance, typed length/angle/coordinate parsing, typed builder/command parameter validation, curve information, evaluation and construction, distance/angle measurement, project state and save requests, recorded change batches, object extensions, placement matrices, object and layer inspection with filtered paged queries, object attribute read/write and rename, layer create/update/current/assignment, progress/warning reports, selection read/write, simple geometry inspection, adding points/lines/polylines/circles, deleting objects and transforming objects. Query `host.capabilities()` before relying on a service. Additional OM9 internal geometry algorithms are candidates for future SDK versions; their presence in the repository does not make them callable from a plugin.

All SDK geometry coordinates and translations use millimetres. Object geometry is local to the object; compose it with the `ObjectInfo.placement` matrix for world coordinates. Matrices are row-major and multiply column vectors, with translation in column 3. Circle `x` and `y` are perpendicular unit basis vectors. Object IDs are opaque strings, not indices or memory addresses.

`host.document_units()` reports the display-unit symbol, millimetres per display unit and the document's own tolerance (`None` when the document states none; the host never invents a default). `host.parse_length`, `host.parse_angle` and `host.parse_coordinates` use the host's typed parsers: a bare number takes the document display unit, a trailing token (`mm`, `cm`, `m`, `in`, `ft`, `um`; `deg`, `rad`, `°`) carries its own unit, and non-finite or malformed text is an error, never zero. Parsed lengths and coordinate components are canonical millimetres and parsed angles are canonical degrees with an explicit radian value. A parsed coordinate tuple is pre-CPlane: resolving it to world space belongs to the host view/CPlane owner, so a plugin must not apply a placement twice.

`host.query_objects` filters by layer, exact geometry kind, label substring or selection, pages the result (default 256, maximum 1024 per page) and reports the total match count. `host.object_attributes`, `host.set_object_attributes` and `host.rename_object` read and stage attribute edits; reads merge the command's own staged records, so a plugin sees what it has already staged. One command stages at most one kind of source write per object — transform, attributes or rename — matching the application's typed property-edit model; mixing them is refused with the `conflict` error. An attribute edit requires the object's layer to be editable; object-level `locked`/`hidden` flags remain editable so a plugin can unlock or unhide an object.

`host.layer_create`, `host.layer_update`, `host.set_current_layer` and `host.assign_objects_to_layer` stage the document owner's own layer plans in the same transaction: one layer write per layer, the document's name grammar/uniqueness/budget, and the source layer must be editable for an assignment. Assignment clears the object colour so the layer colour applies. A layer created by a command can become current and receive newly added objects in that same command: the transaction evaluates the proposed layer record and its captured parent chain. Adding objects to a layer whose flags change in the same command is still refused by the application's permission-read rule. Assignment is an attribute write, so it cannot be combined with a transform, attribute edit or rename of the same object.

`host.set_object_placement` writes one object's affine placement matrix (row-major, object-local to world) through the document transaction: the last row must be `[0,0,0,1]` and the linear part invertible (`placement` refusal otherwise), the object must already exist, and it is one source write per object per command.
`host.object_extensions` reads an object's named extension slots (verbatim JSON-object metadata such as jewelry weight or provenance), and `host.set_object_extension`/`clear_object_extension` write one slot through the document's extension mutation (`extension` refusal for a bad slot/version/payload; one source write per object per command, so it conflicts with a placement/attribute/label/layer write). An object created in the same command must be tagged in the next command.
`host.last_change` returns the change batch the host recorded for the last committed change (plugin command, or host Undo/Redo) (real `from_revision`/`to_revision`, cause `edit`, per-object/layer entries), so a following plugin command can react to what the previous one changed. Other host edits outside these paths are not in this feed; compare `document_info().revision` for those.
`host.project_state` reports the host project path (absent for untitled) and whether the document is dirty; it is file identity, not document state. `host.save_project` asks the host to save after the command commits: it is refused before commit when the project has no path, the command edits are published first, and a save failure is reported as a warning with the document still dirty. `host.save_project_as` asks the host to start its own Save As flow after the commit so the user chooses the destination; a plugin never sees or supplies a path, and a command cannot combine save and save-as. A plugin cannot open, create or replace the active document.
`host.object_info` reads one object's identity/placement record without paging a query. `host.object_placement` reads one object's local placement matrix. `host.clear_current_layer` clears the current-layer pointer (no-op when already clear). `host.block_definitions` lists block definitions (label, member count, revision; read-only). `host.delete_group` removes only the group record (members stay; one Undo restores). `host.create_group` creates one group from existing members (one Undo). `host.groups` lists the document group records (members and revision; read-only). `host.measure_mesh_area` sums fan-triangulated face areas of a mesh (object-local mm²). `host.set_document_tolerance` writes the document tolerance policy (validated, one Undo). `host.layer_info` reads one layer record with the same effective permissions as the layer list. `host.layer_order` lists the layer order (bottom to top), including a layer this command created. `host.report_measurement` sends one measurement label through the command event log (bounded Progress line). `host.object_summary` counts the document's objects by geometry kind (staged additions/deletions included). `host.measure_curve_radius` returns a native circle/arc radius (other curve kinds are `unsupported`). `host.measure_curve_length` returns a curve's sampled length (33-point sweep) in object-local millimetres. `host.object_bounds` returns one object's object-local bounding box, size and center (point position, 33-sampled curve points, or mesh vertices; other kinds are `unsupported`). `host.measure_distance` and `host.measure_angle` return canonical millimetres and undirected degrees using the analysis owner's rules: points beyond the measurement bound or a zero-length direction are a `measure` refusal, and measuring stages no mutation.

`host.add_arc` builds the circle through three points and publishes it with an increasing angle domain; `host.add_interpolated_curve` builds a NURBS curve through points with odd degree 1..=11 and chord-length or uniform knots; `host.add_nurbs_curve` publishes a control-point NURBS record (one weight per pole, `poles + degree + 1` non-decreasing knots, clamped domain from the knot vector); `host.add_ellipse` publishes the exact rational-quadratic ellipse on an orthonormal frame, `host.add_rectangle` publishes a closed four-segment polycurve, `host.add_polygon` closes a point list into one polygon (three or more distinct points, no degenerate edge), `host.add_circle_on_frame` places a full circle on an arbitrary orthonormal frame (not just XY), and `host.add_mesh` publishes a polygon mesh from vertices and face index lists, validated by the mesh record. Frames must be unit and perpendicular, sizes positive. Both publish through the command's own transaction on the current layer, so one Undo removes them, and collinear arcs, even/out-of-range degrees or degenerate point sets are typed refusals.

`host.curve_segments` lists a curve's segments (one for a native curve, many for a polycurve) and `host.evaluate_curve_segment` evaluates one by index with the same D0-D3/1024-sample rules. `host.curve_definition` reports a curve's defining data: NURBS degree, poles, weights, knots, domain, periodic, reversed and seam; a circle's native and optional arc angle domain; a polycurve's segment count. Control data is bounded, and a larger curve is a `budget` refusal rather than a truncated definition. `host.curve_semantics` reports closedness, planarity and the sampled endpoints under an explicit tolerance or the document's own stated tolerance — a document that states none is a `tolerance` refusal, and a curve whose sampled length is at or below the tolerance is refused as degenerate rather than reclassified.

`host.curve_info` reports a curve's kind, native parameter domain and, for NURBS, degree/pole count/periodic/reversed; `host.evaluate_curve` evaluates line, circle (including an arc angle domain) and NURBS curves at native parameters with D1..D3 derivatives in object-local millimetres, at most 1024 samples per call. The object placement is not applied, so compose it exactly once. A polycurve has no single parameter domain and needs an explicit segment parameter, which this SDK version does not expose, so it is refused.

`host.set_selection`, `host.add_to_selection`, `host.remove_from_selection` and `host.toggle_selection` stage the command's selection. Every ID must be a canonical, selectable root with pick geometry, validated against the scene the command will publish, so one bad or locked ID refuses the whole command before publication; `host.selected_objects` reads back the staged selection. Selection is not document state: it creates no transaction or Undo entry, and a refused command leaves the previous selection untouched.

`host.progress` and `host.warning` report one-line progress or warnings into the host command event log — the same log the Command line and observers read. One command may report at most 32 messages of at most 512 bytes each (a longer message or excess count is a `budget` refusal), reports never create an Undo entry, and a command that returns an error publishes nothing, reports included.

`host.validate_parameter` and `host.apply_parameters` validate typed builder/command parameters with the document's own option domain: a typed wire form (`{"kind":"length","millimeters":2.5}`, `{"kind":"angle","degrees":90}`, `{"kind":"count","value":6}`, choice/boolean/text), bounds in canonical units, the optional step grid, the integer count rule, a typed default, required/unknown-name checks and text size. A bare JSON number is refused because a quantity needs a unit, and the unit is the tag, so a length can never be read as an angle. `apply_parameters` returns the filled typed map; a call is bounded to 128 declarations and 128 supplied values.

## Build and package

One source project can produce one `.om9p` package for several platforms. Each native target still requires its own compilation, toolchain, dependencies and checks:

| Rust target | Library produced |
|---|---|
| `x86_64-pc-windows-msvc` | `.dll` |
| `aarch64-pc-windows-msvc` | `.dll` |
| `aarch64-apple-darwin` | `.dylib` |
| `x86_64-apple-darwin` | `.dylib` |

Use the example's manifest and the package tool to collect these outputs. A single pipeline may coordinate Windows and macOS builders. Having a common Rust source, egui or wgpu does not prove that a plugin's native dependencies or the OM9 host support both platforms.

From the repository root on this Windows machine:

```powershell
$env:CARGO_TARGET_DIR = 'L:/om9-target'
rtk cargo build -p om9-hello-plugin --release --target x86_64-pc-windows-msvc -j 2
rtk cargo run -p om9-plugin-host --bin om9-plugin -j 2 -- pack sdk/examples/hello-plugin/descriptor.json L:/om9-target/hello-0.1.0.om9p x86_64-pc-windows-msvc=L:/om9-target/x86_64-pc-windows-msvc/release/om9_hello_plugin.dll
rtk cargo run -p om9-plugin-host --bin om9-plugin -j 2 -- inspect L:/om9-target/hello-0.1.0.om9p
```

`pack` takes `descriptor.json output.om9p target-triple=library [...]`, calculates digests and refuses to overwrite an existing output. Use a fresh output filename for another build. `inspect` validates package bytes, manifest, hashes and library target format without loading native code. To add a Mac build to the same package, supply another `target-triple=library` argument pointing to the actual Mac-produced `.dylib`; no Mac compilation is performed by `pack`.

An `.om9p` is a ZIP containing `manifest.json` and target libraries. The initial tool writes uncompressed (Stored) ZIP entries; v0 has no plugin asset-access API or private native-dependency loader. Format version 1 contains a plugin descriptor and a target-triple map:

```json
{
  "format_version": 1,
  "plugin": {
    "id": "org.example.point",
    "name": "Example Point",
    "version": "0.1.0",
    "sdk_version": 1,
    "commands": [{ "id": "ExamplePoint", "title": "Example Point" }]
  },
  "libraries": {
    "x86_64-pc-windows-msvc": {
      "path": "native/x86_64-pc-windows-msvc/plugin.dll",
      "sha256": "<measured SHA-256 of that library>"
    },
    "aarch64-apple-darwin": {
      "path": "native/aarch64-apple-darwin/plugin.dylib",
      "sha256": "<measured SHA-256 of that library>"
    }
  }
}
```

The SHA-256 strings above illustrate the fields; the package tool must calculate real digests. Checksums detect changed payloads; they do not identify a trusted publisher or sandbox native code. Only include platform entries whose corresponding libraries actually exist and have been checked. Adding a macOS entry does not convert a Windows DLL.

The C boundary transfers bounded UTF-8 JSON through synchronous callbacks. ABI v1 limits messages to 8 MiB, checks the host table version and size, copies replies before the callback returns, and keeps allocation/free on their originating side. It does not pass Rust-owned containers or native kernel pointers across libraries.

Native plugins execute trusted machine code in the host process. Rust unwinding panics are caught at the SDK entry boundary; aborts, invalid pointers and native faults are not contained by that mechanism.

## Run the example

With the matching OM9 host built, execute `CreateProfile` and save the result:

```powershell
rtk cargo run -p om9-app --bin om9 -j 2 -- --plugin-run L:/om9-target/hello-0.1.0.om9p CreateProfile sdk/examples/hello-plugin/arguments.json L:/om9-target/hello-result.om9
```

`--plugin-run-on package.om9p command args.json input.om9 output.om9` runs against a loaded document. In the OM9 Command area, load and invoke the example using:

```text
PluginLoad "L:/om9-target/hello-0.1.0.om9p"
PluginRun org.openmatrix9.hello CreateProfile {"radius_mm":5}
```

`PluginLoad` takes a JSON string path, and `PluginRun` takes a plugin ID, command ID and JSON object. Both CLI and Command-area execution use the same host services. The example creates a line, circle and closed polyline; `{"cancel":true}` exercises a failure after staging those edits so they must all roll back. See the current validation evidence for actual results on this host.

## Validate a port

Check observable outcomes for the target command: geometry and units, placement, permission refusal, selected-object identity, one successful command transaction, Undo/Redo and failure/Cancel preserving the document and previous Redo. A library loading or returning an object ID alone is insufficient proof. Unsupported NURBS/BRep/UI requests must report a capability gap rather than silently emit substitute geometry.

Use `CARGO_TARGET_DIR=L:/om9-target` and `-j 2` for builds on this machine. Keep build outputs outside the checkout. See the root [AGENTS.md](../AGENTS.md) for validation retention and public-source requirements.

## Platform and compatibility status

The SDK contract and simple plugin source are designed to be portable. Windows runtime results belong in the current implementation evidence and root progress ledger. macOS host/plugin execution remains unverified until the native adapters, process owner, dialogs and target builds pass on macOS. The existing product build and release pipeline are Windows-specific; see the concrete blockers in the [inventory](API-INVENTORY.md#platform-boundaries).

This SDK provides OM9 services with explicit capabilities, not RhinoCommon or Rhino C++ ABI compatibility. Source ports replace Rhino APIs with OM9 SDK operations. Binary-only managed or native plugins may require recovery or a behavior-based rewrite and cannot be promised automatic conversion. Preserve applicable notices and keep private inputs outside public source and packages.
