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
