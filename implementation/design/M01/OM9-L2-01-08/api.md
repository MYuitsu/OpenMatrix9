# `OM9-L2-01-08` — Be Units Tolerances Basic Parameters: API contract

Milestone `M01` · Stage `E01` · Planning state `SPEC_ONLY_NOT_IMPLEMENTED` (the tree is not edited)
Audited working tree: `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` + uncommitted UI edits (see `crosswalk.json`)
M00: only QymCAD code reuse is approved (user, 2026-10-16). OCCT, UI, 3DM, Associative, License and
Reproducible are not. Items that need them are marked **NEEDS_USER_M00_DECISION** below.

Status: 0 DONE_WITH_EVIDENCE, 6 PARTIAL, 1 MISSING (`crosswalk.json`). Oracles: `oracles.json`.

## 1. Scope and exclusions

**In scope.** Canonical millimetre lengths, degree angles with radian conversion, one document
tolerance policy, typed numeric parameter values, simple min/max/step parameter validation,
unit-safe serialization of those values and typed world/local coordinate input.

**Out of scope.** Sketch solvers or formula dependencies (L2 exclusion); OCCT tolerance binding
(OM9-L2-01-05, M00 OCCT not approved); any UI; the CMD option grammar (OM9-L2-01-02 owns it and calls
these primitives); 3DM unit import (retained `NativeUnits` stays as is).

## 2. Ownership (single owner per concept, nothing duplicated)

| Concept | Owner (existing or new) | Decision |
| --- | --- | --- |
| Unit enum, factors | `om9-core/src/geometry.rs:37-63` `Unit` | REUSE_IN_PLACE, add `Unit::parse`, `Unit::symbol` |
| Typed length / angle | `om9-core/src/units.rs` (new) | ADD |
| Tolerance type | `om9-core/src/history.rs:14-31` `Tolerance` | EXTEND (no second tolerance type) |
| Document tolerance | `om9-core/src/model.rs` `Document` + `transaction.rs` `Mutation::Tolerance` | EXTEND |
| Typed parameter value | `om9-core/src/request.rs:24-30` `Value` | EXTEND (it already is the typed enum; no `ParamValue`) |
| Parameter domain/range | `om9-core/src/request.rs:65-72` `OptionDomain` | EXTEND (`step`, `validate`, `check`) |
| Coordinates | `om9-core/src/coords.rs` (new) + `om9-scene/src/camera.rs` `impl LocalFrame for CPlane` | ADD / ADAPT |
| CMD coordinate text | `om9-app/src/session.rs:318-330` | delegates to `coords::parse_local` (behavior-preserving) |

ADR-08-1: the planning record proposes `LengthMm::try_new`, `Tolerance::try_new(abs, rel)`. OM9 keeps
`history::Tolerance` with its third field `angular_rad`, so the constructor is
`Tolerance::try_new(absolute_mm, relative, angular_rad)`. No module is copied or duplicated.

ADR-08-2: canonical stored units are fixed: `Dimension::Length` values are millimetres,
`Dimension::Angle` values are **degrees** (user-facing Matrix/Rhino convention), `Ratio` is
dimensionless, `Count` is a non-negative integer. Kernel/geometry code converts to radians through
`AngleDeg::to_radians` only.

## 3. Typed API

```rust
// om9-core/src/units.rs  (new; #![forbid(unsafe_code)] crate)
pub const MAX_ABS_DEGREES: f64 = 1.0e6;          // OM9 numeric admission bound, not Matrix data

#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    NonFinite { dimension: Dimension },
    OutOfRange { dimension: Dimension, value: f64, limit: f64 },
    UnsupportedUnit { token: String },           // token copy bounded to 16 bytes
    MissingUnit { expected: Dimension },         // a bare number where a dimensioned value is required
    WrongDimension { expected: Dimension, found: Dimension },
    NotAnInteger { value: f64 },
    Malformed,                                   // numeric text does not parse
}
impl std::fmt::Display for UnitError; impl std::error::Error for UnitError;
impl From<UnitError> for crate::Error;           // UnsupportedUnit -> Unsupported("unit"),
                                                 // every other variant -> Invalid(<static label>)

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "LengthRepr")]   // wire {"mm": 2.5}; LengthRepr is deny_unknown_fields
pub struct LengthMm { mm: f64 }
impl LengthMm {
    pub fn try_new(mm: f64) -> Result<Self, UnitError>;            // NaN/±inf -> NonFinite
    pub fn from_unit(value: f64, unit: Unit) -> Result<Self, UnitError>; // value*factor, finite check
    pub fn parse(text: &str, default: Unit) -> Result<Self, UnitError>;  // "2.5", "2.5mm", "0.1 in"
    pub fn mm(self) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "DegreesRepr")]  // wire {"deg": 45.0}; DegreesRepr is deny_unknown_fields
pub struct AngleDeg { deg: f64 }
impl AngleDeg {
    pub fn try_new(deg: f64) -> Result<Self, UnitError>;  // non-finite or |deg| > MAX_ABS_DEGREES rejected
    pub fn parse(text: &str) -> Result<Self, UnitError>;  // "90", "90deg", "90°", "1.5707963rad" (explicit rad converts)
    pub fn deg(self) -> f64;
    pub fn to_radians(self) -> AngleRad;
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "RadiansRepr")]  // wire {"rad": 3.14159...}; RadiansRepr is deny_unknown_fields
pub struct AngleRad { rad: f64 }
impl AngleRad { pub fn try_new(rad: f64) -> Result<Self, UnitError>; pub fn rad(self) -> f64;
                pub fn to_degrees(self) -> Result<AngleDeg, UnitError>; }

// om9-core/src/geometry.rs (Unit owner)
impl Unit {
    pub fn parse(token: &str) -> Result<Unit, UnitError>; // mm|cm|m|in|ft|um|µm, ASCII case-insensitive
    pub fn symbol(self) -> &'static str;                   // "mm", "cm", "m", "in", "ft", "um"
}
```

Wire forms are tagged by unit: `{"deg": ..}` cannot deserialize into `LengthMm` or `AngleRad`, so a
value carrying radians cannot be read as degrees and a bare number cannot be read as millimetres.

```rust
// om9-core/src/history.rs  (Tolerance owner)
impl Tolerance {
    pub fn try_new(absolute_mm: f64, relative: f64, angular_rad: f64) -> crate::Result<Self>;
        // absolute_mm finite and > 0; relative, angular_rad finite and >= 0
        // labels: Invalid("absolute tolerance"), Invalid("relative tolerance"), Invalid("angular tolerance")
    pub fn admit_feature(&self, size: LengthMm) -> crate::Result<()>;
        // size.mm() <= absolute_mm -> Err(Invalid("feature at or below document absolute tolerance"))
}

// om9-core/src/model.rs
pub struct Document { /* existing fields */ ,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<crate::history::Tolerance>,       // see M00 gate below
}
impl Document {
    pub fn tolerance(&self) -> Option<&Tolerance>;
    pub fn effective_tolerance(&self) -> crate::Result<Tolerance>; // Err(Missing("document tolerance")) when unset
}
// Document::new keeps tolerance = None: no default is invented (no local source states one).
// validate_internal: if Some(t) then t.validate()?; Usage unchanged (inline Copy-sized data).

// om9-core/src/transaction.rs
pub enum Mutation { /* existing */ ,
    Tolerance { document: DocumentId, expected_revision: u64, tolerance: Tolerance },
}
// participants: [document]; admission: base.document(d).revision == expected_revision else
// Stale("document tolerance revision"); tolerance.validate()?; staging: d.tolerance = Some(tolerance).
// It does NOT bump context_revision (stamps and Clean recipes stay valid; recipes keep their own
// ReplayEnvironment.tolerance). One transaction, one Undo entry, document.revision + 1.
```

```rust
// om9-core/src/request.rs  (Value / OptionDomain owner)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]   // Serialize/Deserialize/PartialEq added
pub enum OptionDomain {
    Number { dimension: Dimension, min: Option<f64>, min_inclusive: bool, max: Option<f64>,
             #[serde(default)] step: Option<f64> },               // NEW field
    Choice(Vec<String>), Boolean, Text { max_bytes: usize },
}
impl OptionDomain {
    pub fn validate(&self) -> crate::Result<()>;
        // Number: bounds finite, min <= max (min < max when !min_inclusive and min == max is empty),
        //   step finite and > 0; Choice non-empty and unique; Text max_bytes > 0
    pub fn check(&self, value: &Value, limits: &Limits) -> crate::Result<()>;
        // the former private option(): Number domains admit ONLY Value::Quantity with the same
        // dimension (bare Value::Number -> UnitError::MissingUnit), finite, inside [min, max],
        // Count integral >= 0, and with step: k = (n - min.unwrap_or(0)) / step,
        // |k - k.round()| <= 1e-9 else Invalid("numeric option step")
}
impl Value {
    pub fn length(v: LengthMm) -> Self;   pub fn angle(v: AngleDeg) -> Self;
    pub fn count(n: u32) -> Self;         pub fn ratio(r: f64) -> crate::Result<Self>; // finite
    pub fn as_length(&self) -> Result<LengthMm, UnitError>; // Quantity{Length} only; Number -> MissingUnit
    pub fn as_angle(&self) -> Result<AngleDeg, UnitError>;
    pub fn as_count(&self) -> Result<u32, UnitError>;       // 2.5 -> NotAnInteger
    pub fn as_ratio(&self) -> Result<f64, UnitError>;
}
// request::admit additionally calls domain.validate() for every schema option before admission.
```

```rust
// om9-core/src/coords.rs  (new)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct WorldPoint { xyz: Point3 }   // try_new(finite), xyz()
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct LocalPoint { xyz: Point3 }   // try_new(finite), xyz()
pub trait LocalFrame {
    fn to_world(&self, p: LocalPoint) -> crate::Result<WorldPoint>;
    fn to_local(&self, p: WorldPoint) -> crate::Result<LocalPoint>;
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateInput { World(WorldPoint), Local(LocalPoint) }
pub fn resolve(input: CoordinateInput, frame: Option<&dyn LocalFrame>) -> crate::Result<WorldPoint>;
    // Local + None -> Err(Missing("coordinate frame")); no From<LocalPoint> for WorldPoint exists
pub fn parse_local(text: &str, unit: Unit) -> crate::Result<LocalPoint>;
    // exactly the current session.rs grammar: 2..=3 comma-separated plain f64 tokens, z defaults to 0,
    // each converted with unit.length_to_mm; SAME error labels as today:
    // Invalid("x,y[,z] coordinate"), Invalid("coordinate number"), Invalid("length conversion")

// om9-scene/src/camera.rs
impl om9_core::coords::LocalFrame for CPlane { /* to_world = CPlane::point; to_local = dot with x, y, normal */ }
```

`session.rs:318-330` is rewritten to `coords::parse_local(text, s.unit)` then
`coords::resolve(CoordinateInput::Local(p), Some(&s.plane))`. Prompts, accepted syntax, error text
and geometry stay identical (the existing `command_sessions` tests must pass unchanged).

## 4. Serialization and the Line-save defect

`standalone/Cargo.toml:13` changes from `serde_json = "=1.0.150"` to
`serde_json = { version = "=1.0.150", features = ["float_roundtrip"] }`. The writer is untouched, so
saved bytes are identical; only parsing becomes exact. This closes the known
`Invalid("archive encoding verification")` on saving a Line (root cause and probe numbers in
`crosswalk.json` → `line_save_bug`). No archive logic changes; owner of the codec remains OM9-L2-01-12.

## 5. M00 gates (stop and ask, `send_message` severity `warning`)

| Item | Why | Coder action |
| --- | --- | --- |
| Persisting `Document.tolerance` (and, in L2-01-03, `Document.groups`, `Entity.extensions`) | adds keys to the `.om9` metadata: archive format decision | Implement everything in memory first, then ask ONE consolidated question (options in `IMPLEMENTATION-CHECKLIST.md` §0). Record the answer in `.agents/tasks/m01-l2-01-02-03-backend/m00-decisions.md`; later loops reuse it. |
| Tolerance pass-through to a kernel | OCCT not approved; OM9-L2-01-05 absent | Only `Document::effective_tolerance()` is provided and tested; no OCCT binding. Mark the sub-oracle `NEEDS_USER_M00_DECISION`. |
| Any changed golden bytes / fingerprints after `float_roundtrip` | would be a format/Reproducible change | Stop and ask. |

## 6. Test design (RED first, run with `cargo test -p <crate> --test <file>` in `standalone/`)

| Test file | Oracle covered |
| --- | --- |
| `om9-archive/tests/float_roundtrip.rs` (new) | Line 0,0,0 → 12,6,0 saves and reopens bit-exact (RED: encoding verification error) |
| `om9-core/tests/units_parameters.rs` (new) | LengthMm/AngleDeg/AngleRad/Unit::parse positive and negative; Value typed accessors; OptionDomain step/validate/check; serde wire forms; exact JSON round-trip of `12f64.hypot(6.)` |
| `om9-core/tests/document_tolerance.rs` (new) | `Mutation::Tolerance` one Undo entry; zero/negative/NaN rejected with snapshot unchanged; `admit_feature`; `effective_tolerance` Missing when unset |
| `om9-core/tests/coordinates.rs` (new) | World vs Local with two frames; Local without frame refused; `parse_local` labels |
| `om9-core/tests/source_binding.rs` (existing) | must still pass; the `Value::Number(0.)` negative stays negative |
| `om9-app/tests/command_sessions.rs` (existing) | unchanged CMD numeric behavior after delegation |

## 7. Absent tooling and unverified items

- `OpenMatrix9_Codex_Spec_v2/` is absent; RCORE-00/CODE_GUIDE/BACKEND_POLICY could not be read.
- `tools/validate_m01_spec.py`, `tools/m01_proof.py`, `om9.m01.oracle.v1.5`: not present locally; the
  schema string is carried as a field only.
- No local source states a default document tolerance; `Document::new` therefore leaves it unset.
- `MAX_ABS_DEGREES = 1e6` is an OM9 admission bound chosen here, not recovered Matrix behavior.

## 8. Implementation record (2026-10-17, W1 closed)

Status: **IMPLEMENTED** — all 7 L3 have code, runtime tests, fixtures and evidence
(`implementation/evidence/M01/OM9-L3-01-08-00N/`); dashboard `OM9-L2-01-08 · ✅7 🟡0 ⬜0`.

| L3 | Owner symbol | Test |
| --- | --- | --- |
| 001 | `units::LengthMm`, `geometry::Unit::{parse,symbol}` | `units_parameters` (l3_001), `command_options_and_repeat::a_bare_option_number_follows_the_document_unit` |
| 002 | `units::{AngleDeg,AngleRad,MAX_ABS_DEGREES}` | `units_parameters` (l3_002) |
| 003 | `history::Tolerance::{try_new,admit_feature,om9_template}`, `Document::{tolerance,effective_tolerance,with_template_tolerance}`, `Mutation::Tolerance` | `document_tolerance`, `document_identity_round_trip::a_document_tolerance_is_persisted_admitted_and_refused_in_a_legacy_payload` |
| 004 | `request::Value::{length,angle,count,ratio,as_*}` | `units_parameters` (l3_004) |
| 005 | `request::OptionDomain::{validate,check,step}` | `units_parameters` (l3_005), `source_binding` |
| 006 | unit-tagged serde wire forms | `units_parameters` (l3_006) |
| 007 | `coords::*`, `impl LocalFrame for CPlane` | `coordinates`, `coordinate_frame` (scene) |

Two points closed after the first pass (owner asked to finish W1):

- **Persistence.** `Document.tolerance` is an additive key: it travels with the document (the container
  encodes the whole `Document`), and `json_admission` now refuses `(Document,"tolerance")` in a legacy
  schema-3 payload with `Invalid("legacy schema has no document tolerance field")`. A document without a
  policy writes no key and decodes byte-identically.
- **Template policy.** `Tolerance::om9_template()` (0.001 mm absolute, 0.01 % relative, one degree
  angular) plus `Document::with_template_tolerance(name)`; the app uses it for a **new user document**
  only. `Document::new` still states no tolerance, so the policy is written into the document instead of
  being hidden in the reader. The numbers are an OM9 template decision (no accessible Matrix page
  confirms them) and a document template may override them.
- **Legacy split.** `rust/src/cplanes.rs` holds a named-plane `Registry`/`Snapshot`; that registry is
  `OM9-L2-03-01` work. This module delivers the frame half (`CPlane` as the typed `LocalFrame`).
  `implementation/design/RUST-LEGACY-MIGRATION.md` classifies the module as `PARTIAL` accordingly.

## 9. Precision and performance contract (2026-10-17, accuracy pass)

CAD accuracy needs a stated guarantee, so this section records the measured behaviour of the
numeric layer. Everything below is measured by `crates/om9-core/tests/l2_01_08_precision.rs`
(deterministic xorshift sweep, 20 000 samples per unit).

### Unit conversion

`Unit::ratio()` returns the **exact integer ratio** `(numerator, denominator)` instead of a rounded
decimal factor: mm `1/1`, cm `10/1`, m `1000/1`, **in `127/5`**, **ft `1524/5`**, µm `1/1000`.
`LengthMm::from_unit`/`in_unit` then convert with `mul_div`, a fused-multiply-add compensated
`a * b / d`:

```
product  = a * b
residual = fma(a, b, -product)          // exact product error
quotient = product / d
remainder= fma(quotient, -d, product)   // exact quotient remainder
result   = quotient + (remainder + residual) / d
```

| Property | Before this pass | After |
| --- | --- | --- |
| Rounding steps per conversion | 2 (rounded factor + product) | 1 (final addition only) |
| `mm → unit → mm` round trip | **2 ULP** (measured: `-5999.633406612266` inch → 2 ULP) | **≤ 1 ULP** (20 000 samples × 6 units, all pass) |
| `1 in`, `1 ft`, `1000 µm` | factor rounding | exact (`25.4`, `304.8`, `1.0` bit-exact) |
| `25.4 in → mm → in` | rounded | lossless (exact values survive) |

### Angles

Degrees are canonical. `to_radians` uses `mul_div(deg, PI, 180)` and `to_degrees` uses
`mul_div(rad, 180, PI)`, which makes the quadrant boundaries exact:

| Input | Result | Exact? |
| --- | --- | --- |
| 180° | `std::f64::consts::PI` | bit-exact |
| 90° | `PI / 2` | bit-exact |
| 360° | `2 * PI` | bit-exact |
| `PI` rad | 180° | bit-exact |
| degree → radian → degree, 20 000 samples in ±1e6° | ≤ 1 ULP | yes (was 2 ULP) |

### Tolerance comparisons

`history::within_tolerance(a, b, tolerance)` is the documented rule: equal when
`|a - b| <= tolerance`, **exact comparison, no epsilon fuzz**, and never true for a non-finite
input. `Tolerance::admit_feature` uses `<=`, so geometry exactly at the absolute tolerance is
refused and one ULP above it is admitted; the test pins both sides of the boundary.

### Admission hot path

Two changes, both measured:

1. `OptionDomain::validate` validates a choice set **without allocating** for ≤ 32 choices (the
   size a command schema actually carries) and falls back to a `BTreeSet` only above that. It used
   to allocate a set per call, and `check` called it per value.
2. `OptionDomain::check_admitted` is the value rule without re-validating the schema; `admit`
   validates every declared domain once up front and then uses `check_admitted`, so a command with
   N options no longer re-checks its schema N times per admission.

Measured on the development machine, 200 000 operations each:

| Operation | Debug | **Release** |
| --- | --- | --- |
| `from_unit` + `in_unit` (inch round trip) | 8.5 ms (42 ns/pair) | **1.46 ms — 7 ns/pair** |
| `check_admitted` on a 32-choice domain | 33.8 ms (169 ns/value) | **10.8 ms — 54 ns/value** |
| the same, rejecting value | same cost class | same cost class (a 32-entry linear scan) |

The choice scan is the dominant cost of `check_admitted` (32 `&str` comparisons). A command schema
that needs more than 32 choices keeps the allocating `BTreeSet` path, which is the right trade at
that size.

The tests assert generous ceilings (2 s for 200 000 operations) so a real regression trips them
without flaking on a loaded machine, and print the measured rate for the record.
