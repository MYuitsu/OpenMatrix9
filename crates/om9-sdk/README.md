# OM9 SDK v1 (experimental)

This public Rust crate contains the standalone OM9 plugin contract, geometry
DTOs, safe invocation-scoped host wrappers and a small versioned C ABI. It has
no dependency on OM9 core internals, egui, a geometry kernel or Rhino.

Plugin authors use a Rust `cdylib` crate, implement `om9_sdk::Plugin`, and invoke
`om9_sdk::export_plugin!(MyPlugin)`. The exported symbol is
`om9_plugin_entry_v1`. Rust types stay behind the ABI; only a fixed `repr(C)`
host table, function pointers and synchronously copied UTF-8 JSON cross it.
No allocator ownership is transferred. Messages are limited to 8 MiB. Errors
contain stable machine-readable `code` and human-readable `message` strings.

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
om9-sdk = { path = "../om9-sdk" }
```

The path is an example for a separately supplied SDK directory. The repository
also supplies an example plugin and packaging tool; see its SDK guide for the
actual workspace layout and host invocation. This crate can be distributed as
source independently; it requires only serde and serde_json.

Supported SDK calls:

- `capabilities()`: explicitly advertised host capabilities.
- `document_info()`: active document identity, revision, layer and units.
- `objects()`, `selected_objects()`, `layers()`: copied domain data and IDs.
- `object_geometry(id)`: supported geometry in object-local coordinates.
- `add_point`, `add_line`, `add_polyline`, `add_circle`, `add_object`.
- `delete_object`, `transform_object`: host-validated edits.
- `snap_context`, `snap_query`, `snap_constrain`, `snap_constrain_hit`
- `snap_track_anchor`, `snap_query_tracked` for SmartTrack source anchors

### Snap queries and point constraints

Use `Host::capabilities()` to discover the snap service. A listed method
can still return `unsupported` for a kind or source the host cannot
evaluate. `snap_context` captures the current scene, view, camera and CPlane
for this invocation. Its tolerance and plane origin are in millimetres.
Pass `viewport_origin`, `viewport_size`, `cursor` and `radius_px` in the
same logical pixel coordinate system. Pass `from_mm`, tracking anchor
positions and constraint points in World millimetres. Use the context
before staging document changes.

Request End, Mid, Center, Quadrant, Point, Near, Tangent or Perpendicular
using their lowercase names. Tangent and Perpendicular need the previous
point in `from_mm`. Use `intersection` for true 3D hits or
`apparent_intersection` to allow a projected hit in the active view.
An apparent hit reports kind `Apparent Intersection`, the selected source's
World point before Project and both source identities. A true hit reports
`Intersection` in either mode. The query returns `None` for a completed miss.

Hosts built with OCCT also support `surface` and `smarttrack`. On the
synchronous host, Surface and SmartTrack return `unsupported`. A query
that needs asynchronous source-edge or general-intersection evaluation
can also return `unsupported` when an eligible source requires it. This
includes intersections involving NURBS or Polycurve and apparent
intersections involving circles in the synchronous host. Source-backed
surface and edge queries may return `pending` while the worker runs.
End the invocation before staging changes, then get a fresh context and
repeat the query in a later invocation. A `stale` error requires a fresh
context and query. Prompts, publication and Undo belong to the host.

- `snap_constrain` resolves caller-supplied coordinates. A free pointer
  uses Grid Snap, Ortho, Planar, `shift` and `base_mm`. Setting
  `object_snap` accepts the supplied point as an already resolved snap
  and applies Project when enabled. Source validation requires an issued hit.
- `snap_constrain_hit` takes the unchanged returned hit in the same
  context and invocation. With Planar and Project on, the latest matching
  ordinary or tracked query supplies `from_mm` for the parallel plane.
  Project uses the command CPlane when Planar is off or the base is absent.
- A `smarttrack` query accepts one or two caller-supplied positions in
  `tracking_anchors_mm`. For a source-backed anchor, call
  `snap_track_anchor` on the issued hit and pass the token to
  `snap_query_tracked`. Leave `tracking_anchors_mm` empty on the tracked
  query. Reuse tokens with the same plugin, scene publication and CPlane.
  A free guide result has no source identity and cannot be retained as an
  anchor.

The packaged `hello-plugin` example includes `SnapDemo`, which obtains a fresh
context, queries the supplied logical cursor, and constrains an issued hit.
Its optional JSON arguments include `kind` or `kinds`, `width`, `height`,
`cursor_x`, `cursor_y`, `radius_px`, `from_mm`, `tracking_anchors_mm`,
`tracking_anchors`, `retain_anchor` and `create_point`. Its viewport
origin is `[0, 0]`.
When `create_point` is true, a hit creates one point through the host's normal
transaction. It propagates `pending` to the caller, which can retry in a later
invocation before creating anything.

```rust
pub fn intersection_input(
    host: &mut om9_sdk::Host<'_>,
    viewport_origin: [f64; 2],
    viewport_size: [f64; 2],
    cursor: [f64; 2],
    previous_point_mm: Option<[f64; 3]>,
) -> om9_sdk::Result<Option<[f64; 3]>> {
    let context = host.snap_context()?;
    let hit = host.snap_query(om9_sdk::SnapQuery {
        context_token: context.token.clone(),
        viewport_origin,
        viewport_size,
        cursor,
        radius_px: 8.0,
        kinds: vec!["apparent_intersection".into()],
        from_mm: previous_point_mm,
        tracking_anchors_mm: vec![],
    })?;
    hit.as_ref()
        .map(|hit| host.snap_constrain_hit(&context, hit))
        .transpose()
}
```

All geometry coordinates, circle radii and placement translations use mm.
`ObjectInfo::placement` is a row-major 4x4 matrix multiplying a column vector;
geometry reads are local, and this matrix maps local points to world points.
`transform_object` applies a world-space delta by premultiplication:
`new_placement = matrix * old_placement`. `DocumentInfo::unit` reports the
document display-unit symbol and does not change the SDK's canonical mm units.
Circle axes are perpendicular unit vectors. Hosts validate geometry, object
permissions and document publication; plugins never own the document or its
Undo journal. Delete/transform return JSON null; add returns a JSON ID string.
Successful descriptor replies return `PluginDescriptor` directly, and command
replies return the plugin's JSON value directly. A nonzero ABI status uses
`SdkError` as the reply body. Query `capabilities` before depending on services;
unsupported geometry must return an error, never an approximate success.
`Describe` is host-free and accepts a null host table; `Execute` requires a
valid v1 host table for the duration of the invocation.

Each invocation constructs a fresh `Plugin::default()` instance. `Host` cannot
leave its invocation or move to a worker thread. No callbacks, host contexts,
or pointer values may be retained. ABI v1 does not offer background callbacks,
UI panels, filesystem/network access, kernel Boolean operations or Rhino API
compatibility. Native plugins are trusted code and are not sandboxed; a Rust
panic with unwinding is converted into an error, but process aborts, native
faults and invalid foreign pointers cannot be contained by this contract.

Build the same plugin source independently for Windows (`.dll`) and macOS
(`.dylib`), including each supported CPU architecture. A single `.om9p` package
may contain these separate libraries. The host selects a compatible binary.
Neither egui/wgpu nor this ABI makes a Windows binary executable on macOS.
Cross-platform source and package design do not establish tested macOS runtime
compatibility: build and execute the plugin on each promised target.
