# OM9-L3-01-08-002 — Angle Conversion And Degrees (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/units.rs`. Legacy decision: **ADAPT**.

Oracle — positive: *Góc lưu bằng độ; đổi radian chính xác tới 8.9e-16; text radian chuyển tường minh.*
Negative: *|deg| vượt MAX_ABS_DEGREES bị OutOfRange; radian không đọc lại được thành độ.*

## 1. Typed API

```rust
pub const MAX_ABS_DEGREES: f64 = 1.0e6;
pub struct AngleDeg { deg: f64 }   // wire {"deg": 45.0}
pub struct AngleRad { rad: f64 }   // wire {"rad": 0.785...}
impl AngleDeg { pub fn try_new(deg: f64) -> Result<Self, UnitError>; pub fn parse(text: &str) -> Result<Self, UnitError>;
                pub fn deg(self) -> f64; pub fn to_radians(self) -> AngleRad; }
impl AngleRad { pub fn try_new(rad: f64) -> Result<Self, UnitError>; pub fn rad(self) -> f64;
                pub fn to_degrees(self) -> Result<AngleDeg, UnitError>; }
```

## 2. Errors (exact strings)

- `UnitError::OutOfRange { dimension: Angle, .. } -> Invalid("numeric value range")`
- `UnitError::NonFinite { dimension: Angle }      -> Invalid("finite numeric value")`

## 3. Legacy `rust/` audit

- `rust/src/units.rs (Dimension/AngleRadians)`
- `rust/src/core_angle.rs`

Legacy angle handling lives inside the C context (angular_rad) and `core_angle.rs`. OM9 keeps degrees as the stored unit and makes the radian boundary explicit; no native call is needed.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test units_parameters l3_002`
- `cargo test -p om9-core --test l2_01_08_precision`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-002/OM9-L4-01-08-002-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-002/OM9-L4-01-08-002-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-002/OM9-L4-01-08-002-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-002/positive.json`, `negative.json`
