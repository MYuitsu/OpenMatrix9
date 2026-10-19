# OM9-L3-01-08-006 — Unit Safe Parameter Serialization (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/units.rs, request.rs`. Legacy decision: **ADAPT**.

Oracle — positive: *Wire form có tag đơn vị, round-trip bit-chính-xác cho f64.*
Negative: *Payload radian không đọc được thành độ; số trần không đọc được thành mm.*

## 1. Typed API

```rust
serde wire: LengthMm {"mm":..}, AngleDeg {"deg":..}, AngleRad {"rad":..}; other keys refused by deny_unknown_fields
Value::Quantity { value, dimension } keeps the dimension through JSON
```

## 2. Errors (exact strings)

- `a degrees payload cannot deserialize into LengthMm (and vice versa)`
- `a bare number cannot deserialize into a typed unit value`

## 3. Legacy `rust/` audit

- `rust/src/units.rs`
- `rust/src/prepared_result.rs (stored options)`

Legacy stored a raw f64 with a separate context; OM9 stores the unit in the value, so a stored number can never be read with the wrong dimension.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test units_parameters l3_006`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-006/OM9-L4-01-08-006-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-006/OM9-L4-01-08-006-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-006/OM9-L4-01-08-006-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-006/positive.json`, `negative.json`
