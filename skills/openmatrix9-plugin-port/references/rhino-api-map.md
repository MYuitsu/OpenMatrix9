# Rhino to OM9 mapping guide

This is an OM9-authored migration guide, not copied Rhino SDK source or a binary compatibility promise. Read the actual `crates/om9-sdk` contract and `sdk/API-INVENTORY.md` in the selected checkout before using a row. The public SDK initially exposes simple geometry only. Names below identify familiar API families for analysis; overloads, defaults, return values and failure semantics require checking against the input plugin's actual use.

## Input routes

| Input | Practical route | Evidence needed |
|---|---|---|
| C#/VB/.NET source | Read command entry points and dependency graph, replace Rhino services with OM9 public calls, rewrite portable logic in Rust. | Original options, errors, units, geometry and side effects. |
| Managed `.rhp` without source | Inspect metadata/references and managed/native dependencies. Where authorized, recover only the relevant implementation or reconstruct behavior. | Recovery limitations, dependencies and test oracle; no claim of full source recovery. |
| C++ source | Preserve reusable algorithms and notices; port logic to Rust. Replace Rhino/native object ownership and services. | Geometry ownership, precision, topology semantics, error and lifetime assumptions. |
| Native `.rhp` without source | Bounded authorized binary analysis and behavior-based reimplementation; scope each command separately. | Observable inputs/outputs, uncertainty and unsupported dependencies. |

No route loads Rhino to provide runtime services. `.om9p` is packaging for OM9 native libraries; changing a binary filename is insufficient.

## Initial mappings

| Rhino-oriented use | OM9 initial SDK route | Difference to resolve |
|---|---|---|
| PlugIn identity and command registration | `PluginDescriptor`, `CommandDescriptor`, `Plugin::descriptor`, `export_plugin!` | ID/command collision rules and SDK version; no Rhino plugin lifecycle ABI. |
| `Command.RunCommand` | `Plugin::execute` with command ID and JSON arguments | Initial synchronous commands; no automatic interactive Rhino command state machine. |
| Active document information | `Host::document_info` | Opaque document ID/revision, canonical mm. No mutable document pointer. |
| Object-table enumeration | `Host::objects` | Copied object info; ID-based requests, permissions and placement. |
| Existing selection | `Host::selected_objects` | Inspection only. `GetObject` prompts/filter UI require future public services. |
| Layer-table reads | `Host::layers` | Inspection only; layer creation/assignment/state mutation are not exported initially. |
| Point/line/polyline/circle access | `Host::object_geometry`, `Geometry` DTO | Coordinates are local mm; use `ObjectInfo.placement` for world coordinates. Other geometry kinds return an explicit gap. |
| `Objects.AddPoint/AddLine/AddPolyline/AddCircle` | `add_point/add_line/add_polyline/add_circle` or `add_object` | Host validates geometry, current context and publication; returns opaque object ID. |
| `Objects.Delete` | `Host::delete_object` | Host permission and transaction guards apply; do not swallow refusal. |
| Object transformation | `Host::transform_object` | Row-major matrix, column vectors, translation column 3 in mm. Confirm whether the source transform is local or world and whether composition order matches the desired behavior. |
| Undo record calls | One host-owned plugin command transaction | Do not begin a second independent history/Undo store in the plugin. |
| Result/failure/cancel | `Result<Value>`, `SdkError` and propagation | No partial success; verify document and existing Redo preservation. |
| Capability probing | `Host::capabilities` | Only rely on services returned by the running host. |

## Capability gaps requiring explicit work

The following are not initial SDK services: NURBS creation/editing, BRep/solid operations, arbitrary intersections/Boolean/fillet, trim/join, mesh creation, advanced measurement, block creation, layer mutation, persistent plugin storage, events, interactive `GetPoint`/`GetObject`, file import/export, panels, display conduits and custom GPU access.

Some have internal Rust/native implementations listed in the inventory, but exposing them requires a public contract and host validation. Do not call fixture helpers such as `om9_kernel::transform_box`, `curve_point_and_derivative`, `line_face_hits` or `fillet_volume` as if they process arbitrary plugin geometry. Do not fake an unavailable solid operation with a display mesh.

Windows UI frameworks (WinForms/WPF/MFC), Win32/COM calls and Windows-only DLLs need portable replacements. SDK UI services must be added through OM9-owned portable descriptions or explicit host abstractions; sharing `egui::Ui`/`wgpu::Device` across a plugin native ABI is not an accepted shortcut. Python bindings, .NET hosting and a Rhino runtime are not part of this Rust SDK migration path.

## Per-command acceptance note

Record the source command and relevant source/API references; supported options; input geometry and units; host capabilities used; expected outputs and tolerance; permissions; transaction result; Undo/Redo; Error/Cancel behavior; dependencies replaced; target triples actually built; packaged-library hashes; runtime replay results; and remaining uncertainty. Retain original fixtures privately when required and publish only authorized authored fixtures or derived non-private evidence.

A Windows build proves that target only. A package containing separately built Windows/macOS libraries can be one user install file, but each target needs actual build and runtime checks. Check macOS host-native adapters and packaging before claiming a completed cross-platform port.
