# OM9-L3-01-08-001 — Millimeter World Unit Model (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/units.rs, crates/om9-core/src/geometry.rs`. Legacy decision: **ADAPT**.

Oracle — positive: *Độ dài có kiểu trong milimét; số trần trong lệnh dùng đơn vị tài liệu, hậu tố tường minh thắng; token đơn vị không phân biệt hoa thường và chỉ có một bảng đơn vị.*
Negative: *NaN/inf bị từ chối; đơn vị lạ là Unsupported; chữ số không parse được là Malformed.*

## 1. Typed API

```rust
pub struct LengthMm { mm: f64 }   // #[serde(try_from = "LengthRepr")], wire {"mm": 2.5}
impl LengthMm { pub fn try_new(mm: f64) -> Result<Self, UnitError>;
                pub fn from_unit(value: f64, unit: Unit) -> Result<Self, UnitError>;
                pub fn parse(text: &str, default: Unit) -> Result<Self, UnitError>;
                pub fn mm(self) -> f64; }
impl Unit { pub fn parse(token: &str) -> Result<Unit, UnitError>; pub fn symbol(self) -> &'static str; }
```

## 2. Errors (exact strings)

- `UnitError::NonFinite { dimension: Length }  -> Invalid("finite numeric value")`
- `UnitError::UnsupportedUnit { token }        -> Unsupported("unit")`
- `UnitError::Malformed                        -> Invalid("numeric text")`

## 3. Legacy `rust/` audit

- `rust/src/units.rs`
- `tests: rust/tests/units.rs, numeric_units.rs, tolerances.rs`

Legacy `units.rs` owns the same ideas through a C ABI (Context/Dimension/UnitChangeMode). The portable part — canonical mm, unit tokens, checked conversion — is re-implemented in safe Rust with typed values; the FFI context and the C error codes are not ported.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test units_parameters l3_001`
- `cargo test -p om9-app --test command_options_and_repeat a_bare_option_number`
- `cargo test -p om9-core --test l2_01_08_precision`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-001/OM9-L4-01-08-001-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-001/OM9-L4-01-08-001-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-001/OM9-L4-01-08-001-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-001/positive.json`, `negative.json`
