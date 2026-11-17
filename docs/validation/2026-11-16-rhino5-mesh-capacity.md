# Rhino 5 x64 mesh capacity: bounded decompile investigation

Checked 2026-11-16 at the user's request. OM9-authored derived analysis only;
no recovered source or disassembly is reproduced. Private inputs are unchanged
and resolve through the ignored reference-location registry.

**No two-million guard was identified in the inspected vertex insertion path.**
Signed 32-bit counts are only an indexing bound. The inspected existing-storage
reallocation wrapper also narrows its byte request to unsigned 32-bit. Neither
finding establishes a usable whole-scene or per-mesh maximum, or an FPS guarantee.

## Input identity

Installed Rhino 5 x64 binaries were checked against bounded Ghidra procedures
using PE exports/imports, image bases and native instructions. RhinoCommon
metadata: AssemblyVersion 5.1.30000.17, AssemblyFileVersion 5.14.00522.08390.
Rhino.exe version text includes 22-May-2017. Findings do not establish behavior
for every Rhino 5 service release or later Rhino versions.

| Logical input | SHA-256 |
|---|---|
| Installed openNURBSx64.dll | `a146c92c590c8a942929fa05bab5807262fb418e105dd71bfedfb50b7c43a96d` |
| Installed rhcommon_c.dll | `23012666240edffe65c967c097e8b6cd43bee1208ba40a6b99dc1f16880aecec` |
| Installed Rhino.exe | `8daacaa0a9a7ade3c82793309fdff421306b736fa57bf1342b01369d0f14089b` |
| Private openNURBSx64.dll.c | `a3eb70b72cb2c261aa3af487eab9f3fcf236a6e0dfa1c8fce84f5ad718642003` |
| Private rhcommon_c.dll.c | `f7829de54216fbe11a02e8207349436f5ff62f9265f558865ca6111bd3397e18` |
| Private Rhino.exe.c | `7a4a9d4fe3cdb94b9c42b71b3ed8468bd7fdbbe2d49da5f78063d72ef5b7711e` |

DLL image base: `0x180000000`; Rhino.exe: `0x140000000`. Addresses below are
VAs; definition lines refer to these hashed exports. This research informs
OM9-L3-01-06-003 and M1-R capacity planning, not implementation completion.

## Inspected managed-to-native path

Private `RhinoCommon/Geometry/Collections/MeshVertexList.cs` lines 27–40 expose
an int Count; lines 101–106 Add at the current Count through SetVertex.
Count setter rejects negative input; no two-million maximum was identified.
MeshFaceList.Count also uses an integer.

| Procedure | VA | Definition / selected evidence | Derived behavior |
|---|---|---|---|
| rhcommon_c ON_Mesh_GetInt | `0x18002A740` | line 37772, vertex/face switch | Exposes native counts. |
| rhcommon_c ON_Mesh_SetInt | `0x18002A5F0` | line 37712, vertex/face cases | Reserves/sets array count; no identified 2M check in these cases. |
| rhcommon_c ON_Mesh_SetVertex | `0x180029CA0` | line 37218 | Delegates float coordinates to ON_Mesh. |
| ON_Mesh::SetVertex(int, ON_3fPoint const&) | `0x1801751D0` | line 292573; signed checks and append call at `0x180175332` | Replaces an existing vertex or appends at Count. |
| Point-array Append | `0x18026BDA0` | line 475041; native through `0x18026BEB5` | Signed 32-bit count/capacity, 12-byte stride, dynamic growth. COMDAT aliases are not different implementations. |
| Point-array SetCapacity | `0x1800B8230` | line 146893; native through `0x1800B82D9` | Positive signed capacity, virtual reallocation and allocation-failure handling. |
| Point-array Realloc | `0x1800B8050` | line 146766; native through `0x1800B8061` | Computes 64-bit capacity times 12, then delegates. |
| onrealloc_from_pool | `0x1801F38B0` | line 386141; native through `0x1801F399B` | Zero frees; null old pointer uses onmalloc_from_pool. Existing-storage branch narrows bytes before helper `0x1802F5380`. |
| onmalloc_from_pool | `0x1801F36D0` | line 386042, bounded body | Decompile forwards bytes to helper `0x1802F4040`; heap backend not exhaustively validated. |

Named native entry mapping: EXACT/A where PE/address identity was checked.
Managed entry: wrapper. Behavioral confidence: High for signed counts, stride,
growth arithmetic and existing-storage byte narrowing (native corroboration).
Overall usable capacity: unestablished. The downstream allocator helper was
read as necessary context, not exhaustively proved.

## Meaning of the boundaries

- Signed 32-bit positive counts represent at most **2,147,483,647 elements per
  array**. This is not proof that a mesh at that size works; topology,
  attributes, arithmetic, heap and GPU constraints can intervene earlier.
- Append changes growth policy above `0x10000000` current bytes (256 MiB).
  Its increment is then at most `0x155555d` points: 22,369,629 elements or
  268,435,548 bytes. **256 MiB is a growth-policy threshold, not a total cap.**
- Existing-storage reallocation narrows the byte request to 32-bit. Keeping
  requested capacity times 12 below 2^32 requires capacity at most
  357,913,941. This is a **derived non-wrapping arithmetic condition**, not a
  supported vertex maximum: growth reserves extra capacity, branches differ,
  and no boundary-size runtime test was performed. Crossing 4 GiB can wrap
  on this path rather than produce a documented clean refusal. Do not call
  this a safe product-wide memory cap or claim a 357M mesh works.

## Partition request is not a scene limit

ON_Mesh::CreatePartition(int,int), VA `0x180182280`, definition 302772,
compares total counts with supplied bounds and partitions larger meshes
(notably lines 302863–302885).

An unnamed Rhino.exe caller `FUN_1407528F0`, definition 1253763, requests
64,000 vertices and 64,000 triangles per part. Native call `0x140752C94`
resolves via IAT `0x140EDF6A8` to the CreatePartition import. Data RVA
`0x15A26EC` contains 64,000. Decompile context references `commands\\cmdDaleL.cpp`;
the public command name and relationship to viewport rendering are unproved.
Thus **64,000 is a command-local partition request**, not a global mesh/scene
limit or proved universal display batch size.

## Remaining uncertainty and OM9 consequence

A bounded textual search for numeric spellings of 2,000,000 in the three
native exports found no match. Constants may reside in data or have other
spellings, so this alone cannot prove absence of a global guard. The inspected
insertion/capacity path is stronger evidence within its stated scope.

No Rhino runtime maximum, native CAD control-point maximum, whole-document/
Undo maximum or sustainable-FPS vertex count was measured. NURBS control
points, per-mesh vertices and total cached viewport vertices are different
quantities. Many independent spheres can hold many total vertices while
remaining below every individual array boundary.

The evidence does not justify OM9's 2M whole-scene point guard as a Rhino 5
compatibility ceiling. Separate parser/source guards, per-buffer/index ranges
and RAM/VRAM admission before changing a product allowance. The completed OM9
4M stress fixtures are separate evidence; this research changes no defaults.

Next bounded investigation: actual Rhino display-buffer batching/upload and
index formats, followed by controlled runtime capacity sweeps if requested.
A universal practical ceiling remains unknown.
