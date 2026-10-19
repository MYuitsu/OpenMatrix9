# Standalone 3DM implementation matrix

This records the currently tested component slice, not an enabled application capability or acceptance of complete RCORE-10. Evidence: [native conversion](2026-10-13-standalone-native-conversion.json), [strict source inventory](2026-10-13-standalone-native-source-inventory.json), [owned Rust export](2026-10-13-standalone-exchange.json), [full foundation progress](2026-10-13-standalone-foundation-progress.md).

Cold inventory reads actual on-disk native records. Tests compare geometry and metadata against immutable current OM9 sources. The app has no 3DM import/export dialog or registered native operation yet. “Current export” below means the owned library operation; Rhino 5 actual open/edit remains untested.

Schema-2 source inventory independently reads registered native tables, original units/tolerances, UUID/index references and unknown-userdata descriptors. A strict low-level scan rejects skipped unknown classes/tables, damaged CRCs, native recovery warnings and incomplete endmarks. SDK-decoded indices are explicitly marked; original source bytes remain authoritative. `geometry_projection_exact` certifies only the emitted geometry fields. Every object requires host admission; omitted attributes and transitive definition/unit requirements remain explicit. No source DTO claims `editable_exact` or `host_ready`.

| Type | Read/inventory evidence | OM9 display/edit | Retain original source | Current export evidence |
|---|---|---|---|---|
| Point | Actual ON_Point coordinates, ID and supported attributes | Owned scene point; Point session | Pending | Placement baked once, current edit/delete/new validated |
| Line | Actual ON_LineCurve endpoints/domain | Owned curve scene; Line session | Pending | Native line, arbitrary increasing domain preserved |
| Circle | Actual full ON_ArcCurve plane/radius/native domain/angular interval; independent cold D0–D3 oracle | Owned curve scene, exact native sample/pick correspondence; unresolved f64 parameter precision disclosed; Circle editor pending | Pending | Uniform/mirrored placement with phase and affine angle/parameter mapping; exact full angular turn required; near-full/partial arcs and direct nonuniform circle rejected |
| NURBS curve | Degree, Euclidean poles, rational weights, expanded knots, native domain/periodicity | Nonperiodic sampled display/evaluator; periodic evaluation/editor pending | Pending | Rational/periodic structure tested; reversed/seam, unmatched periodicity, trimmed domain and noncanonical outer knots rejected |
| Polycurve | Actual native segment masters/domains/order | Owned curve scene; composite editor pending | Pending | Contiguous accepted segments preserved; unsupported domain/orientation rejected |
| NURBS surface | Actual U-major control grid, degrees, weights, knots, domains/periodicity | Owned master; surface scene/editor pending | Pending | Rational and genuine periodic native fixtures; trims/reverse unsupported |
| BRep/trimmed face/shell/solid | Actual native BRep identified as retained-only, including a definition member and transitive dependent instances; complete topology fields pending | Typed topology/selected operation adapter pending | Pending | Pending; no CAD-to-mesh substitution |
| Extrusion | Complete type/field matrix pending | Pending | Pending | Pending |
| Mesh | Actual V5 double vertices, native triangle/quad indices, normals/colors | Owned mesh scene; dedicated editor pending | Pending | Original indices and transformed vertices; normalized inverse-transpose normals with native float tolerance |
| Cloud | Actual ordered native point rows, normals/colors | Owned native-index cloud scene; dedicated editor pending | Pending | Point row order/placement/colors; hidden=true refused, all-false runtime hidden channel omission disclosed |
| Shared/nested instance | Actual definition IDs, member IDs, native instance transforms/units | Shared owned scene buffers/occurrence picking; BlockEdit pending | Pending | Structure retained within new output, mirrored/nonuniform transforms not flattened; linked/external behavior pending |
| Layer | Tested IDs, order/index, parent, name, color, visible/locked | Owned permission model; UI controls/semantic slots pending | Pending | Tested fields preserved; independent selectable=false rejected; unassigned objects map to disclosed native Default |
| Annotation/page/detail | Actual native Text with nondefault DimStyle UUID and text-height fixture; original page-unit settings; complete annotation/page/detail field matrix pending | Pending | Pending | Pending |
| Material/light/render tables | Registered component keys/kinds/relations inventoried; actual material, linetype, group, DimStyle and hatch fixtures; complete field matrix pending | Pending | Pending | Pending |
| Unknown userdata/History | Native userdata descriptors and anonymous user-table inventory; unknown-class/table completeness rejection; opaque payload/remap cold preservation pending | Native History replay/plugins not executed | Pending | Geometry-only omission disclosed; preservation writer pending |

| Field | Tested policy/evidence | Open limits |
|---|---|---|
| Model/page units | Current canonical geometry in millimeters; independent source mm/inch/custom/None/Unset model/page unit/tolerance inventories, per-definition unit dependency readiness | Rust native-to-host conversion and explicit unknown-unit resolution policy pending |
| UUID/name | Current entity/definition/layer IDs and Unicode names cold-compared; duplicate/nil/noncanonical/name-normalization rejection | Each future import needs a fresh host namespace and durable source mapping |
| Placement/parameters | Ordinary geometry baked once; instance matrix remains native; domains/knots/weights checked against current masters; mapped circle phase and native/fraction D0–D3 independently tested | General trim/topology/subelement maps pending; mapped circle display explicitly withholds unresolved native f64 sample bindings |
| Visibility/lock/selectable | Independent supported visibility or lock verified; combined hidden+locked ordinary objects, locked definition members and selectable=false explicitly refused | Full native/OM9 field mapping, selection semantics and preservation metadata pending |
| Color | Native object/layer color source and RGBA checked | Parent material/render semantics, all color-management fields pending |
| Parent/dependency | Layer graph, shared/nested definitions and exact ordered members; missing/cycle/longest-path depth rejected | Retained external/deleted dependency remap pending |
| File lifecycle | Exclusive private native output, CRC/corrupt read rejection, sync/hash/cold validation, source/Cancel/hash revalidation, same-volume replacement; good destination survives rejection | Disk-full fault injection, durable directory flushing and full resource/retained merge policy pending |
| Budget/lifetime | Bounded input/DOM/geometry/native commit/stdout/stderr/deadline; suspended Job assignment/reap/reader join; source/retained coordinator/packet reservation before launch | Concurrent global reservations, physical RAM measurement and native memory-safety certification not accepted |

Preservation export must merge retained/edited/deleted/new state with source tombstones and resource/table remapping. No snapshot passthrough or the above current geometry conversion substitutes for that writer. All RCORE-00.M01–M12 and complete RCORE-10 capability/fixture gates remain open.
