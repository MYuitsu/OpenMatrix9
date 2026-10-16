# M03 L4 authoring conventions (om9.l4.v1)

Authority: this file is the single source of truth for every L4 written under
`OM9_Milestones/M03 - Curve, Surface, Solid and 3DM Exchange/`. `validate_l4.py` (same folder)
enforces it mechanically and reads the tolerance glossary from section 12.

Scope of the whole task: SPEC ONLY. Nothing here writes Rust/C++/Python product code. Authors only
ADD `L4-NNN - Name/l4.json`, optional `L4.md`, and one `L4_INDEX.md` per L3. Never modify or delete
`L1.json`, `L2.json`, `l3.json`, `reference\**`, never `git add` / `git commit`, never touch the
private `reference\` folders in any other location.

---------------------------------------------------------------------------------------------------

## 0. Verified facts about the repository (read before authoring)

Checked on this machine while writing the convention (commands and counts are real):

| Fact | Value |
|---|---|
| M03 root | `H:\OpenMatrix9\OM9_Milestones\M03 - Curve, Surface, Solid and 3DM Exchange` (74 chars) |
| Hierarchy | 1 L1 (`L1.json`, `l4_count: 0`), 14 L2 (`L2.json`), 243 L3 (`l3.json`, `schema_version: om9.l3.v1`) |
| L3 roles | `backend_framework`/tag `be` = 36, `interactive_feature`/`febe` = 42, `ui_command_adapter`/`febe` = 165 |
| Existing L4 in M03 | NONE. `Get-ChildItem -Recurse -Directory -Filter 'L4*'` and `-File l4*` both return 0 under M03 |
| Existing L4 elsewhere | M00 = 73, M01 = 516, M02 = 306 `l4.json` (other schema, see 0.1). M04-M09, ZAFTER = 0 |
| Pre-existing M03 files | 2960 files (fingerprinted in `validate_l4.py`, `EXPECTED_DIGESTS`) |
| `OpenMatrix9_Codex_Spec_v2` | DOES NOT EXIST at `H:\OpenMatrix9\OpenMatrix9_Codex_Spec_v2` (AGENTS.md names it, the folder is absent). Do not cite `specs/02-curve`. Use the L3 `reference\` text, the legacy Rust in `rust/src`, and QymCAD samples instead. |
| OCCT on this machine | No `OCCT_*` env vars, no OCCT install except inside `C:\Program Files\FreeCAD 1.1`. Native OCCT tests therefore cannot be assumed runnable: see 10.3. |
| Rust toolchain | `cargo 1.98.1`, workspace edition 2024, `rust-version` 1.88 (`om9-app` 1.95), Python 3.13 |
| Test command verified | `cargo test --locked --manifest-path standalone/Cargo.toml -p om9-core circle_native_parameter_mapping` -> `1 passed`, exit 0, about 10 s (run from `H:\OpenMatrix9`) |

### 0.1 Difference from the existing M00/M01/M02 L4 files

M00-M02 L4 use the older "process step" schema (`task{}`, `implementation_contract{}`, `agent_runbook_markdown`,
folder `L4-NN - Source Audit|API Contract|Implementation|Integration|Regression|Acceptance`, id suffix 2 digits).
M03 deliberately uses a richer, per-capability schema (`om9.l4.v1`) with 3-digit folders as directed by the
planning task, because each M03 L4 must be implementable by a small model from the file alone.
The id scheme stays compatible: `OM9-L4-<L2 digits>-<L3 number>-<NN>` (M02 example `OM9-L4-01-07-001-03`).
L2 ids are unique across all milestones (102 L2 checked, 0 duplicates), so M03 L4 ids never collide with M00-M02.

---------------------------------------------------------------------------------------------------

## 1. Language and ownership rules (from AGENTS.md and `skills/openmatrix9-workflow/references/rust-first.md`)

1. Rust is the first choice. Safe Rust owns model, validation, algorithms, command sessions, undo/transactions, scene, tests.
2. C++ only for a necessary OCCT or openNURBS adapter (C ABI shim). An L4 containing C++ files must have
   `owner_layer` = `occt-adapter` or `opennurbs-adapter` and a `language_note` that states the real constraint.
3. Python only for tooling/fixtures with a real constraint (`owner_layer` = `fixture-tool`, `language_note` required).
4. Every adapter L4 must keep `unsafe` inside a named `ffi*.rs` file, no OCCT handle crossing the C ABI unless an
   earlier L4 defined its lifecycle, no panic/exception crossing the ABI, allocation and free on the same side.
5. QymCAD (`H:\OpenMatrix9\QymCAD`) is a SAMPLE source only. OM9 never build-depends on it. L4 lists QymCAD files under
   `references.qymcad` (repo-relative `QymCAD/crates/...`). `files.*` and `reuse_symbols` must never point into `QymCAD/`.
   When an L4 adapts a QymCAD routine it states so in `references.qymcad`; `QymCAD/THIRD-PARTY-NOTICES.md` and `QymCAD/LICENSE` apply.
6. New code goes to the standalone workspace `standalone/`. The legacy crate `rust/` (`openmatrix9_rust`, FreeCAD host era)
   and `Gui/*.cpp` are PORT SOURCES only (`references.legacy_om9`); AGENTS.md requires migration, not extension.
7. Unsupported commands/options stay VISIBLE and DISABLED with a reason (standalone README). Any L4 with prompts lists
   disabled options with `enabled:false` plus `reason`.
8. Business behavior comes from the L3 `reference\` manual text (Matrix 8/9, Rhino 5) and the legacy Rust prompts. An
   L4 never invents a Matrix default; when the manual is silent it records an `assumptions` entry that picks the simplest
   option.

---------------------------------------------------------------------------------------------------

## 2. Folder and file naming

```
<M03 root>\L2-xx-xx - <name>\L3-NNN - <title>\
    l3.json                       (existing, never edited)
    reference\                    (existing, never edited)
    L4_INDEX.md                   (NEW, exactly one per L3)
    L4-001 - <Short Task Name>\
        l4.json                   (NEW, required)
        L4.md                     (NEW, optional human runbook)
    L4-002 - <Short Task Name>\
        ...
```

Rules (all enforced by the validator):

* Folder: `L4-NNN - <Short Task Name>`; NNN is 3 digits zero-padded, starting at `001`, contiguous (no gaps, no repeats).
* Short name: ASCII only, letters/digits/space/`_`/`+`/`-`, 2 to 27 characters, Title Case words, single spaces, no trailing space, no dot.
  Vietnamese goes in `notes_vi`, never in a folder name.
* Only `l4.json` and `L4.md` may exist in an L4 folder. Only `L4-*` folders and `L4_INDEX.md` may be added to an L3 folder.
  Nothing named `L4*` may be added to an L2 folder or the M03 root.
* JSON is UTF-8 without BOM, 2-space indent, no duplicate keys, no `NaN`/`Infinity`.

### 2.1 Path-length budget

Windows limit used by the project: every path must be **< 240 characters**. Measured:

* The longest EXISTING path in M03 is 239 characters (`reference\text\...` files). Existing L3 folders are up to 192 characters.
* New file `.../L3-xxx/L4-NNN - Name/l4.json` adds `1 + len(folder) + 1 + 7` characters to the L3 folder path.
* Worst L3 folder (`L2-04-01 ... \L3-016 - Transform · Orient Perpendicular to Curve - menu binding`) = 192 characters.
  Therefore `192 + 1 + len(folder) + 1 + 7 < 240` requires `len(folder) <= 38`.
* Convention: `len(folder) <= 36` (`"L4-NNN - "` = 9 characters, so the short name is **<= 27 characters**). That leaves 2 characters spare in the worst case.

Maximum folder-name length that fits per L2 (`239 - longest L3 folder - 9`), for information:

| L2 | longest L3 folder | max L4 folder name |
|---|---|---|
| L2-01-09 | 162 | 68 |
| L2-01-10 | 167 | 63 |
| L2-01-11 | 170 | 60 |
| L2-01-13 | 161 | 69 |
| L2-02-04 | 168 | 62 |
| L2-02-05 | 171 | 59 |
| L2-02-06 | 172 | 58 |
| L2-03-08 | 174 | 56 |
| L2-03-16 | 182 | 48 |
| L2-03-17 | 184 | 46 |
| L2-03-18 | 176 | 54 |
| L2-03-19 | 173 | 57 |
| L2-03-20 | 177 | 53 |
| L2-04-01 | 192 | 38 |

The 36-character universal cap is used everywhere so authors never need this table.
**Rule: before finishing a branch run the validator; its `pathlen` check computes, for every added file,
`max(actual absolute length, length under the canonical H:\ checkout)` and fails at 240 or more.**
`L4_INDEX.md` lives in the L3 folder and is always shorter than `l4.json`.

---------------------------------------------------------------------------------------------------

## 3. Id scheme

| Level | Pattern | Example |
|---|---|---|
| L2 | `OM9-L2-<LL>-<NN>` | `OM9-L2-01-09` |
| L3 | `OM9-L3-<LL>-<NN>-<NNN>` | `OM9-L3-01-09-001` |
| **L4** | `OM9-L4-<LL>-<NN>-<NNN>-<NN>` = L3 id with `L3` replaced by `L4` plus `-<seq 2 digits>` | `OM9-L4-01-09-001-03` |

* `seq` (integer) = folder number `L4-003` = id suffix `03`. Maximum 99 L4 per L3 (in practice 3-8).
* `parent_l3_id` and `parent_l2_id` must equal the values in the parent `l3.json`.
* L4 ids are the only keys other files may use to depend on an L4.
* Cross-L3 dependencies are allowed (DAG over the whole M03 tree). Cross-milestone dependencies are NOT L4 dependencies:
  express them in `dependencies.external_contracts` by L2 id.

---------------------------------------------------------------------------------------------------

## 4. l4.json schema (`om9.l4.v1`)

Unknown keys are rejected. Types: S string (non-empty), I integer, B boolean, L[x] list, O object, `S|null` nullable.

### 4.1 Top level

| Field | Type | Rule |
|---|---|---|
| `schema_version` | S | exactly `om9.l4.v1` |
| `level` | S | `L4` |
| `id` | S | pattern in section 3 |
| `milestone` | S | `M03` |
| `parent_l3_id` | S | existing L3 id |
| `parent_l2_id` | S | L2 id from the parent `l3.json` |
| `seq` | I | 1..99, equals folder number and id suffix |
| `title` | S | <= 80 chars, one line, English (the folder name is its short form) |
| `planning_state` | S | exactly `SPEC_ONLY_NOT_IMPLEMENTED` |
| `tag` | S | `be` or `febe`, equals `specification.tag` of the parent L3 |
| `owner_layer` | S | `rust-domain`, `occt-adapter`, `opennurbs-adapter`, `command-session`, `ui-binding`, `fixture-tool` |
| `language` | S | primary language `rust`, `cpp`, `python` |
| `language_note` | S | required when `language` != rust or any file is cpp/python: the real constraint |
| `kind` | S | `contract`, `algorithm`, `adapter`, `command-session`, `ui-binding`, `fixture`, `integration-test` |
| `size` | O | `{ "impl_loc": I, "test_loc": I }`; `impl_loc <= 250`, `impl_loc + test_loc <= 450` |
| `goal` | S | exactly ONE sentence ending in `.` (no second sentence inside) |
| `scope` | O | section 4.2 |
| `dependencies` | O | section 4.3 |
| `files` | O | section 4.4 |
| `api` | O | section 4.5 |
| `algorithm` | O | section 4.6 |
| `interaction` | O or `null` | section 4.7; `null` requires `interaction_na_reason` |
| `interaction_na_reason` | S | only when `interaction` is `null` |
| `tests` | O | section 4.8 |
| `definition_of_done` | L[S] | >= 4 items, each >= 10 chars, one runs the test command |
| `references` | O | section 4.9 |
| `assumptions` | L[O] | section 4.10 (may be empty) |
| `notes_vi` | S | optional Vietnamese reading note, never normative |

### 4.2 `scope` (content standard item 2)

```
"scope": {
  "inputs":           L[S] >= 1   exact input types/units
  "outputs":          L[S] >= 1   exact outputs and observable effects
  "not_in_this_task": L[S] >= 1   explicit exclusions, each naming the L4/L3 that owns it when one exists
}
```

### 4.3 `dependencies` (item 3)

```
"dependencies": {
  "depends_on_l4": L[id]                       // DAG; may be []; ids must exist; no self, no cycles; M03 ids only
  "external_contracts": L[{                    // other L2/milestone services this task relies on
      "l2_id": "OM9-L2-xx-xx",                 // must exist in OM9_Milestones
      "contract": S,                           // what is relied on, in one sentence
      "status": "existing" | "planned" }]
  "reuse_symbols": L[{                         // symbols to reuse (or sibling symbols to call)
      "path": S,                               // repo-relative, '/' separators, not under QymCAD/
      "symbol": S,                             // e.g. "Transform::compose"
      "kind": "fn|method|struct|enum|trait|const|mod|type|cpp-fn|cpp-class",
      "status": "existing" | "planned",
      "from_l4": S | null,                     // planned: producing L4 id, which must be in depends_on_l4
      "signature": S | null,                   // planned: exact Rust/C signature text (required)
      "use": S }]                              // how this task uses it
  "no_reuse_reason": S                         // required only when reuse_symbols is []
}
```

`existing` symbols are checked: the file must exist and the last identifier of `symbol` must occur in it.

### 4.4 `files` (item 4)

```
"files": {
  "create": L[{ "path": S, "language": "rust|cpp|python|toml|json|cmake|markdown", "purpose": S }],
  "modify": L[{ "path": S, "language": ..., "change": S, "from_l4": S | null }]
}
```

* Paths are repo-relative (`standalone/crates/...`), never under `QymCAD/` or `OM9_Milestones/`.
* `modify` of a file that exists today uses `from_l4: null`. `modify` of a file that a sibling L4 creates sets `from_l4` to that
  L4 id (must be in `depends_on_l4` and list the path in its `files.create`).
* `tests.file` and every `api.signatures[].path` must appear in `create` or `modify`.
* A new workspace crate is created by the FIRST L4 that needs it and must list `standalone/Cargo.toml` (`members`) in `modify`
  and say that `cargo build --manifest-path standalone/Cargo.toml -p <crate>` (without `--locked`) is run once to extend `Cargo.lock`.

### 4.5 `api` (item 4)

```
"api": {
  "signatures":      L[{ "path": S, "code": S }]            // >= 1; exact Rust (or C ABI) text; multi-line with \n
  "data_structures": L[{ "name": S, "path": S,
                         "fields": L[{ "name": S, "type": S, "unit": S, "constraint": S }] }]   // may be []
  "error_variants":  L[{ "enum": S, "variant": S, "display": S, "when": S }]                      // may be []
  "invariants":      L[S] >= 1
}
```

Units are `mm`, `rad`, `deg`, `mm3`, `mm2`, `none`, `bytes`, `index`, `fraction`. `display` is the exact `Display` text.

### 4.6 `algorithm` (item 5)

```
"algorithm": {
  "tolerances":       L[{ "name": UPPER_SNAKE, "value": S, "unit": S, "local": B (optional) }]
  "parameter_ranges": L[{ "name": S, "range": S, "unit": S }]
  "steps":            L[{ "n": I, "action": S, "formula": S|null, "on_failure": S|null }]   // >= 3, n = 1..N
  "edge_cases":       L[{ "case": S, "behavior": S }]                                        // >= 2
  "failures":         L[{ "condition": S, "error": S, "message": S, "state_after": S }]      // >= 1
}
```

* Tolerance names come from the glossary (section 12). A name in the glossary must carry the glossary value and unit.
  A name not in the glossary needs `"local": true` plus an `assumptions` entry that justifies it.
* `value` is a number written as text (`"1e-7"`, `"0.001"`).
* Every step is one action a small model can type: name the function, the formula, the order of checks.
* `failures.error` is the exact Rust expression/variant; `message` is the exact user-visible or `Display` text.

### 4.7 `interaction` (item 6, null when not applicable)

```
"interaction": {
  "feature_id":   "OM9-SOLID-012",              // id in standalone/crates/om9-app/assets/features.json
  "command":      S,                            // original Matrix9 command name, unchanged
  "entry_points": L[S] >= 1,                    // e.g. "menu: Solid > Box", "cmd: Box"
  "prompts": L[{ "id": S, "text": S, "accepts": S,
                 "options": L[{ "name": S, "alias": S|null, "default": S|null, "enabled": B, "reason": S|null }],
                 "on_enter": S, "on_esc": S, "on_undo": S }],   // >= 1; text is the exact prompt string
  "defaults": L[{ "name": S, "value": S }],
  "keys": { "enter": S, "esc": S, "undo": S },
  "preview": S | null,
  "source_note": S                              // where the strings/behavior come from (manual page, legacy fn)
}
```

* Menu/mouse invocation and the CMD frame must reach the SAME session (AGENTS.md). Say so in `entry_points`.
* A disabled option needs `enabled:false` and a `reason`.
* Every `febe` L3 needs at least one L4 with an `interaction` object. `be` L3 L4s normally use `null` with a reason.

### 4.8 `tests` (item 7)

```
"tests": {
  "file":           S,                          // repo path; listed in files.create/modify
  "command":        S,                          // exact portable command, section 10
  "native_command": S | null,                   // ignored native tests, ends with "-- --ignored"
  "cases": L[{ "name": snake_case, "kind": "positive|negative|edge",
               "runs_in": "portable" | "native-ignored", "input": S, "expected": S }],
  "fixtures": L[S]                              // fixture files/ids used, may be []
}
```

* >= 3 `portable` cases, at least one `portable` with `kind: negative`. Native OCCT/openNURBS cases are additional.
* Every case `name` is the real test function name and contains the filter (last token of `command`), so one command runs all of them.
* `input` / `expected` carry explicit numbers, strings, enum values, counts. "works", "correct" are not acceptable.

### 4.9 `references` (item 9)

```
"references": {
  "text":       L[S],    // "../reference/text/<file>.md" (L3 reference) or "../../reference/text/<file>.md" (L2 shared reference)
  "images":     L[S],    // "../reference/images/<file>.png"
  "qymcad":     L[S],    // "QymCAD/crates/<crate>/<path>" repo-relative, sample only
  "legacy_om9": L[S],    // "rust/src/solid.rs", "Gui/CoreSolidResult.cpp": port sources
  "behavior_notes": L[S],// what the manual says that this task implements
  "no_reference_reason": S | null   // required when text and images are both empty (13 M03 L3 have no reference text)
}
```

All links are resolved from the L4 folder by the validator and must stay inside the M03 tree. Rows marked `link_to_L2` in a
`REFERENCE.md` live in `../../reference/`. The manual text was extracted from two-column PDFs, so neighbouring sections
(for example Pipe options inside the Box file) interleave; use the section headed with the command name, and quote the
page range in `behavior_notes`.

### 4.10 `assumptions` (item 10, unknowns)

```
"assumptions": L[{ "id": "A1", "statement": S, "chosen": S }]
```

Unknowns are never left open: state the question in `statement` and the simplest decision in `chosen`.
No `TODO`, `TBD`, `FIXME`, `placeholder`, `lorem ipsum`, `to be determined`, `???`, `todo!()` or `unimplemented!()` anywhere
in any value; the validator rejects them (also in `L4.md` and `L4_INDEX.md`).

---------------------------------------------------------------------------------------------------

## 5. Content standard -> schema map

| # | Content standard item | Fields |
|---|---|---|
| 1 | id, title, parent L3, milestone, owner layer, language | `id`, `title`, `parent_l3_id`, `parent_l2_id`, `milestone`, `tag`, `owner_layer`, `language`, `language_note` |
| 2 | one-sentence goal, in/out scope, NOT list | `goal`, `scope.inputs`, `scope.outputs`, `scope.not_in_this_task` |
| 3 | preconditions/deps | `dependencies.depends_on_l4`, `external_contracts`, `reuse_symbols` |
| 4 | files, signatures, structures, invariants | `files`, `api.signatures`, `api.data_structures`, `api.error_variants`, `api.invariants` |
| 5 | numbered algorithm | `algorithm.*` |
| 6 | command-line / UI behavior | `interaction` |
| 7 | acceptance tests | `tests` |
| 8 | definition of done | `definition_of_done` |
| 9 | reference pointers | `references` |
| 10 | size, unknowns | `size`, `assumptions` |

---------------------------------------------------------------------------------------------------

## 6. Example 1 - backend / OCCT L4 (L3 `OM9-L3-01-09-001`, tag `be`)

Planned L4 split of that L3 (see 8.1): `001 Primitive Spec And Errors`, `002 Analytic Mass Oracle`, `003 OCCT Crate And ABI`,
`004 OCCT Box Sphere Builders` (this example), `005 OCCT Cylinder Cone Torus`, `006 OCCT Pyramid Tube Ellipsoid`.
The example is checked by `python validate_l4.py <root> --check-examples CONVENTIONS.md`.

```json l4-example folder="L4-004 - OCCT Box Sphere Builders"
{
  "schema_version": "om9.l4.v1",
  "level": "L4",
  "id": "OM9-L4-01-09-001-04",
  "milestone": "M03",
  "parent_l3_id": "OM9-L3-01-09-001",
  "parent_l2_id": "OM9-L2-01-09",
  "seq": 4,
  "title": "OCCT Box And Sphere Builders",
  "planning_state": "SPEC_ONLY_NOT_IMPLEMENTED",
  "tag": "be",
  "owner_layer": "occt-adapter",
  "language": "rust",
  "language_note": "OCCT exposes only a C++ API, so the two BRepPrimAPI calls live in standalone/crates/om9-occt/native/om9_occt_prim.cpp behind a C ABI; every other file is safe Rust and unsafe is confined to ffi_prim.rs.",
  "kind": "adapter",
  "size": { "impl_loc": 215, "test_loc": 190 },
  "goal": "Add build_primitive to the om9-occt crate so a validated PrimitiveSpec::Box or PrimitiveSpec::Sphere becomes an exact OCCT solid and only an owned BrepSummary (counts, volume, area, validity) returns to Rust.",
  "scope": {
    "inputs": [
      "PrimitiveSpec::Box { origin: Point3 (mm), axes: [Point3; 3] (right-handed unit vectors), size: [f64; 3] (mm, each > 0) }",
      "PrimitiveSpec::Sphere { center: Point3 (mm), radius: f64 (mm, > 0) }",
      "tolerance_mm: f64, the document absolute tolerance (default DOC_ABS_TOL_MM = 0.001)"
    ],
    "outputs": [
      "Ok(BrepSummary { face_count, edge_count, vertex_count, volume_mm3, area_mm2, is_valid, is_closed }) with no OCCT handle retained",
      "Err(KernelError) for invalid input, missing OCCT, OCCT failure or invalid result, leaving no allocation behind"
    ],
    "not_in_this_task": [
      "Cylinder, cone, torus, pyramid, tube and ellipsoid builders (OM9-L4-01-09-001-05 and OM9-L4-01-09-001-06)",
      "Defining PrimitiveSpec, KernelError, BrepSummary, RawSummary or the crate skeleton (OM9-L4-01-09-001-01 and OM9-L4-01-09-001-03)",
      "Storing the shape in the document, History or Undo (OM9-L2-01-05 transaction path)",
      "Tessellation or meshing of the solid",
      "Matrix Box and Sphere prompts (OM9-L3-03-18-012 and OM9-L3-03-18-013)"
    ]
  },
  "dependencies": {
    "depends_on_l4": ["OM9-L4-01-09-001-01", "OM9-L4-01-09-001-02", "OM9-L4-01-09-001-03"],
    "external_contracts": [
      { "l2_id": "OM9-L2-01-05", "contract": "OCCT bridge lifecycle rule: no TopoDS_Shape outlives one C call, so this adapter returns owned summaries only.", "status": "planned" },
      { "l2_id": "OM9-L2-01-08", "contract": "The document absolute tolerance in millimetres is supplied by the caller (om9_core::history::Tolerance.absolute_mm).", "status": "existing" }
    ],
    "reuse_symbols": [
      { "path": "standalone/crates/om9-core/src/geometry.rs", "symbol": "finite", "kind": "fn", "status": "existing", "from_l4": null, "signature": null, "use": "Reject non-finite origin or center coordinates before marshalling." },
      { "path": "standalone/crates/om9-core/src/geometry.rs", "symbol": "Point3", "kind": "type", "status": "existing", "from_l4": null, "signature": null, "use": "Coordinate type of PrimitiveSpec fields." },
      { "path": "standalone/crates/om9-kernel/src/primitive.rs", "symbol": "PrimitiveSpec::validate", "kind": "method", "status": "planned", "from_l4": "OM9-L4-01-09-001-01", "signature": "pub fn validate(&self) -> Result<(), KernelError>", "use": "First step of build_primitive; its error is returned unchanged." },
      { "path": "standalone/crates/om9-kernel/src/error.rs", "symbol": "KernelError", "kind": "enum", "status": "planned", "from_l4": "OM9-L4-01-09-001-01", "signature": "pub enum KernelError { InvalidParameter { name: &'static str, reason: &'static str }, DegenerateInput(&'static str), OcctUnavailable, OcctFailure { operation: &'static str, code: i32 }, InvalidResult(&'static str) }", "use": "Error type of every function in this task." },
      { "path": "standalone/crates/om9-kernel/src/analytic.rs", "symbol": "mass_properties", "kind": "fn", "status": "planned", "from_l4": "OM9-L4-01-09-001-02", "signature": "pub fn mass_properties(spec: &PrimitiveSpec) -> Result<MassProperties, KernelError>", "use": "Independent oracle for the native tests (volume and area formulas)." },
      { "path": "standalone/crates/om9-occt/src/raw.rs", "symbol": "map_status", "kind": "fn", "status": "planned", "from_l4": "OM9-L4-01-09-001-03", "signature": "pub(crate) fn map_status(operation: &'static str, code: i32) -> Result<(), KernelError>", "use": "Turns the C status code into KernelError after each FFI call." },
      { "path": "standalone/crates/om9-occt/src/raw.rs", "symbol": "summary_from_raw", "kind": "fn", "status": "planned", "from_l4": "OM9-L4-01-09-001-03", "signature": "pub(crate) fn summary_from_raw(raw: &RawSummary) -> Result<BrepSummary, KernelError>", "use": "Converts the repr(C) summary into the owned BrepSummary." },
      { "path": "standalone/crates/om9-occt/native/om9_occt_abi.h", "symbol": "Om9OcctSummary", "kind": "cpp-class", "status": "planned", "from_l4": "OM9-L4-01-09-001-03", "signature": "typedef struct Om9OcctSummary { uint32_t face_count; uint32_t edge_count; uint32_t vertex_count; double volume_mm3; double area_mm2; int32_t is_valid; int32_t is_closed; } Om9OcctSummary;", "use": "C struct filled by the two C++ builders." }
    ]
  },
  "files": {
    "create": [
      { "path": "standalone/crates/om9-occt/src/ops/primitive.rs", "language": "rust", "purpose": "build_primitive, BoxArgs, SphereArgs, box_args, sphere_args and the in-module tests." },
      { "path": "standalone/crates/om9-occt/src/ffi_prim.rs", "language": "rust", "purpose": "The only unsafe code of this task: extern declarations and two safe call wrappers, compiled only with feature occt." },
      { "path": "standalone/crates/om9-occt/native/om9_occt_prim.cpp", "language": "cpp", "purpose": "extern C om9_occt_make_box and om9_occt_make_sphere using BRepPrimAPI_MakeBox and BRepPrimAPI_MakeSphere." }
    ],
    "modify": [
      { "path": "standalone/crates/om9-occt/src/ops/mod.rs", "language": "rust", "change": "Add the line pub mod primitive;", "from_l4": "OM9-L4-01-09-001-03" },
      { "path": "standalone/crates/om9-occt/src/lib.rs", "language": "rust", "change": "Add #[cfg(feature = \"occt\")] #[allow(unsafe_code)] mod ffi_prim;", "from_l4": "OM9-L4-01-09-001-03" },
      { "path": "standalone/crates/om9-occt/build.rs", "language": "rust", "change": "Add native/om9_occt_prim.cpp to the cc::Build file list and to the rerun-if-changed list.", "from_l4": "OM9-L4-01-09-001-03" }
    ]
  },
  "api": {
    "signatures": [
      {
        "path": "standalone/crates/om9-occt/src/ops/primitive.rs",
        "code": "pub fn build_primitive(spec: &PrimitiveSpec, tolerance_mm: f64) -> Result<BrepSummary, KernelError>;\npub(crate) struct BoxArgs { pub origin: [f64; 3], pub axes: [f64; 9], pub size: [f64; 3] }\npub(crate) struct SphereArgs { pub center: [f64; 3], pub radius: f64 }\npub(crate) fn box_args(spec: &PrimitiveSpec) -> Result<BoxArgs, KernelError>;\npub(crate) fn sphere_args(spec: &PrimitiveSpec) -> Result<SphereArgs, KernelError>;"
      },
      {
        "path": "standalone/crates/om9-occt/src/ffi_prim.rs",
        "code": "#[cfg(feature = \"occt\")]\nunsafe extern \"C\" {\n    fn om9_occt_make_box(origin: *const f64, axes: *const f64, size: *const f64, tolerance_mm: f64, out: *mut RawSummary) -> i32;\n    fn om9_occt_make_sphere(center: *const f64, radius: f64, tolerance_mm: f64, out: *mut RawSummary) -> i32;\n}\n#[cfg(feature = \"occt\")]\npub(crate) fn call_make_box(args: &BoxArgs, tolerance_mm: f64) -> Result<RawSummary, KernelError>;\n#[cfg(feature = \"occt\")]\npub(crate) fn call_make_sphere(args: &SphereArgs, tolerance_mm: f64) -> Result<RawSummary, KernelError>;"
      },
      {
        "path": "standalone/crates/om9-occt/native/om9_occt_prim.cpp",
        "code": "extern \"C\" int32_t om9_occt_make_box(const double origin[3], const double axes[9], const double size[3], double tolerance_mm, Om9OcctSummary* out);\nextern \"C\" int32_t om9_occt_make_sphere(const double center[3], double radius, double tolerance_mm, Om9OcctSummary* out);"
      }
    ],
    "data_structures": [
      {
        "name": "BoxArgs",
        "path": "standalone/crates/om9-occt/src/ops/primitive.rs",
        "fields": [
          { "name": "origin", "type": "[f64; 3]", "unit": "mm", "constraint": "finite, each |v| <= MAX_COORD_MM" },
          { "name": "axes", "type": "[f64; 9]", "unit": "none", "constraint": "row-major x0,x1,x2,y0,y1,y2,z0,z1,z2; orthonormal within CPLANE_ORTHO_TOL; y = z cross x" },
          { "name": "size", "type": "[f64; 3]", "unit": "mm", "constraint": "each in [MIN_DIM_MM, MAX_COORD_MM]" }
        ]
      },
      {
        "name": "SphereArgs",
        "path": "standalone/crates/om9-occt/src/ops/primitive.rs",
        "fields": [
          { "name": "center", "type": "[f64; 3]", "unit": "mm", "constraint": "finite, each |v| + radius <= MAX_COORD_MM" },
          { "name": "radius", "type": "f64", "unit": "mm", "constraint": "in [MIN_DIM_MM, MAX_COORD_MM]" }
        ]
      }
    ],
    "error_variants": [
      { "enum": "KernelError", "variant": "InvalidParameter { name, reason }", "display": "invalid parameter {name}: {reason}", "when": "tolerance_mm is outside [TOL_MIN_MM, TOL_MAX_MM], or PrimitiveSpec::validate rejects a field" },
      { "enum": "KernelError", "variant": "OcctUnavailable", "display": "OCCT backend not available in this build", "when": "feature occt is off" },
      { "enum": "KernelError", "variant": "OcctFailure { operation, code }", "display": "OCCT {operation} failed with status {code}", "when": "the C shim returns status 2, 3 or 5" },
      { "enum": "KernelError", "variant": "InvalidResult(&'static str)", "display": "invalid OCCT result: {0}", "when": "BRepCheck_Analyzer rejects the shape (status 4) or the summary has a non-positive or non-finite volume" }
    ],
    "invariants": [
      "No TopoDS_Shape, Handle or pointer to OCCT memory crosses the C ABI; the shape lives on the C++ stack and is destroyed before the function returns.",
      "No C++ exception and no Rust panic crosses the ABI; every shim body is wrapped in try/catch and returns a status code.",
      "unsafe appears only in ffi_prim.rs and each unsafe block has a SAFETY comment naming the pointer lengths (3, 9, 3 doubles).",
      "Without feature occt the crate still compiles and build_primitive validates its inputs, then returns OcctUnavailable."
    ]
  },
  "algorithm": {
    "tolerances": [
      { "name": "TOL_MIN_MM", "value": "1e-9", "unit": "mm" },
      { "name": "TOL_MAX_MM", "value": "1e6", "unit": "mm" },
      { "name": "MIN_DIM_MM", "value": "1e-7", "unit": "mm" },
      { "name": "MAX_COORD_MM", "value": "1e9", "unit": "mm" },
      { "name": "CPLANE_ORTHO_TOL", "value": "1e-6", "unit": "none" },
      { "name": "TEST_VOL_REL_EPS", "value": "1e-6", "unit": "none" }
    ],
    "parameter_ranges": [
      { "name": "tolerance_mm", "range": "[1e-9, 1e6]", "unit": "mm" },
      { "name": "size[i]", "range": "[1e-7, 1e9]", "unit": "mm" },
      { "name": "radius", "range": "[1e-7, 1e9]", "unit": "mm" }
    ],
    "steps": [
      { "n": 1, "action": "Call spec.validate() and return its error unchanged; this covers finiteness, size and radius ranges and frame orthonormality.", "formula": null, "on_failure": "return Err(e); no FFI call" },
      { "n": 2, "action": "Check tolerance_mm is finite and inside the allowed range.", "formula": "TOL_MIN_MM <= tolerance_mm <= TOL_MAX_MM", "on_failure": "Err(InvalidParameter { name: \"tolerance_mm\", reason: \"outside [1e-9, 1e6] mm\" })" },
      { "n": 3, "action": "Marshal: box_args flattens the three axes row-major; sphere_args copies center and radius. Both are pure functions with no OCCT dependency.", "formula": "axes9 = [x0,x1,x2, y0,y1,y2, z0,z1,z2]", "on_failure": "Err(InvalidParameter) when the spec variant is not Box (box_args) or not Sphere (sphere_args)" },
      { "n": 4, "action": "Under #[cfg(not(feature = \"occt\"))] return Err(OcctUnavailable) here, after validation.", "formula": null, "on_failure": null },
      { "n": 5, "action": "Under feature occt call call_make_box or call_make_sphere with pointers to the stack arrays, then map_status(operation, code) where operation is \"make_box\" or \"make_sphere\".", "formula": null, "on_failure": "map_status error returned; out struct ignored" },
      { "n": 6, "action": "C++ box: build gp_Ax2 from origin, Dir(z) and Dir(x), call BRepPrimAPI_MakeBox(ax2, sx, sy, sz) and require IsDone().", "formula": "local box = [0,sx] x [0,sy] x [0,sz] in the frame (x, y = z cross x, z)", "on_failure": "return status 3" },
      { "n": 7, "action": "C++ sphere: BRepPrimAPI_MakeSphere(gp_Ax2(gp_Pnt(center), gp_Dir(0,0,1)), radius) and require IsDone().", "formula": null, "on_failure": "return status 3" },
      { "n": 8, "action": "C++ summary: BRepCheck_Analyzer(shape).IsValid() -> is_valid; BRep_Tool::IsClosed(shape) -> is_closed; TopExp::MapShapes into TopTools_IndexedMapOfShape for TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX; GProp_GProps with BRepGProp::VolumeProperties then BRepGProp::SurfaceProperties.", "formula": "volume_mm3 = VolumeProperties.Mass(); area_mm2 = SurfaceProperties.Mass()", "on_failure": "is_valid false -> status 4" },
      { "n": 9, "action": "Wrap the body in try { OCC_CATCH_SIGNALS ... } catch (Standard_Failure&) { return 2; } catch (...) { return 5; }; return 0 on success.", "formula": null, "on_failure": "statuses 2 and 5 map to OcctFailure" },
      { "n": 10, "action": "Rust: summary_from_raw(&raw) converts 0/1 ints to bool and rejects a non-finite or non-positive volume.", "formula": "volume_mm3 > 0 and finite", "on_failure": "Err(InvalidResult(\"non-positive volume\"))" }
    ],
    "edge_cases": [
      { "case": "Sphere radius exactly MIN_DIM_MM (1e-7 mm)", "behavior": "Accepted by validate; OCCT returns a valid tiny solid; summary_from_raw only requires volume > 0 and finite, so it is returned unchanged." },
      { "case": "Box size component negative or zero", "behavior": "Rejected by PrimitiveSpec::validate in step 1; the command layer normalises signs before building a PrimitiveSpec." },
      { "case": "Axes not right-handed (z . (x cross y) < CPLANE_RIGHT_HAND_MIN)", "behavior": "Rejected by PrimitiveSpec::validate with InvalidParameter { name: \"axes\" }." },
      { "case": "tolerance_mm is NaN", "behavior": "The range comparison is false, so InvalidParameter { name: \"tolerance_mm\" } is returned." }
    ],
    "failures": [
      { "condition": "Sphere radius is 0", "error": "KernelError::InvalidParameter { name: \"radius\", .. }", "message": "invalid parameter radius: <reason from PrimitiveSpec::validate>", "state_after": "no FFI call, nothing allocated" },
      { "condition": "tolerance_mm = 1e-10", "error": "KernelError::InvalidParameter { name: \"tolerance_mm\", .. }", "message": "invalid parameter tolerance_mm: outside [1e-9, 1e6] mm", "state_after": "no FFI call" },
      { "condition": "feature occt is off", "error": "KernelError::OcctUnavailable", "message": "OCCT backend not available in this build", "state_after": "unchanged" },
      { "condition": "BRepPrimAPI_* IsDone() is false", "error": "KernelError::OcctFailure { operation, code: 3 }", "message": "OCCT make_box failed with status 3", "state_after": "C++ locals destroyed, no handle leaked" },
      { "condition": "BRepCheck_Analyzer reports an invalid shape", "error": "KernelError::InvalidResult(\"shape failed BRepCheck_Analyzer\")", "message": "invalid OCCT result: shape failed BRepCheck_Analyzer", "state_after": "C++ locals destroyed, no handle leaked" }
    ]
  },
  "interaction": null,
  "interaction_na_reason": "Backend adapter without prompts; the Matrix Box and Sphere commands that call it are specified in OM9-L3-03-18-012 and OM9-L3-03-18-013.",
  "tests": {
    "file": "standalone/crates/om9-occt/src/ops/primitive.rs",
    "command": "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-occt prim_",
    "native_command": "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-occt --features occt prim_ -- --ignored",
    "cases": [
      { "name": "prim_box_args_flatten_axes_row_major", "kind": "positive", "runs_in": "portable", "input": "PrimitiveSpec::Box { origin [1,2,3], axes x=[0,1,0] y=[-1,0,0] z=[0,0,1], size [10,20,30] }", "expected": "BoxArgs { origin [1,2,3], axes [0,1,0,-1,0,0,0,0,1], size [10,20,30] }" },
      { "name": "prim_sphere_args_copy_center_and_radius", "kind": "positive", "runs_in": "portable", "input": "PrimitiveSpec::Sphere { center [-5,0,2.5], radius 7.25 }", "expected": "SphereArgs { center [-5,0,2.5], radius 7.25 }" },
      { "name": "prim_build_rejects_zero_radius_before_ffi", "kind": "negative", "runs_in": "portable", "input": "build_primitive(Sphere { center [0,0,0], radius 0.0 }, 0.001)", "expected": "Err(KernelError::InvalidParameter { name: \"radius\", .. }) with and without feature occt" },
      { "name": "prim_build_rejects_tolerance_outside_range", "kind": "negative", "runs_in": "portable", "input": "build_primitive(Box 10x20x30 at origin, tolerance_mm = 1e-10) and again with tolerance_mm = 2e6", "expected": "both calls return Err(KernelError::InvalidParameter { name: \"tolerance_mm\", .. })" },
      { "name": "prim_build_without_occt_feature_is_unavailable", "kind": "edge", "runs_in": "portable", "input": "#[cfg(not(feature = \"occt\"))] build_primitive(Box 10x20x30 at origin, axes identity, 0.001)", "expected": "Err(KernelError::OcctUnavailable)" },
      { "name": "prim_box_10_20_30_matches_oracle", "kind": "positive", "runs_in": "native-ignored", "input": "build_primitive(Box 10x20x30 at origin, axes identity, 0.001) with feature occt", "expected": "faces 6, edges 12, vertices 8, is_valid true, is_closed true, volume 6000 mm3 and area 2200 mm2 each within TEST_VOL_REL_EPS" },
      { "name": "prim_sphere_r5_matches_oracle", "kind": "positive", "runs_in": "native-ignored", "input": "build_primitive(Sphere { center [0,0,0], radius 5.0 }, 0.001) with feature occt", "expected": "faces 1, is_valid true, volume 523.5987755982989 mm3 and area 314.1592653589793 mm2 each within TEST_VOL_REL_EPS" }
    ],
    "fixtures": []
  },
  "definition_of_done": [
    "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-occt prim_ passes with 5 portable tests.",
    "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-occt --features occt prim_ -- --ignored passes on a machine with OCCT_INCLUDE_DIR and OCCT_LIB_DIR set; if OCCT is absent the session note says 'native tests not run: OCCT missing'.",
    "cargo clippy --locked --manifest-path standalone/Cargo.toml -p om9-occt --all-targets -- -D warnings is clean.",
    "cargo fmt --manifest-path standalone/Cargo.toml -p om9-occt --check is clean.",
    "unsafe code exists only in ffi_prim.rs, each block has a SAFETY comment, and no qymcad crate appears in any Cargo.toml.",
    "No OCCT handle crosses the ABI and the C++ file frees every local before returning."
  ],
  "references": {
    "text": [
      "../reference/text/m8book1__p232-233__box.md",
      "../reference/text/m8book1__p233-233__sphere.md"
    ],
    "images": [
      "../reference/images/m8book1_p232_img01.png",
      "../reference/images/m8book1_p233_img01.png"
    ],
    "qymcad": [
      "QymCAD/crates/qymcad-kernel/build.rs",
      "QymCAD/crates/qymcad-kernel/src/occt_bridge.cpp",
      "QymCAD/crates/qymcad-kernel/src/occt_common.hpp",
      "QymCAD/crates/qymcad-kernel/src/shape/build.rs",
      "QymCAD/crates/qymcad-kernel/tests/every_entry_catches_what_occt_throws.rs"
    ],
    "legacy_om9": [
      "rust/src/solid.rs",
      "Gui/CoreSolidResult.cpp"
    ],
    "behavior_notes": [
      "Book 1 p.232-233: Box is two base corners plus a height; Sphere is a center plus a radius. The prose is two-column interleaved, so only the analytic formulas (box V = sx*sy*sz, sphere V = 4/3*pi*r^3) define the expected numbers.",
      "QymCAD qym_shape_sphere (occt_bridge.cpp) shows the try/QYM_SIGNALS/catch shape that every shim function must follow; build.rs shows the OCCT_INCLUDE_DIR and OCCT_LIB_DIR convention."
    ],
    "no_reference_reason": null
  },
  "assumptions": [
    { "id": "A1", "statement": "Which task creates the om9-occt crate if M01 OM9-L2-01-05 lands first.", "chosen": "OM9-L4-01-09-001-03 creates it; if the crate already exists that L4 only adds missing files." },
    { "id": "A2", "statement": "Whether sphere edge and vertex counts are stable across OCCT versions.", "chosen": "They are not asserted; only face_count == 1 is checked for the sphere because seam and degenerate edge counting is version dependent." },
    { "id": "A3", "statement": "The exact wording of PrimitiveSpec::validate messages.", "chosen": "They are owned by OM9-L4-01-09-001-01; tests here match on the variant and the field name only." },
    { "id": "A4", "statement": "Whether BRepGProp should use an epsilon integration mode.", "chosen": "Use the default exact analytic integration (no epsilon argument) because both solids are analytic." }
  ]
}
```

---------------------------------------------------------------------------------------------------

## 7. Example 2 - febe / UI menu-binding L4 (L3 `OM9-L3-03-18-012`, tag `febe`, "Solid · Box - menu binding")

Planned L4 split of that L3 (see 8.2): `001 Box Registry And Types`, `002 Box Corners Prompt Session` (this example),
`003 Box Mode Options Session`, `004 Box Commit And Undo Publish`, `005 Box Menu Cmd Parity Test`.
Prompt strings and Enter/Undo/Esc behavior are ported from the legacy `rust/src/solid.rs` (`Session::command_options`,
`input_inner`, `point_geometry`, `finish_box`) and Book 1 p.232-233.

```json l4-example folder="L4-002 - Box Corners Prompt Session"
{
  "schema_version": "om9.l4.v1",
  "level": "L4",
  "id": "OM9-L4-03-18-012-02",
  "milestone": "M03",
  "parent_l3_id": "OM9-L3-03-18-012",
  "parent_l2_id": "OM9-L2-03-18",
  "seq": 2,
  "title": "Box Corners Prompt Session",
  "planning_state": "SPEC_ONLY_NOT_IMPLEMENTED",
  "tag": "febe",
  "owner_layer": "command-session",
  "language": "rust",
  "kind": "command-session",
  "size": { "impl_loc": 190, "test_loc": 200 },
  "goal": "Implement BoxCornersSession in om9-commands so the default Box method asks First corner of base, Other corner of base and Height with the Matrix Enter, Undo and Esc behavior and yields exactly one BoxRequest.",
  "scope": {
    "inputs": [
      "PlaneFrame { origin: Point3 (mm), axes: [Point3; 3] } captured from the active CPlane when the command starts",
      "SessionInput::Point(Point3) in document millimetres, already resolved onto the CPlane by the caller",
      "SessionInput::Number(f64) in millimetres (Height only) and SessionInput::Enter",
      "SessionInput::Option(String) carrying a typed option keyword"
    ],
    "outputs": [
      "SessionStep::Done(CommandOutput::Box(BoxRequest { origin, axes, size })) with right-handed axes and every size component > 0",
      "A Prompt snapshot (id, exact text, options) after every accepted input",
      "Err(CommandError) with the session state unchanged for any rejected input"
    ],
    "not_in_this_task": [
      "Diagonal, 3Point, Vertical, Center and Cube methods (OM9-L4-03-18-012-03)",
      "Building the BRep, publishing to the document, History and Undo of the document (OM9-L4-03-18-012-04)",
      "Registry entry, menu and CMD wiring, availability (OM9-L4-03-18-012-01)",
      "Rubber-band wire preview drawing",
      "Parsing unit suffixes such as 10mm or 1in; the caller converts text with om9_core::geometry::Unit"
    ]
  },
  "dependencies": {
    "depends_on_l4": ["OM9-L4-03-18-012-01"],
    "external_contracts": [
      { "l2_id": "OM9-L2-01-09", "contract": "The primitive service consumes BoxRequest; this task only produces the request and never calls it.", "status": "planned" },
      { "l2_id": "OM9-L2-03-07", "contract": "Matrix main menu and the CMD frame start the same registry entry OM9-SOLID-012.", "status": "planned" }
    ],
    "reuse_symbols": [
      { "path": "standalone/crates/om9-core/src/geometry.rs", "symbol": "finite", "kind": "fn", "status": "existing", "from_l4": null, "signature": null, "use": "Reject non-finite point coordinates before any state change." },
      { "path": "standalone/crates/om9-core/src/geometry.rs", "symbol": "Point3", "kind": "type", "status": "existing", "from_l4": null, "signature": null, "use": "Coordinate type of every point in the session." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "CommandSession", "kind": "trait", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub trait CommandSession { fn prompt(&self) -> Prompt; fn accept(&mut self, input: SessionInput) -> Result<SessionStep, CommandError>; fn undo(&mut self) -> Result<(), CommandError>; fn cancel(&mut self); }", "use": "Trait implemented by BoxCornersSession." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "SessionInput", "kind": "enum", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub enum SessionInput { Point(Point3), Number(f64), Enter, Option(String) }", "use": "Input accepted by accept." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "SessionStep", "kind": "enum", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub enum SessionStep { Continue, Done(CommandOutput) }", "use": "Result of an accepted input." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "Prompt", "kind": "struct", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub struct Prompt { pub id: &'static str, pub text: String, pub options: Vec<PromptOption> }", "use": "Prompt snapshot returned by prompt()." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "PromptOption", "kind": "struct", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub struct PromptOption { pub name: &'static str, pub alias: Option<&'static str>, pub enabled: bool, pub reason: Option<&'static str> }", "use": "One visible option; disabled ones carry a reason." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "CommandError", "kind": "enum", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub enum CommandError { PointRequired, InvalidValue(&'static str), OptionDisabled { name: String }, NothingToUndo, Finished }", "use": "Error type of accept and undo." },
      { "path": "standalone/crates/om9-commands/src/solid/mod.rs", "symbol": "BoxRequest", "kind": "struct", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub struct BoxRequest { pub origin: Point3, pub axes: [Point3; 3], pub size: [f64; 3] }", "use": "Value produced when the command completes." },
      { "path": "standalone/crates/om9-commands/src/session.rs", "symbol": "PlaneFrame", "kind": "struct", "status": "planned", "from_l4": "OM9-L4-03-18-012-01", "signature": "pub struct PlaneFrame { pub origin: Point3, pub axes: [Point3; 3] }", "use": "Frame captured from the CPlane; axes[2] is the Height direction." }
    ]
  },
  "files": {
    "create": [
      { "path": "standalone/crates/om9-commands/src/solid/box_session.rs", "language": "rust", "purpose": "BoxCornersSession state machine for the default Corners method." },
      { "path": "standalone/crates/om9-commands/tests/box_corners_session.rs", "language": "rust", "purpose": "Integration tests that drive BoxCornersSession only through the public CommandSession trait." }
    ],
    "modify": [
      { "path": "standalone/crates/om9-commands/src/solid/mod.rs", "language": "rust", "change": "Add the line pub mod box_session; and re-export BoxCornersSession.", "from_l4": "OM9-L4-03-18-012-01" }
    ]
  },
  "api": {
    "signatures": [
      {
        "path": "standalone/crates/om9-commands/src/solid/box_session.rs",
        "code": "pub struct BoxCornersSession { /* private: frame, phase, first, length, width, history */ }\nimpl BoxCornersSession {\n    pub fn new(frame: PlaneFrame) -> Self;\n}\nimpl CommandSession for BoxCornersSession {\n    fn prompt(&self) -> Prompt;\n    fn accept(&mut self, input: SessionInput) -> Result<SessionStep, CommandError>;\n    fn undo(&mut self) -> Result<(), CommandError>;\n    fn cancel(&mut self);\n}\nfn dimension(n: f64) -> Result<f64, CommandError>;"
      }
    ],
    "data_structures": [
      {
        "name": "BoxPhase",
        "path": "standalone/crates/om9-commands/src/solid/box_session.rs",
        "fields": [
          { "name": "FirstCorner | OtherCorner | Height | Finished | Canceled", "type": "enum", "unit": "none", "constraint": "transitions only FirstCorner -> OtherCorner -> Height -> Finished; undo moves one step back; cancel is terminal" }
        ]
      },
      {
        "name": "BoxCornersSession",
        "path": "standalone/crates/om9-commands/src/solid/box_session.rs",
        "fields": [
          { "name": "frame", "type": "PlaneFrame", "unit": "mm", "constraint": "axes orthonormal and right-handed (checked by the caller from CPlane)" },
          { "name": "phase", "type": "BoxPhase", "unit": "none", "constraint": "see BoxPhase" },
          { "name": "first", "type": "Option<Point3>", "unit": "mm", "constraint": "Some after the first corner is accepted" },
          { "name": "length", "type": "f64", "unit": "mm", "constraint": "signed distance of the second corner along axes[0]; nonzero after OtherCorner" },
          { "name": "width", "type": "f64", "unit": "mm", "constraint": "signed distance of the second corner along axes[1]; nonzero after OtherCorner" },
          { "name": "history", "type": "Vec<(BoxPhase, Option<Point3>, f64, f64)>", "unit": "none", "constraint": "snapshot pushed before each accepted input; popped by undo; max 3 entries" }
        ]
      }
    ],
    "error_variants": [
      { "enum": "CommandError", "variant": "PointRequired", "display": "Enter a point or dimension before confirming", "when": "Enter is pressed at FirstCorner or OtherCorner" },
      { "enum": "CommandError", "variant": "InvalidValue(&'static str)", "display": "the carried message, for example: Enter a value other than zero", "when": "a dimension is zero, below MIN_DIM_MM, above MAX_COORD_MM or non-finite, or the input kind does not fit the phase" },
      { "enum": "CommandError", "variant": "OptionDisabled { name }", "display": "Option is unavailable at this step", "when": "a mode keyword (Diagonal, 3Point, Vertical, Center) is typed before OM9-L4-03-18-012-03 enables it" },
      { "enum": "CommandError", "variant": "NothingToUndo", "display": "No input to undo", "when": "undo at FirstCorner with an empty history" },
      { "enum": "CommandError", "variant": "Finished", "display": "command already finished", "when": "any input after Done or cancel" }
    ],
    "invariants": [
      "A rejected input leaves phase, first, length, width and history exactly as before the call.",
      "BoxRequest.size components are all >= MIN_DIM_MM and origin plus every corner satisfy |coordinate| <= MAX_COORD_MM.",
      "BoxRequest.axes equals PlaneFrame.axes; the session never rotates the frame.",
      "The session never touches the document, the scene or any global state."
    ]
  },
  "algorithm": {
    "tolerances": [
      { "name": "MIN_DIM_MM", "value": "1e-7", "unit": "mm" },
      { "name": "MAX_COORD_MM", "value": "1e9", "unit": "mm" }
    ],
    "parameter_ranges": [
      { "name": "length, width, height (absolute value)", "range": "[1e-7, 1e9]", "unit": "mm" },
      { "name": "every corner coordinate", "range": "[-1e9, 1e9]", "unit": "mm" }
    ],
    "steps": [
      { "n": 1, "action": "new(frame): phase = FirstCorner, first = None, length = 0, width = 0, history empty.", "formula": null, "on_failure": null },
      { "n": 2, "action": "prompt(): FirstCorner -> id corner1, text \"First corner of base\", options Diagonal(D), 3Point(3P), Vertical(V), Center(C) all enabled=false with reason \"Box mode options are enabled by OM9-L4-03-18-012-03\"; OtherCorner -> id corner2, text \"Other corner of base\", no options; Height -> id height, text \"Height. Press Enter to use width <W mm>\" with W the absolute width formatted by Rust {} (20.0 prints 20, 12.5 prints 12.5), no options.", "formula": "W = |width|", "on_failure": null },
      { "n": 3, "action": "accept(Point p) at FirstCorner: check finite(p) and |each coordinate| <= MAX_COORD_MM, push a snapshot, set first = p, phase = OtherCorner, return Continue.", "formula": null, "on_failure": "InvalidValue(\"Invalid point coordinates\")" },
      { "n": 4, "action": "accept(Point p) at OtherCorner: compute the signed extents along the frame axes, validate each with dimension(), push a snapshot, store length and width, phase = Height, return Continue.", "formula": "d = p - first; length = d . axes[0]; width = d . axes[1]", "on_failure": "the dimension() error; state unchanged" },
      { "n": 5, "action": "dimension(n): reject non-finite, zero, tiny and huge values in this order.", "formula": "not finite -> \"Enter a valid number\"; n == 0 -> \"Enter a value other than zero\"; |n| < MIN_DIM_MM -> \"This value is too small. Enter a larger value\"; |n| > MAX_COORD_MM -> \"This value is too large. Enter a smaller value\"", "on_failure": "InvalidValue(<message>)" },
      { "n": 6, "action": "accept(Number h) at Height calls finish(h); accept(Enter) at Height calls finish(|width|); accept(Point p) at Height calls finish((p - first) . axes[2]).", "formula": null, "on_failure": null },
      { "n": 7, "action": "finish(h): h = dimension(h)?; dims = [length, width, h]; origin = first; for each i with dims[i] < 0 add axes[i] * dims[i] to origin; size = absolute values; check all 8 corners origin + sum(axes[i] * size[i] * bit_i) stay within MAX_COORD_MM; phase = Finished; return Done(CommandOutput::Box(BoxRequest { origin, axes: frame.axes, size })).", "formula": "corner(bits) = origin + sum over i of bit_i * size[i] * axes[i]", "on_failure": "InvalidValue(\"Box exceeds supported coordinate range\")" },
      { "n": 8, "action": "accept(Enter) at FirstCorner or OtherCorner returns PointRequired; accept(Number) there returns InvalidValue(\"Pick the required point or enter coordinates X,Y,Z; 0 means the construction-plane origin\"); accept(Option(name)) returns OptionDisabled for the four mode names and InvalidValue(\"Option is unavailable at this step\") for any other name.", "formula": null, "on_failure": "state unchanged" },
      { "n": 9, "action": "undo(): pop the last snapshot and restore it; with an empty history return NothingToUndo. cancel(): phase = Canceled and clear history; any later accept or undo returns Finished.", "formula": null, "on_failure": "NothingToUndo, Finished" }
    ],
    "edge_cases": [
      { "case": "Second corner has x offset 0 (same length axis)", "behavior": "length is 0, dimension() returns \"Enter a value other than zero\", phase stays OtherCorner." },
      { "case": "Second corner is left of and below the first (negative length and width)", "behavior": "Accepted; finish moves origin by the negative extents so BoxRequest.size is positive." },
      { "case": "Height is negative", "behavior": "Accepted; origin moves along axes[2] by the negative height." },
      { "case": "Height typed as 1e-8", "behavior": "Rejected as too small; prompt stays at height." }
    ],
    "failures": [
      { "condition": "Zero length or zero width at the second corner", "error": "CommandError::InvalidValue(\"Enter a value other than zero\")", "message": "Enter a value other than zero", "state_after": "phase OtherCorner, first kept, prompt corner2" },
      { "condition": "Height below MIN_DIM_MM", "error": "CommandError::InvalidValue(\"This value is too small. Enter a larger value\")", "message": "This value is too small. Enter a larger value", "state_after": "phase Height, prompt height" },
      { "condition": "Height above MAX_COORD_MM", "error": "CommandError::InvalidValue(\"This value is too large. Enter a smaller value\")", "message": "This value is too large. Enter a smaller value", "state_after": "phase Height, prompt height" },
      { "condition": "Undo with empty history", "error": "CommandError::NothingToUndo", "message": "No input to undo", "state_after": "unchanged" }
    ]
  },
  "interaction": {
    "feature_id": "OM9-SOLID-012",
    "command": "Box",
    "entry_points": [
      "menu: Solid > Box",
      "cmd: Box (same registry entry and same BoxCornersSession as the menu; see OM9-L4-03-18-012-01)"
    ],
    "prompts": [
      {
        "id": "corner1",
        "text": "First corner of base",
        "accepts": "A point picked on the CPlane or typed as x,y[,z] in page units",
        "options": [
          { "name": "Diagonal", "alias": "D", "default": null, "enabled": false, "reason": "Box mode options are enabled by OM9-L4-03-18-012-03" },
          { "name": "3Point", "alias": "3P", "default": null, "enabled": false, "reason": "Box mode options are enabled by OM9-L4-03-18-012-03" },
          { "name": "Vertical", "alias": "V", "default": null, "enabled": false, "reason": "Box mode options are enabled by OM9-L4-03-18-012-03" },
          { "name": "Center", "alias": "C", "default": null, "enabled": false, "reason": "Box mode options are enabled by OM9-L4-03-18-012-03" }
        ],
        "on_enter": "Error: Enter a point or dimension before confirming",
        "on_esc": "Cancel the session; the document is unchanged",
        "on_undo": "Error: No input to undo"
      },
      {
        "id": "corner2",
        "text": "Other corner of base",
        "accepts": "A point only; a bare number is rejected in this method",
        "options": [],
        "on_enter": "Error: Enter a point or dimension before confirming",
        "on_esc": "Cancel the session; the document is unchanged",
        "on_undo": "Return to prompt corner1 and forget the first corner"
      },
      {
        "id": "height",
        "text": "Height. Press Enter to use width <W mm>",
        "accepts": "A signed number in mm, or a point whose distance along the CPlane Z axis from the first corner is the height",
        "options": [],
        "on_enter": "Use the absolute width as the height and complete the command",
        "on_esc": "Cancel the session; the document is unchanged",
        "on_undo": "Return to prompt corner2 and forget the second corner"
      }
    ],
    "defaults": [
      { "name": "Method", "value": "Corners (no keyword needed)" },
      { "name": "Height", "value": "absolute width of the base when Enter is pressed" }
    ],
    "keys": {
      "enter": "At corner1 and corner2 an error; at height it accepts the default height and finishes.",
      "esc": "Cancels the session at every prompt without changing the document.",
      "undo": "Steps back one accepted input; an error at corner1 with nothing to undo."
    },
    "preview": "The wire box preview after the second corner is drawn by OM9-L4-03-18-012-04 and is not part of this task.",
    "source_note": "Prompt texts and Enter defaults ported from rust/src/solid.rs (Session::command_options phases 1, 2 and 4, input_inner, finish_box); step order from Book 1 p.232-233 Box steps."
  },
  "tests": {
    "file": "standalone/crates/om9-commands/tests/box_corners_session.rs",
    "command": "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-commands box_corners_",
    "native_command": null,
    "cases": [
      { "name": "box_corners_flow_commits_with_enter_height", "kind": "positive", "runs_in": "portable", "input": "frame = world XY (origin [0,0,0], axes identity); Point [0,0,0]; Point [10,20,0]; then Enter", "expected": "after the second point prompt id is height with text \"Height. Press Enter to use width <20 mm>\"; Enter returns Done(Box(BoxRequest { origin [0,0,0], axes identity, size [10,20,20] }))" },
      { "name": "box_corners_negative_extents_shift_origin", "kind": "positive", "runs_in": "portable", "input": "same frame; Point [0,0,0]; Point [-10,-20,0]; Number 5", "expected": "Done(Box(BoxRequest { origin [-10,-20,0], size [10,20,5] }))" },
      { "name": "box_corners_height_point_uses_z_distance", "kind": "positive", "runs_in": "portable", "input": "same frame; Point [5,5,0]; Point [15,25,0]; Point [99,99,7.5]", "expected": "Done(Box(BoxRequest { origin [5,5,0], size [10,20,7.5] }))" },
      { "name": "box_corners_negative_height_shifts_origin_down", "kind": "edge", "runs_in": "portable", "input": "same frame; Point [0,0,0]; Point [10,20,0]; Number -5", "expected": "Done(Box(BoxRequest { origin [0,0,-5], size [10,20,5] }))" },
      { "name": "box_corners_zero_length_rejected_state_kept", "kind": "negative", "runs_in": "portable", "input": "same frame; Point [0,0,0]; Point [0,20,0]", "expected": "second accept returns Err(InvalidValue(\"Enter a value other than zero\")); prompt() still has id corner2 and text \"Other corner of base\"" },
      { "name": "box_corners_height_out_of_range_rejected", "kind": "negative", "runs_in": "portable", "input": "same frame; Point [0,0,0]; Point [10,20,0]; Number 1e-8; then Number 2e9", "expected": "first Err(InvalidValue(\"This value is too small. Enter a larger value\")), second Err(InvalidValue(\"This value is too large. Enter a smaller value\")); prompt id stays height" },
      { "name": "box_corners_undo_walks_back_then_reports_empty", "kind": "edge", "runs_in": "portable", "input": "same frame; Point [0,0,0]; Point [10,20,0]; undo; undo; undo", "expected": "after the first undo prompt id is corner2, after the second corner1, third undo returns Err(NothingToUndo) displaying \"No input to undo\"" },
      { "name": "box_corners_mode_options_visible_but_disabled", "kind": "positive", "runs_in": "portable", "input": "new session; read prompt(); accept(Option(\"Diagonal\"))", "expected": "options names [Diagonal, 3Point, Vertical, Center], aliases [D, 3P, V, C], all enabled false with reason \"Box mode options are enabled by OM9-L4-03-18-012-03\"; accept returns Err(OptionDisabled { name: \"Diagonal\" })" },
      { "name": "box_corners_cancel_then_input_reports_finished", "kind": "negative", "runs_in": "portable", "input": "new session; Point [0,0,0]; cancel(); accept(Point [1,1,0])", "expected": "Err(Finished) displaying \"command already finished\"" }
    ],
    "fixtures": []
  },
  "definition_of_done": [
    "cargo test --locked --manifest-path standalone/Cargo.toml -p om9-commands box_corners_ passes with 9 tests.",
    "cargo clippy --locked --manifest-path standalone/Cargo.toml -p om9-commands --all-targets -- -D warnings is clean.",
    "cargo fmt --manifest-path standalone/Cargo.toml -p om9-commands --check is clean.",
    "Prompt ids and texts equal the table in interaction.prompts character for character.",
    "The module has no dependency on om9-app, egui or any scene type, so the session is testable headless.",
    "No code in the task reads or writes a document, and no qymcad crate appears in any Cargo.toml."
  ],
  "references": {
    "text": ["../reference/text/m8book1__p232-233__box.md"],
    "images": ["../reference/images/m8book1_p232_img01.png"],
    "qymcad": [],
    "legacy_om9": ["rust/src/solid.rs", "rust/src/command_options.rs"],
    "behavior_notes": [
      "Book 1 p.232-233 Box steps: select Box from the Solid menu, pick the first corner of the base, pick the other corner, then the height; Enter at height reuses the width.",
      "Legacy Session shows the option keywords Diagonal, 3Point, Vertical, Center and Cube on the first prompt only; here they are visible but disabled until the mode task lands."
    ],
    "no_reference_reason": null
  },
  "assumptions": [
    { "id": "A1", "statement": "Whether a point picked at the Height prompt is measured from the first corner or from the opposite base corner.", "chosen": "From the first corner along axes[2], matching the legacy Session (dot(p - first, axes[2])); both give the same value because the base lies in one plane." },
    { "id": "A2", "statement": "How W in the Height prompt is formatted.", "chosen": "Rust default f64 Display of the absolute width, so 20.0 prints 20 and 12.5 prints 12.5, as the legacy format! call does." }
  ],
  "notes_vi": "Lời nhắc (prompt) giữ nguyên tiếng Anh như Matrix9; chỉ ghi chú này dành cho người đọc."
}
```

---------------------------------------------------------------------------------------------------

## 8. Decomposition recipes (3-8 L4 per L3, one short session each)

Total expectation: roughly 900-1000 L4 for the 243 L3. Never merge two L3 into one L4. An L4 that would exceed 250
implementation lines is split. A trivial L3 may have 2 L4 (never 1 unless `impl_loc <= 120`).

### 8.1 `backend_framework` (tag `be`, 36 L3: L2-01-09/10/11/13)

1. `Spec And Errors` - data types, `KernelError` variants, `validate()` in `om9-kernel` (pure Rust). First L4 creates the crate.
2. `Analytic Oracle` or `Algorithm Core` - pure-Rust formulas/algorithm (volume/area, frame math, matrix composition) that double as the test oracle.
3. `Adapter Skeleton And ABI` - ONLY for the first OCCT/openNURBS L3 of the program: `om9-occt` crate, `build.rs`, feature `occt`, `RawSummary`, `map_status`.
4..n. One adapter operation family per L4 (C++ shim + safe wrapper + tests).
n+1. `Fixture And Regression` - golden numbers, degenerate inputs, cancellation.
The L3 `api_contract.edge_cases` ("Invalid/empty geometry, cancellation, tolerance/pathological input") must each appear as an
`algorithm.edge_cases` or `failures` entry in some L4 of the L3.

### 8.2 `interactive_feature` (tag `febe`, 42 L3: L2-01-13 UI, 02-04/05/06, 03-19/20) and `ui_command_adapter` (165 menu bindings)

1. `Registry Entry And Types` - register `feature_id` (see `standalone/crates/om9-app/assets/features.json`), availability through
   `om9_core::capability::CapabilityRegistry::availability(operation_id)` (disabled with reason until the owner capability has
   receipts), request/response types. First menu L3 of a family also creates the family module (`om9-commands/src/<family>/mod.rs`).
2..k. One L4 per prompt flow or method (exact prompts, options, defaults, Enter/Esc/Undo from the manual and legacy Rust).
k+1. `Commit And Undo Publish` - build the owner request, call the owning L2 service (NEVER reimplement geometry in a menu adapter),
   publish through `TransactionCoordinator`/`TransactionRequest::new(label, mutations)`, one Undo step, visible failure message.
last. `Menu Cmd Parity Test` - menu path and CMD path reach the same session; cancel/undo leave the document unchanged.
Menu-binding L3 of the same L2 reuse the family module of the first one via `depends_on_l4`; they never copy it.
Algorithms owned by another L3 (`reuses_or_dispatches_to_module_ids` in `l3.json`) are called through the owner's request type, and the
binding L4 depends on the owner's `Registry Entry And Types` or service L4 (cross-L3 `depends_on_l4`). Author owners before
bindings so the dependency ids exist.

### 8.3 Per-L3 `L4_INDEX.md` format (required, validated)

```markdown
# L4 Index - OM9-L3-03-18-012 - Solid · Box — menu binding

- L3: OM9-L3-03-18-012 (`l3.json` in this folder)
- Owner L2: OM9-L2-03-18
- Role: ui_command_adapter (febe)
- Module path: standalone/crates/om9-commands/src/solid/
- L4 count: 5; total estimated implementation LOC: 860

## L4 list

| Seq | L4 id | Folder | Title | Kind | Layer | Depends on (L4) | LOC impl/test |
|---|---|---|---|---|---|---|---|
| 1 | OM9-L4-03-18-012-01 | L4-001 - Box Registry And Types | Box Registry And Types | contract | command-session | none | 170/150 |
| 2 | OM9-L4-03-18-012-02 | L4-002 - Box Corners Prompt Session | Box Corners Prompt Session | command-session | command-session | OM9-L4-03-18-012-01 | 190/200 |
| 3 | OM9-L4-03-18-012-03 | L4-003 - Box Mode Options Session | Box Mode Options Session | command-session | command-session | OM9-L4-03-18-012-02 | 180/190 |
| 4 | OM9-L4-03-18-012-04 | L4-004 - Box Commit And Undo Publish | Box Commit And Undo Publish | ui-binding | ui-binding | OM9-L4-03-18-012-03, OM9-L4-01-09-001-04 | 200/180 |
| 5 | OM9-L4-03-18-012-05 | L4-005 - Box Menu Cmd Parity Test | Box Menu Cmd Parity Test | integration-test | ui-binding | OM9-L4-03-18-012-04 | 120/160 |

## Dependency order

1. OM9-L4-03-18-012-01
2. OM9-L4-03-18-012-02
3. OM9-L4-03-18-012-03
4. OM9-L4-03-18-012-04
5. OM9-L4-03-18-012-05

## Recommended implementation order

1. OM9-L4-03-18-012-01 - everything else needs its types
2. OM9-L4-03-18-012-02 - first complete prompt flow, testable headless
3. OM9-L4-03-18-012-03 - remaining methods reuse the same session types
4. OM9-L4-03-18-012-04 - needs the OCCT builders of the other L3, so it goes last among the code tasks
5. OM9-L4-03-18-012-05 - closes the L3 with the menu versus CMD parity check

## Cross-L3 notes

- OM9-L4-03-18-012-04 needs OM9-L4-01-09-001-04 (OCCT Box Sphere Builders) first; author and implement that L3 before this one.
```

Validator rules for the index: first line is `# L4 Index - <L3 id> - <title>`; the three sections exist; the table lists every L4 id
of that L3 exactly once; both numbered lists contain every L4 id exactly once, start each item with the id, and are
topologically ordered with respect to `depends_on_l4`. "Dependency order" is the strict topological order; "Recommended
implementation order" may differ only by choosing among independent items (for example native-risk items last) and still obeys dependencies.

### 8.4 Optional `L4.md` format

First line `# <id> - <title>`, then these headings exactly: `## Goal`, `## Scope`, `## Steps`, `## Tests`,
`## Definition of done`, `## References`. It restates `l4.json` in readable prose (Vietnamese or English), must mention every test
function name, and is never normative: `l4.json` wins on any conflict.

---------------------------------------------------------------------------------------------------

## 9. Crate and module map (discovered in the repo)

### 9.1 Existing standalone workspace `standalone/` (target runtime)

`standalone/Cargo.toml`: resolver 3, edition 2024, `rust-version` 1.88, license LGPL-2.1-or-later, dependencies pinned with `=`
(`serde =1.0.228`, `serde_json =1.0.150`, `sha2 =0.10.9`, `uuid =1.23.3`). `Cargo.lock` is committed (345 packages, `cc 1.8.0` is present transitively).

| Crate | Key modules (verified) | M03 relevance |
|---|---|---|
| `om9-core` (`#![forbid(unsafe_code)]`) | `geometry` (Point3, Transform, Curve, NurbsCurve, NurbsSurface, Geometry, Unit, Bounds), `evaluation`, `request`, `result`, `capability`, `history`, `transaction`, `model`, `identity`, `budget`, `error`, `retained`, `layers`, `jobs`, `resource` | numeric types, errors, transactions, capability gating, units/tolerance |
| `om9-scene` | `scene` (Scene, pick), `affine` (AffineGesture), `camera` (CPlane, ray), `workspace` (ViewId), `picking`, `grid` | CPlane frames, previews, gumball-style transforms |
| `om9-app` (`rust-version` 1.95) | `session` (Controller: dispatch/submit/prompt/cancel/availability; today only `OM9-CURVE-002` Line, `OM9-CURVE-034` Point), `transcript`, `ui/*`, `assets/features.json` (feature ids), `layer_command` | command routing, CMD frame, menu IDs |
| `om9-exchange` | `ImportCapture`, `SourcePreparation`, `ImportOptions`, `ImportMode`, `ImportReceipt`, `NativeExchange`, `ValidatedExport`, `import_dto`, `retained_*` | `.3dm` import/export (L2-01-13) |
| `om9-archive` | `.om9` container, `admit_json`, atomic file | persistence |
| `om9-native-owner` | Win32 process/file adapters | file picker (L2-01-13 L3-010) |

### 9.2 Native (C++) already in the standalone profile

`standalone/native/CMakeLists.txt` (CMake 3.25, C++20, VS2022 x64): targets `om9-exchange-native` (`native_exchange.cpp`; modes `--write`,
`--inspect`, `--inventory-source`, `--project-source`, `--prepare-retained-source`, `--write-retained`, `--inspect-retained-current`) and
`om9-opennurbs-probe` (`opennurbs_probe.cpp`). openNURBS is pinned at `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`
in `build/standalone-deps/opennurbs`. Python receipts: `tools/validate_standalone_{foundation,ui,exchange,import,resources,opennurbs_probe}.py`.
There is NO OCCT crate in the standalone profile.

### 9.3 Legacy crate (port source only) `rust/` = `openmatrix9_rust` (`staticlib` + `rlib`, run `cargo test --manifest-path rust/Cargo.toml`)

Relevant modules: `solid.rs` (Box/Sphere sessions, exact prompts), `surface.rs` (Sweep1/Sweep2/Loft sessions), `curve.rs`, `circle.rs`, `ellipse.rs`,
`rectangle.rs`, `spline.rs`, `nurbs_basis.rs`, `polycurve_contract.rs`, `geometry_contract.rs`, `tolerances.rs`, `units.rs`, `command_options.rs`
(OptionSpec/OptionSession), `coordinates.rs`, `coordinate_input.rs`, `cage.rs`, `edit.rs`, `array_occurrences.rs`, `instance_graph.rs`, `topology_snapshot.rs`,
`shape_validation.rs`, `solid_result.rs`, `surface_result.rs`, `curve_result.rs`. C++ host samples: `Gui/CoreSolidResult.cpp`, `Gui/CoreSurfaceResult.cpp`,
`Gui/CoreShapeValidation.cpp`, `tests/native/occt_offset_d3_probe.cpp`.

### 9.4 PLANNED new crates created by M03 L4 (names fixed here; none exists today)

| Crate (path) | Purpose | Rules |
|---|---|---|
| `om9-kernel` (`standalone/crates/om9-kernel`) | Pure-Rust geometry-kernel contracts and algorithms: `error` (KernelError), `tol`, `frame`, `primitive`, `analytic`, `profile`, `extrude`, `revolve`, `loft`, `sweep`, `boolean`, `blend`, `shell`, `heal`, `xform`, `pattern`, `brep` (BrepSummary + trait) | `#![forbid(unsafe_code)]`; depends on `om9-core` only; created by the first be-L3 L4 (`OM9-L4-01-09-001-01`) |
| `om9-occt` (`standalone/crates/om9-occt`) | Capability-scoped OCCT adapter. `src/lib.rs`, `src/raw.rs` (repr(C) summary, `map_status`), `src/ops/<family>.rs` safe wrappers, `src/ffi_<family>.rs` (only place with `unsafe`), `native/*.cpp`, `native/om9_occt_abi.h`, `build.rs` | cargo feature `occt` (default off); `build.rs` uses `cc =1.8.0` and env `OCCT_INCLUDE_DIR`, `OCCT_LIB_DIR`; MSVC needs `/EHa`, `NOMINMAX`, `WIN32_LEAN_AND_MEAN` (pattern from `QymCAD/crates/qymcad-kernel/build.rs`); created by `OM9-L4-01-09-001-03` |
| `om9-commands` (`standalone/crates/om9-commands`) | Headless command sessions and menu-family registry: `session.rs` (CommandSession, Prompt, SessionInput, SessionStep, CommandError, PlaneFrame), `registry.rs`, `solid/`, `curve/`, `surface/`, `transform/`, `top11/`, `utility/`, `measure/` | no dependency on `om9-app`/egui; `om9-app` calls it from `Controller::dispatch`; created by `OM9-L4-03-18-012-01` (or the first febe L4 written) |

Whichever L4 reaches a new crate first creates it with the same skeleton; later L4 only `modify` (with `from_l4`).

### 9.5 Which crate each M03 L2 targets

| L2 | Kind | Pure-Rust logic | Native adapter | Command/UI |
|---|---|---|---|---|
| OM9-L2-01-09 creation ops | be | `om9-kernel` primitive/profile/extrude/revolve/loft/sweep | `om9-occt` `ops/*` | - |
| OM9-L2-01-10 boolean/fillet/shell | be | `om9-kernel` boolean/blend/shell/heal | `om9-occt` `ops/*` | - |
| OM9-L2-01-11 transforms/patterns | be | `om9-kernel` `xform`, `pattern` (extends `om9_core::geometry::Transform`) | only where a BRep must be moved | - |
| OM9-L2-01-13 `.3dm` import | be + 2 febe | `om9-exchange` | `standalone/native` openNURBS | `om9-app` (picker, options, summary) |
| OM9-L2-02-04 / 02-05 / 02-06 freeform workflows | febe | `om9-kernel` (curve/surface algorithms) | `om9-occt` for BRep dependent ops | `om9-commands` `curve/`, `surface/`, `transform/` |
| OM9-L2-03-08 Top11 | febe | - | - | `om9-commands` `top11/` + `om9-app` toolbar |
| OM9-L2-03-16 / 03-17 / 03-18 / 04-01 menu bindings | febe | owner crates above | owner crates above | `om9-commands` family modules + `om9-app` `Controller::availability` |
| OM9-L2-03-19 / 03-20 utilities and measure | febe | `om9-kernel` (health, bounds, measure) | `om9-occt` (validity checks) | `om9-commands` `utility/`, `measure/` |

---------------------------------------------------------------------------------------------------

## 10. Tests

### 10.1 Exact commands (run from `H:\OpenMatrix9`; verified: exit 0, `1 passed`)

```powershell
# portable test of one L4 (crate in the standalone workspace)
cargo test --locked --manifest-path standalone/Cargo.toml -p <crate> <filter>

# native tests that need OCCT or openNURBS binaries (marked #[ignore])
cargo test --locked --manifest-path standalone/Cargo.toml -p <crate> --features occt <filter> -- --ignored

# done-gate extras (README of standalone/)
cargo fmt --manifest-path standalone/Cargo.toml -p <crate> --check
cargo clippy --locked --manifest-path standalone/Cargo.toml -p <crate> --all-targets -- -D warnings
```

* `--locked` is used by the repo; the first L4 that adds a workspace crate runs one non-locked `cargo build` to update `Cargo.lock`.
* The docs sometimes prefix commands with `rtk` (a local output-compressing proxy); `l4.json` always writes the plain `cargo` form.
* The legacy form `cargo test --manifest-path rust/Cargo.toml <filter>` is accepted by the validator only for explicit legacy maintenance; M03 L4 do not add code there.
* `fixture-tool` L4 in Python use `python -m unittest <dotted.module>` (existing pattern `python -m unittest discover -s tools/tests`).

### 10.2 Naming and placement

* Unit tests of crate-private items: `#[cfg(test)] mod tests` in the same source file (then `tests.file` is that source file).
* Public-API tests: `standalone/crates/<crate>/tests/<topic>_<what>.rs`, topic = the L3 module name.
* Test function names are snake_case, begin with an L4-specific prefix (`prim_`, `box_corners_`) that is also the command filter, and are unique in the crate.
* Floating point: compare analytic pure-Rust results with `TEST_ABS_EPS`, OCCT mass properties with `TEST_VOL_REL_EPS` (glossary).
  Existing tests use the same style (`standalone/crates/om9-core/tests/curve_oracles.rs`).

### 10.3 Portable versus native

OCCT and openNURBS binaries are not guaranteed on a developer machine. Therefore every L4 must have >= 3 portable tests
(input validation, marshalling, error mapping, pure algorithm, prompt flow), >= 1 of them negative. Native tests are additional,
marked `runs_in: native-ignored`, guarded by `#[ignore = "requires OCCT: set OCCT_INCLUDE_DIR and OCCT_LIB_DIR"]`, and invoked through `native_command`.

---------------------------------------------------------------------------------------------------

## 11. Existing reusable symbols (verified by reading the files)

### 11.1 Standalone Rust (`standalone/crates/...`)

| File | Symbols | Use |
|---|---|---|
| `om9-core/src/geometry.rs` | `Point3 = [f64;3]`, `finite(p)`, `valid_domain`, `Bounds`, `Unit::{factor,length_to_mm}`, `Transform::{identity,translation,scale,validate,compose,point}`, `NurbsCurve::{validate,convert_lengths}`, `NurbsSurface::validate`, `Curve::{Line,Circle,Nurbs,Polycurve}`, `Geometry`, `GeometryKind`, `Mesh`, `Cloud` | vector math, validation, unit conversion |
| `om9-core/src/error.rs` | `Error::{Invalid,Budget,Overflow,Missing,Stale,Permission,Unsupported,Canceled,Busy,WrongOwner,Io}`, `Result` | error mapping target |
| `om9-core/src/budget.rs` | `Limits` (default `max_degree` 32, `max_depth` 32, `max_points` 2_000_000), `Usage` | resource limits |
| `om9-core/src/evaluation.rs` | `Parameter::{Native,Fraction}`, `CurveEvaluator::{capture,evaluate}`, `evaluate_curve` | curve evaluation oracle |
| `om9-core/src/history.rs` | `Tolerance { absolute_mm, relative, angular_rad }`, `Tolerance::validate`, `Recipe` | document tolerance carrier |
| `om9-core/src/retained.rs` | `NativeUnits { absolute_tolerance, angular_tolerance_radians, relative_tolerance, millimeters_per_unit }` | `.3dm` unit/tolerance mapping |
| `om9-core/src/capability.rs` | `CapabilityRegistry::{register,availability,admit}`, `Operation`, `Backend::{Rust,Native}`, `Receipt`, `ReceiptScope`, `Availability` | gate commands until receipts exist |
| `om9-core/src/request.rs` | `RequestSchema`, `Role`, `OptionDomain`, `Selection`, `SourceBinding`, `BoundInput`, `AdmittedRequest`, `admit` | selection-to-request binding |
| `om9-core/src/result.rs` | `PreparedOutput`, `PreparedResult`, `ConversionReport`, `Exactness`, `ParameterMap`, `ResultStatus`, `PublishedResult` | publishing operation results |
| `om9-core/src/transaction.rs` | `TransactionCoordinator`, `TransactionRequest::new`, `Mutation::{Placement,Publish,Layers,..}`, `PreparedTransaction`, `CommitReceipt`, `Cancellation`, `World` | atomic publish, Undo |
| `om9-core/src/identity.rs` | `DocumentId`, `ObjectId`, `DefinitionId`, `LayerId`, `SourceStamp` | ids |
| `om9-core/src/model.rs` | `Document`, `Entity`, `Definition`, `Layer`, `Attributes`, `Permissions` | document model |
| `om9-scene/src/camera.rs` | `CPlane { origin, x, .. }`, `ray` | CPlane frame |
| `om9-scene/src/affine.rs` | `AffineGesture::{begin,update,cancel,prepare}` | interactive transforms |
| `om9-scene/src/scene.rs` | `Scene`, `Scene::pick` | picking |
| `om9-app/src/session.rs` | `Controller::{dispatch,submit,prompt,cancel,undo,redo,accept_world_point,availability,preview}`, `InvocationOrigin::{Menu,Cmd}`, `LINE`, `POINT`, `DispatchOutcome` | existing command session pattern (Line/Point) |
| `om9-app/src/transcript.rs` | `Transcript::{set_prompt,append_history,record_command}`, `MAX_BYTES` 64 KiB, `MAX_DRAFT_BYTES` 4096 | CMD frame |
| `om9-exchange/src/lib.rs` | `ImportCapture`, `SourcePreparation`, `ImportOptions`, `ImportMode`, `ImportReceipt`, `PreparedNativeImport`, `UnitResolution`, `NativeExchange`, `ValidatedExport` | `.3dm` import |

### 11.2 Legacy Rust to port (read-only)

`rust/src/solid.rs` (`Session`, `Kind`, `Mode`, prompts, `native_output_valid`, `validate_frame`, `dimension`), `rust/src/command_options.rs` (`OptionSpec::{action,native_input,alias,hidden,disabled}`, `OptionSession`, `OptionContext`), `rust/src/surface.rs` (`Kind`, `Session`, `transport`, `transport_with_height`, `slash_parameter`), `rust/src/tolerances.rs` (`resolve`, `Operation`, `Resolved`), `rust/src/units.rs` (`Context`, `parse_length`, `parse_angle`), `rust/src/geometry_frame.rs` (`Frame::compose`), `rust/src/analytic_support.rs` (`Analytic::capture`), `rust/src/polycurve_contract.rs`, `rust/src/spline.rs`, `rust/src/curve.rs`.

### 11.3 QymCAD samples (read-only, never a dependency)

* OCCT FFI kernel: `QymCAD/crates/qymcad-kernel/` -> `build.rs` (OCCT env vars, libs, MSVC flags), `src/occt_bridge.cpp` (`qym_shape_sphere`, `qym_shape_cylinder`, QYM_SIGNALS try/catch), `src/occt_common.hpp`, `src/shape/build.rs` (`extrude_profile_named`, `revolve_profile_axis_named`, `sweep_profile_named`, `loft_sections_named`, `cylinder_named`, `sphere_named`, `cone_named`, `torus_named`, `boolean`, `boolean_checked`, `fuse_many`), `src/shape/ops.rs` (`shell_center`, `mirrored_plane`, `push_face`, `split_faces`, `split_by_plane`, `thicken_face_join`), `src/ops/blend.rs`, `src/ops/shell.rs`, `src/ops/profile.rs`, `tests/every_entry_catches_what_occt_throws.rs`.
* Math: `QymCAD/crates/qymcad-core/src/{geom,measure,offset,names,refs}.rs`. Sketch/pick: `QymCAD/crates/qymcad-sketch/src/{clicks,corners,offset,snap}.rs`, `QymCAD/crates/qymcad-pick/src/body_hits.rs`. File formats: `QymCAD/crates/qymcad-io/src/*.rs` (STL, OBJ, PLY, DXF, 3MF; no `.3dm`).

---------------------------------------------------------------------------------------------------

## 12. Glossary of tolerances and constants (single source; the validator reads this table)

Use these names and values only. A value differing from this table fails validation. Sources are real repo locations or explicit OM9 test policy.

<!-- GLOSSARY:BEGIN -->
| Name | Value | Unit | Source |
|---|---|---|---|
| DOC_ABS_TOL_MM | 0.001 | mm | Default document absolute tolerance: M01 `OM9-L3-01-08-003` positive case; `om9-core/src/retained.rs` fixture |
| DOC_ANG_TOL_RAD | 0.001 | rad | `om9-core/src/retained.rs` fixture `angular_tolerance_radians`; OM9 default for L4 unless a document supplies another |
| DOC_REL_TOL | 0 | none | `om9-core/src/retained.rs` fixture `relative_tolerance`; relative tolerance off unless an operation sets it |
| MIN_DIM_MM | 1e-7 | mm | `rust/src/solid.rs` `MIN`; `rust/src/curve.rs` minimum segment; smallest accepted dimension |
| MAX_COORD_MM | 1e9 | mm | `rust/src/solid.rs` `finite()`; largest accepted coordinate magnitude |
| JOIN_CURVES_TOL_MM | 1e-7 | mm | `rust/src/tolerances.rs` legacy `Operation::JoinCurves` |
| JOIN_SURFACES_TOL_MM | 1e-6 | mm | `rust/src/tolerances.rs` legacy `Operation::JoinSurfaces` |
| TOL_MIN_MM | 1e-9 | mm | `rust/src/tolerances.rs` lowest accepted effective tolerance |
| TOL_MAX_MM | 1e6 | mm | `rust/src/tolerances.rs` highest accepted effective tolerance |
| NUMERIC_FLOOR_FACTOR | 32 | none | `rust/src/tolerances.rs`: floor = 32 * f64::EPSILON * max(1, coordinate bound) |
| AFFINE_EPS_FACTOR | 64 | none | `rust/src/geometry_frame.rs` EPS and `solid.rs` frame check: 64 * f64::EPSILON |
| FRAME_UNIT_TOL | 1e-12 | none | `om9-core/src/geometry.rs` Circle orthonormal frame check |
| CPLANE_ORTHO_TOL | 1e-6 | none | `rust/src/solid.rs` `validate_frame` orthonormality |
| CPLANE_RIGHT_HAND_MIN | 0.999999 | none | `rust/src/solid.rs` `validate_frame`: z . (x cross y) must be >= this |
| MIN_UNIT_NORM | 1e-10 | none | `rust/src/surface.rs` `unit()` rejects shorter vectors |
| RADIUS_CONSTRAINT_REL_TOL | 1e-6 | none | `rust/src/curve.rs` radius constraint check, relative to max(1, r) |
| NURBS_MAX_DEGREE | 32 | none | `om9-core/src/budget.rs` `Limits::default().max_degree` |
| MAX_NESTING_DEPTH | 32 | none | `om9-core/src/budget.rs` `Limits::default().max_depth` |
| MAX_POINTS | 2000000 | none | `om9-core/src/budget.rs` `Limits::default().max_points` |
| COMMAND_INPUT_MAX_BYTES | 4096 | bytes | `om9-app/src/session.rs` `submit_effect` input cap |
| MM_PER_UM | 0.001 | mm | `om9_core::geometry::Unit::Micrometer.factor()` |
| MM_PER_CM | 10 | mm | `Unit::Centimeter.factor()` |
| MM_PER_M | 1000 | mm | `Unit::Meter.factor()` |
| MM_PER_INCH | 25.4 | mm | `Unit::Inch.factor()` |
| MM_PER_FOOT | 304.8 | mm | `Unit::Foot.factor()` |
| DEG_TO_RAD | 0.017453292519943295 | rad | pi / 180; `rust/src/units.rs` `parse_angle` |
| TEST_ABS_EPS | 1e-10 | mm | OM9 test policy, matches `om9-core/tests/curve_oracles.rs` |
| TEST_ANGLE_EPS_RAD | 1e-9 | rad | OM9 test policy for angle comparisons |
| TEST_VOL_REL_EPS | 1e-6 | none | OM9 test policy for OCCT mass properties against analytic formulas |
<!-- GLOSSARY:END -->

Notes: canonical internal length unit is the millimetre; angles are radians internally and degrees at the UI; a document's own tolerance
overrides `DOC_*` defaults at run time, but L4 test expectations always use the values above.

---------------------------------------------------------------------------------------------------

## 13. How a small model executes an L4

1. Read only: this file's section 4 (schema) once, the L4's `l4.json`, the parent `l3.json`, and the files named in `files.modify`, `reuse_symbols`, `references`.
2. Do not widen scope: everything in `scope.not_in_this_task` belongs to another L4.
3. Create/modify exactly the listed files, with the exact signatures in `api.signatures`.
4. Follow `algorithm.steps` in order; messages and prompts are copied character for character.
5. Write the listed tests first (names exactly as in `tests.cases`), run `tests.command`, then fmt and clippy.
6. A step that cannot be completed is reported, never silently skipped; an `assumptions` entry is changed only by editing the spec first.

### 13.1 Authoring order (for the people or agents who WRITE the L4 specs)

* Cite only L4 ids that exist on disk: an id in `depends_on_l4`, in `from_l4`, or inside any prose (`not_in_this_task`, `behavior_notes`)
  must resolve to a written `l4.json`. Write all L4 of an L3 first, then its `L4_INDEX.md`, then run the validator on the branch.
* Author backend owners before the menu bindings that dispatch to them (`reuses_or_dispatches_to_module_ids` in the binding's `l3.json`),
  so cross-L3 `depends_on_l4` ids exist. The whole-tree DAG must stay acyclic.
* Never edit a pre-existing file to "register" the new L4 (no `children_l4`, no counts). Discovery is by folder scan.
* Keep each folder name <= 36 characters (short name <= 27) and re-run the validator after renames.

---------------------------------------------------------------------------------------------------

## 14. Validator checks (`validate_l4.py`)

Run (PowerShell, any directory):

```powershell
$m03 = 'H:\OpenMatrix9\OM9_Milestones\M03 - Curve, Surface, Solid and 3DM Exchange'
python H:\OpenMatrix9\.agents\tasks\m03-l4\validate_l4.py $m03                         # whole tree
python H:\OpenMatrix9\.agents\tasks\m03-l4\validate_l4.py $m03 L2-01-09                 # one branch (prefix or path)
python H:\OpenMatrix9\.agents\tasks\m03-l4\validate_l4.py $m03 --check-examples H:\OpenMatrix9\.agents\tasks\m03-l4\CONVENTIONS.md
python H:\OpenMatrix9\.agents\tasks\m03-l4\validate_l4.py $m03 --print-digests         # fingerprint of pre-existing files
```

Exit 0 = PASS, 1 = FAIL, 2 = usage error. Checks and counters printed in the summary:

| Check | What it enforces |
|---|---|
| `integrity` | The 2960 pre-existing files (L1/L2/l3/reference) are byte-identical: per-group SHA-256 digests embedded in the script (`EXPECTED_DIGESTS`), plus `L1.json l4_count == 0` and every `l3.json` still `children_l4 == []`, `l4_status == NOT_PLANNED_YET`. Any mismatch is FAIL. |
| `structure` | Only `L4-NNN - Name/{l4.json,L4.md}` and `L4_INDEX.md` are added, only inside L3 folders |
| `tree` | `L1.json` counts match the folders (full run) |
| `coverage` | Every L3 folder has >= 1 L4 folder |
| `json` | Every `l4.json` is UTF-8 (no BOM) JSON without duplicate keys or NaN |
| `schema` | Field presence, types, enums, unknown keys, `goal` one sentence, language rules |
| `identity` | Folder pattern, id derived from parent L3, `seq` = folder = suffix, contiguous 1..N, tag/parent match `l3.json` |
| `size` | `impl_loc <= 250`, `impl_loc + test_loc <= 450` |
| `dag` | `depends_on_l4` ids exist (M03 only), no self loop, planned symbols and modified files come from a listed dependency, **no cycles** |
| `algorithm` | >= 3 numbered steps, >= 2 edge cases, >= 1 failure, tolerance names/values match section 12 |
| `interaction` | Object or `null` + reason; febe L3 has >= 1 interactive L4; disabled options carry a reason |
| `tests` | >= 3 portable cases, >= 1 negative, valid commands, names contain the filter, test file declared |
| `links` | Reference text/images exist inside M03, QymCAD and legacy paths exist, existing symbols found, L2/L3/L4 ids cited anywhere exist |
| `placeholders` | No TODO/TBD/placeholder style text in any value, `L4.md` or `L4_INDEX.md` |
| `pathlen` | Every added file path (actual and canonical `H:\` length) < 240 |
| `index` | `L4_INDEX.md` format of 8.3 |
| `l4md` | Optional `L4.md` format of 8.4 |

The validator is read-only for the tree. If an approved change to pre-existing files is ever needed, regenerate digests with
`--print-digests > digests.json` and pass `--expected-digests digests.json`; that is a deliberate act, never a way to hide an edit.
