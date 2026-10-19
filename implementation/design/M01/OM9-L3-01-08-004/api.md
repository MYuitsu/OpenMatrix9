# OM9-L3-01-08-004 — Typed Numeric Parameter Values (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/request.rs`. Legacy decision: **ADAPT**.

Oracle — positive: *Value giữ đúng chiều; đọc lại bằng as_length/as_angle/as_count/as_ratio.*
Negative: *Số trần không phải length/angle; count 2.5 là NotAnInteger; sai chiều là WrongDimension.*

## 1. Typed API

```rust
impl Value { pub fn length(LengthMm) -> Self; pub fn angle(AngleDeg) -> Self;
             pub fn count(u32) -> Self; pub fn ratio(f64) -> Result<Self>;
             pub fn as_length(&self) -> Result<LengthMm, UnitError>;
             pub fn as_angle(&self) -> Result<AngleDeg, UnitError>;
             pub fn as_count(&self) -> Result<u32, UnitError>;
             pub fn as_ratio(&self) -> Result<f64, UnitError>; }
```

## 2. Errors (exact strings)

- `UnitError::MissingUnit { .. }      -> Invalid("missing unit")`
- `UnitError::WrongDimension { .. }   -> Invalid("numeric dimension")`
- `UnitError::NotAnInteger { .. }     -> Invalid("integer value")`

## 3. Legacy `rust/` audit

- `rust/src/command_options.rs`
- `rust/src/units.rs (Dimension)`
- `rust/src/typed option parsing`

Legacy `command_options.rs` carries dimensioned option values through C; OM9 keeps the typed value in Rust and reads it back typed.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test units_parameters l3_004`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-004/OM9-L4-01-08-004-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-004/OM9-L4-01-08-004-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-004/OM9-L4-01-08-004-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-004/positive.json`, `negative.json`
