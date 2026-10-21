# 3DM V5 import/export completion — P0 contract

Date: 2026-10-20
Status: **P0 COMPLETE — P1 TABLES/ATTRS/PAGE OVERLAY DONE — P2 HATCH DISPLAY PROXY DONE — P3 EXPLICIT OMISSION DONE — TEXT-STYLE/ANNOTATION/LIGHT PENDING**
Owner: `OM9-L2-01-13` (M03), exchange surface.

This document is the disposition contract for the remaining Rhino 5 import/export
gaps. It is deliberately separate from the BRep milestone: BRep editable import
and export are already green (44/44 bind, 30/30 editable, release import 5.09 s,
export 5.23 s). The gaps below are V5 metadata, host classes and retained records.

## 1. Current host surface

| Surface | Host in OM9 today | Export today |
|---|---|---|
| Layer | name, parent, color, visible, selectable, locked, preset slot | name, parent, color, visible, locked |
| Object attributes | layer, color, visible, hidden, selectable, locked | layer, color, visible, locked |
| Geometry | Point, Cloud, Curve, Mesh, Brep, NativeBrep, Instance | same families through current writer |
| Definitions | static instance definitions, mm | static instance definitions, mm |
| Tables | layers, definitions only | layers, definitions only |
| Retained | source archive bytes and native components | omitted with `EXPORT_RETAINED_OMITTED` |

## 2. Disposition matrix

Every native V5 family must be one of:

- **Host** — typed Rust data, editable, exported.
- **Retained** — source bytes preserved, not editable, exported only through an
  explicit retained merge or reported as omitted.
- **Unsupported** — rejected/diagnosed; never silently converted.

| Native family | Import disposition target | Export target | Priority |
|---|---|---|---|
| Render material table | Host `ExchangeMetadata.materials` | Native table + refs | P1 |
| Line pattern table | Host `ExchangeMetadata.line_patterns` | Native table + refs | P1 |
| Group table | Host `ExchangeMetadata.groups` | Native table + membership | P1 |
| Dim style table | Host `ExchangeMetadata.dim_styles` | Native table + annotation refs | P1/P2 |
| Text style table | Host `ExchangeMetadata.text_styles` | Native table + annotation refs | P1/P2 |
| Hatch pattern table | Host `ExchangeMetadata.hatch_patterns` | Native table + hatch refs | P1/P2 |
| Layer material/linetype/plot | Host layer fields | Native layer fields | P1 |
| Object material/linetype/plot/groups/display order/space/viewport | Host attributes | Native attributes | P1 |
| ON_Annotation (text/dimension/leader) | Host `Geometry::Annotation` | Native annotation | P2 |
| ON_Hatch | Host `Geometry::Hatch` | Native hatch | P2 |
| ON_Light (component) | Host `Geometry::Light` | Native light | P2 |
| Object/geometry userdata | Retained source | Retained merge or explicit omitted | P3 |
| History/results | Retained source | Retained merge or explicit omitted | P3 |
| User tables | Retained source | Retained merge or explicit omitted | P3 |
| Retained-only geometry | Retained | Retained merge or explicit omitted | P3 |
| V6/V7/V8/V4 | Out of P0–P3 scope | Deferred phase | P4 |

### P1 group slice implemented

Groups already had a core host type (`Document.groups`), so this slice did not
need an archive bump:

- `ImportBundle.groups` carries host groups; import maps native `ON_Group`
  components and object group indices to `GroupId`/members.
- Export packet includes `document.groups`; the native writer creates the V5
  group table and applies `AddToGroup` indices to object attributes.
- `ImportReceipt.groups` and the UI import summary expose the count.
- Evidence: `om9-app` `three_dm_groups` export→native→import round-trip passes;
  the native artifact was rebuilt and rehashed.

Still pending in P1: render materials, dim/text/hatch pattern tables, and typed
layer/object attributes beyond groups.

### P1 line-pattern slice implemented

- `ExchangeMetadata.line_patterns` is a typed document table of
  `LinePattern { id, name, segments }`; segments carry length and line/space type.
- Native source inventory now emits `ON_Linetype` segments; import maps them.
- Export packet carries `document.exchange.line_patterns`; the native writer
  creates the V5 `ON_Linetype` table.
- Layer and object attributes carry `line_pattern`/`line_pattern_source`; the
  writer sets `ON_Layer::SetLinetypeIndex` and object `m_linetype_index`/
  `SetLinetypeSource`.
- Plot color/weight, plot sources, display order and model-space are typed on
  attributes; layer plot color/weight are typed too. Page-space objects are still
  skipped by the current importer (`attributes.space == 1` gate).
- Evidence: `three_dm_line_patterns` export→native→import round-trip passes for
  the table plus layer/entity refs and plot/display attributes.
- Renderer unchanged; viewport still uses layer/object color and does not apply
  the pattern to display geometry yet.

Still pending: text-style export and full page-layout semantics; page-space objects are hosted as model-space overlays with an explicit diagnostic. A text-style import scaffold exists, but the native V5 writer does not
currently emit the text-style table; export reports that limitation explicitly
rather than claiming success.

### P1 material table implemented

- `ExchangeMetadata.materials` stores `RenderMaterial { id, name, diffuse }`.
- Native source inventory emits `ON_Material` diffuse color.
- Export packet/native writer creates the V5 `ON_Material` table.
- Evidence: `three_dm_line_patterns` now also round-trips a material name and
  diffuse color.
- Layer/object material references are host fields and round-trip; material
  texture/reflection fields remain retained-only.

### P1 dimension-style table implemented

- `ExchangeMetadata.dim_styles` stores `DimStyle { id, name, text_height,
  length_resolution }`.
- Native source inventory already emits text height/length resolution; import
  maps the dim-style table.
- Export packet/native writer creates the V5 `ON_DimStyle` table and sets those
  two fields.
- Evidence: `three_dm_line_patterns` round-trips a dim style name and height.
- Pending: annotation-to-dimstyle references and all other dim-style fields.

### P1 hatch-pattern table implemented

- `ExchangeMetadata.hatch_patterns` stores `HatchPattern { id, name,
  fill_type }`.
- Native source inventory emits `ON_HatchPattern::FillType()`.
- Export packet/native writer creates the V5 `ON_HatchPattern` table.
- Evidence: `three_dm_line_patterns` round-trips hatch-pattern name/fill type.
- Hatch-pattern line definitions (angle/base/offset/dashes) are host fields and
  round-trip when fill type is Lines; hatch geometry/references remain pending.

### P1 page-space overlay implemented

- Native projection no longer marks `Attributes.page_space` unsupported.
- Page-space objects project with model units and are hosted with `space=2`.
- Native diagnostic states that page layout is not modeled; the object is a
  model-space overlay, not a full paper-space layout.
- Evidence: `three_dm_line_patterns` exports and reimports a page-space line.

### P2 Hatch display proxy implemented

- `Geometry::Hatch` carries `display: Mesh` plus loop/pattern metadata; scene
  cache maps it through the existing mesh path.
- Native `geometry_inventory` emits Hatch with a boundary-fan display proxy and
  `#pattern_index`.
- Native export creates `ON_Hatch` via `ON_Hatch::Create` and a default Solid
  hatch pattern table when needed.
- Evidence: `three_dm_hatch` export→native ON_Hatch→import passes; display proxy
  and `#0` survive.
- Exact boundary loops are host polylines and round-trip; display remains a
  boundary-fan proxy, not Rhino pattern fill rendering. Pattern lines are table
  metadata, not per-hatch rendered fill.

### P3 retained export policy implemented (explicit omission tier)

- Current geometry-only 3DM export does not merge retained-native records.
- `ExportReceipt` already counts `dropped_native_records`, history/results/
  resources/retained sources; the UI emits `EXPORT_RETAINED_OMITTED` when records
  were not written.
- Original source bytes remain in the project; the export never silently claims
  that omitted records were preserved.
- This is the explicit-omission tier, not a retained merge. A future retained
  merge remains optional and is not required for this policy.

### P2 Light scaffold (not complete, export blocked)

- `ExchangeMetadata.lights` stores `RenderLight { id, name, style, location,
  direction, diffuse, intensity }`.
- Native import reads `ON_Light` settings into the table.
- Native export does **not** write `ON_Light` components; export emits an explicit
  retained-only diagnostic when lights are present.
- `Geometry::Light` scaffold exists (display point, location, direction, color,
  intensity, style); native import can emit the JSON, but native export of an
  ON_Light entity failed the export correspondence gate and was removed.
- Packet inspection unit test passes, but no native export/import fixture; Light is not a completed host type.

## 3. P1 schema proposal (remaining, not yet implemented)

### Document carrier

Do not add six top-level `Document` fields. Add one additive carrier:

```rust
pub struct ExchangeMetadata {
    pub materials: BTreeMap<MaterialId, RenderMaterial>,
    pub line_patterns: BTreeMap<LinePatternId, LinePattern>,
    pub groups: BTreeMap<GroupId, Group>,
    pub dim_styles: BTreeMap<DimStyleId, DimStyle>,
    pub text_styles: BTreeMap<TextStyleId, TextStyle>,
    pub hatch_patterns: BTreeMap<HatchPatternId, HatchPattern>,
}
```

`Document.exchange: ExchangeMetadata` with serde default and skip-if-empty.
Old `.om9` loads with empty metadata; new `.om9` written by an old binary must
fail explicitly, so the archive schema is bumped or version-gated.

### Attributes and layer

```rust
pub struct Attributes {
    // existing fields
    pub material: Option<MaterialId>,
    pub material_source: MaterialSource,
    pub line_pattern: Option<LinePatternId>,
    pub line_pattern_source: LinePatternSource,
    pub plot_weight_mm: Option<f64>,
    pub plot_color: Option<[u8; 4]>,
    pub display_order: i32,
    pub groups: Vec<GroupId>,
    pub space: Space,
    pub viewport_id: Option<ViewportId>,
}
```

Layer gets material/linetype/plot fields. All new fields are optional/default so
old documents and tests can construct via defaults.

### Native packet

Import DTO already reads the component tables. P1 maps them into
`ExchangeMetadata` and emits counts. Export packet schema becomes 2 with:

```json
{
  "schema_version": 2,
  "mode": "geometry_only_current",
  "document": {
    "...": "existing",
    "exchange": { "materials": [], "line_patterns": [], "groups": [], "...": [] }
  }
}
```

Native writer creates `ON_Material`, `ON_LinePattern`, `ON_Group`, `ON_DimStyle`,
`ON_TextStyle`, `ON_HatchPattern`, and sets layer/object references. Unknown
family or missing reference is a hard diagnostic, not a default.

### Render boundary

P1 does **not** change `RenderBuffers`, shaders or render passes. Materials,
linetypes, groups and plot fields are data/export metadata. The viewport keeps
using layer/object color.

## 4. P2 host class boundary

Host classes carry a render-neutral display proxy:

- `Hatch.display: Mesh` (fill + boundary/pattern lines when available).
- `Annotation.display: Mesh` (box/line proxy); text remains metadata.
- `Light.display: Mesh/Points` (icon); no viewport lighting effect.

Scene maps `display` to existing `RenderBuffers::Mesh/Points`; no renderer edit.
Text/hatch-pattern fidelity is an explicit follow-up, not hidden.

## 5. P3 retained policy

No silent loss. Export receipt must report:

- `preserved_retained` — records merged into output,
- `omitted_retained` — records deliberately not exported,
- `unsupported` — families rejected with a code.

`EXPORT_RETAINED_OMITTED` remains for a known omission; it is not a success claim.

## 6. Acceptance for P1

1. V5 fixture with material, line pattern, group, dim/text/hatch pattern tables
   and layer/object references imports with exact counts.
2. Export/reimport via native `--inspect` preserves table names/IDs and object
   references.
3. `.om9` save/reopen preserves metadata; old documents default empty.
4. No change to `RenderBuffers`, tests still 116/116 app lib.
5. Import/export timing regression ≤20% over 5.09/5.23 s on the BRep fixture.

## 7. Open decisions before P1 code

1. Archive schema bump: `.om9` schema 5 vs version-gated additive field.
2. Group display/UI: groups are selection sets; no viewport render change.
3. Material/linetype editing scope: property display only in P1, full render use
   is P5.
4. Retained merge scope in P3: raw native record merge vs explicit omission if
   merge is not exact.

## 8. Current evidence

- BRep editable E2E green: 30/30 editable, 29 root + 1 member, export 44 objects.
- App lib 116/116; four-view and face/edge pick smoke pass.
- Release import 5.09 s / export 5.23 s; default `occt-brep`.
- Public audit git-public scope 0 errors; `--strict` private material remains.















